use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::fs_util::Tool;
use crate::linker;
use crate::tagger;

use super::app::{App, Panel, Screen, TextInputState, TreeRow};

impl App {
    pub(super) fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        if self.text_input.is_some() {
            return self.handle_text_input_key(code);
        }

        self.status_msg.clear();

        match code {
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('1') => {
                self.screen = Screen::Skills;
                self.panel = Panel::Left;
            }
            KeyCode::Char('2') => {
                self.screen = Screen::Projects;
                self.panel = Panel::Left;
                self.rebuild_tree();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.panel = match self.panel {
                    Panel::Left => Panel::Right,
                    Panel::Right => Panel::Left,
                };
            }
            _ => match self.screen {
                Screen::Skills => self.handle_skills_screen_key(code)?,
                Screen::Projects => self.handle_projects_screen_key(code)?,
            },
        }

        self.clamp_all_selections();
        Ok(())
    }

    // ── Screen 1: Skills & Tags ──

    fn handle_skills_screen_key(&mut self, code: KeyCode) -> Result<()> {
        match self.panel {
            Panel::Left => self.handle_skill_list_key(code),
            Panel::Right => self.handle_tag_list_key(code),
        }
    }

    fn handle_skill_list_key(&mut self, code: KeyCode) -> Result<()> {
        let len = self.skill_dirs.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.skills_state.selected_skill =
                    self.skills_state.selected_skill.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.skills_state.selected_skill + 1 < len {
                    self.skills_state.selected_skill += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_tag_list_key(&mut self, code: KeyCode) -> Result<()> {
        let len = self.all_tags.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.skills_state.selected_tag = self.skills_state.selected_tag.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.skills_state.selected_tag + 1 < len {
                    self.skills_state.selected_tag += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.toggle_tag_on_selected_skill()?;
            }
            KeyCode::Char('a') => {
                self.text_input = Some(TextInputState {
                    input: String::new(),
                    cursor: 0,
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn toggle_tag_on_selected_skill(&mut self) -> Result<()> {
        let Some(skill) = self
            .skill_dirs
            .get(self.skills_state.selected_skill)
            .cloned()
        else {
            return Ok(());
        };
        let Some(tag) = self.all_tags.get(self.skills_state.selected_tag).cloned() else {
            return Ok(());
        };

        let has_tag = self
            .tag_map
            .get(&skill)
            .is_some_and(|tags| tags.contains(&tag));

        if has_tag {
            tagger::remove_tag(&self.paths, &skill, &tag)?;
            self.status_msg = format!("{}: removed [{}]", skill, tag);
        } else {
            tagger::add_tag(&self.paths, &skill, &tag)?;
            self.status_msg = format!("{}: added [{}]", skill, tag);
        }
        self.reload()?;
        Ok(())
    }

    // ── Screen 2: Projects ──

    fn handle_projects_screen_key(&mut self, code: KeyCode) -> Result<()> {
        match self.panel {
            Panel::Left => self.handle_project_list_key(code),
            Panel::Right => self.handle_tree_key(code),
        }
    }

    fn handle_tree_key(&mut self, code: KeyCode) -> Result<()> {
        let len = self.tree_rows.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.projects_state.tree_cursor = self.projects_state.tree_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.projects_state.tree_cursor + 1 < len {
                    self.projects_state.tree_cursor += 1;
                }
            }
            KeyCode::Enter => {
                self.toggle_tree_collapse();
            }
            KeyCode::Char(' ') => {
                self.toggle_tree_link()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn toggle_tree_collapse(&mut self) {
        let Some(row) = self.tree_rows.get(self.projects_state.tree_cursor) else {
            return;
        };
        match row {
            TreeRow::TagHeader { tag, .. } => {
                let tag = tag.clone();
                if !self.projects_state.collapsed_tags.remove(&tag) {
                    self.projects_state.collapsed_tags.insert(tag);
                }
                self.rebuild_tree();
            }
            TreeRow::UntaggedHeader { .. } => {
                let key = "__untagged__".to_string();
                if !self.projects_state.collapsed_tags.remove(&key) {
                    self.projects_state.collapsed_tags.insert(key);
                }
                self.rebuild_tree();
            }
            _ => {}
        }
    }

    fn toggle_tree_link(&mut self) -> Result<()> {
        let Some(project) = self.selected_project_path().map(|s| s.to_string()) else {
            self.status_msg = "No project selected".to_string();
            return Ok(());
        };
        let Some(row) = self.tree_rows.get(self.projects_state.tree_cursor).cloned() else {
            return Ok(());
        };

        let project_path = std::path::Path::new(&project);
        let project_name = crate::fs_util::basename(&project);

        match row {
            TreeRow::TagHeader { tag, .. } => {
                let skills = self.skills_for_tag(&tag);
                let all_linked = skills.iter().all(|s| self.is_skill_linked_to_selected(s));

                if all_linked {
                    for skill in &skills {
                        linker::unlink_skill(project_path, skill, Tool::Claude)?;
                    }
                    self.status_msg = format!("Unlinked [{}] from {}", tag, project_name);
                } else {
                    let count = linker::link_by_tags(
                        &self.paths,
                        project_path,
                        std::slice::from_ref(&tag),
                        Tool::Claude,
                    )?;
                    self.status_msg =
                        format!("Linked [{}] to {} ({} skills)", tag, project_name, count);
                }
                self.reload_project_links(&project);
            }
            TreeRow::Skill { skill } | TreeRow::UntaggedSkill { skill } => {
                if self.is_skill_linked_to_selected(&skill) {
                    linker::unlink_skill(project_path, &skill, Tool::Claude)?;
                    self.status_msg = format!("Unlinked {} from {}", skill, project_name);
                } else {
                    linker::link_skill(&self.paths, project_path, &skill, Tool::Claude)?;
                    self.status_msg = format!("Linked {} to {}", skill, project_name);
                }
                self.reload_project_links(&project);
            }
            TreeRow::UntaggedHeader { .. } => {}
        }
        Ok(())
    }

    fn handle_project_list_key(&mut self, code: KeyCode) -> Result<()> {
        let len = self.project_paths.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.projects_state.selected_project =
                    self.projects_state.selected_project.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 && self.projects_state.selected_project + 1 < len {
                    self.projects_state.selected_project += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── Text Input Modal ──

    fn handle_text_input_key(&mut self, code: KeyCode) -> Result<()> {
        let state = self.text_input.as_mut().unwrap();
        match code {
            KeyCode::Enter => {
                let new_tag = state.input.trim().to_string();
                self.text_input = None;
                if new_tag.is_empty() {
                    return Ok(());
                }
                if let Some(skill) = self
                    .skill_dirs
                    .get(self.skills_state.selected_skill)
                    .cloned()
                {
                    tagger::add_tag(&self.paths, &skill, &new_tag)?;
                    self.reload()?;
                    self.status_msg = format!("{}: added [{}]", skill, new_tag);
                }
            }
            KeyCode::Esc => {
                self.text_input = None;
            }
            KeyCode::Char(c) => {
                state.input.insert(state.cursor, c);
                state.cursor += c.len_utf8();
            }
            KeyCode::Backspace => {
                if state.cursor > 0 {
                    let prev = state.input[..state.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    state.input.replace_range(prev..state.cursor, "");
                    state.cursor = prev;
                }
            }
            KeyCode::Left => {
                if state.cursor > 0 {
                    state.cursor = state.input[..state.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                }
            }
            KeyCode::Right => {
                if state.cursor < state.input.len() {
                    state.cursor += state.input[state.cursor..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
