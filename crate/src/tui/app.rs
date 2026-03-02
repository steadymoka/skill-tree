use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;

use crate::config::Paths;
use crate::fs_util::Tool;
use crate::linker;
use crate::scanner;
use crate::tagger;
use crate::yaml::{self, SkillTagMap};

use super::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tags,
    Skills,
    Projects,
}

pub struct App {
    pub(super) paths: Paths,
    pub(super) tag_map: SkillTagMap,
    pub(super) all_tags: Vec<String>,
    pub(super) project_paths: Vec<String>,

    pub(super) focus: Focus,
    pub(super) selected_tag: usize,
    pub(super) selected_skill: usize,
    pub(super) selected_project: usize,
    pub(super) project_links: BTreeMap<String, HashSet<String>>,
    pub(super) status_msg: String,

    skill_dirs: Vec<String>,
    skill_dir_set: HashSet<String>,
    should_quit: bool,

    /// Tag editing state: when Some, we are in inline edit mode
    pub(super) editing_tags: Option<TagEditState>,
}

pub(super) struct TagEditState {
    pub skill_name: String,
    pub input: String,
    pub cursor: usize,
}

impl App {
    pub fn new(paths: Paths, project_paths: Vec<String>) -> Result<Self> {
        let mut app = App {
            paths,
            tag_map: BTreeMap::new(),
            skill_dirs: Vec::new(),
            skill_dir_set: HashSet::new(),
            all_tags: Vec::new(),
            project_paths,
            focus: Focus::Tags,
            selected_tag: 0,
            selected_skill: 0,
            selected_project: 0,
            project_links: BTreeMap::new(),
            status_msg: String::new(),
            should_quit: false,
            editing_tags: None,
        };
        app.reload()?;
        Ok(app)
    }

    pub(super) fn skill_count(&self) -> usize {
        self.skill_dirs.len()
    }

    fn reload(&mut self) -> Result<()> {
        self.tag_map = yaml::read_skills_yaml_or_empty(&self.paths.skills_yaml)?;
        self.skill_dirs = scanner::scan_skill_dirs(&self.paths.skill_tree_dir)?;
        self.skill_dir_set = self.skill_dirs.iter().cloned().collect();

        let mut tags = BTreeSet::new();
        for t in self.tag_map.values().flatten() {
            tags.insert(t.clone());
        }
        self.all_tags = tags.into_iter().collect();

        self.reload_all_project_links();
        Ok(())
    }

    fn reload_project_links(&mut self, project: &str) {
        let linked = scanner::scan_linked_skills(std::path::Path::new(project), Tool::Claude);
        self.project_links
            .insert(project.to_string(), linked.into_iter().collect());
    }

    fn reload_all_project_links(&mut self) {
        self.project_links.clear();
        for p in &self.project_paths {
            let linked = scanner::scan_linked_skills(std::path::Path::new(p), Tool::Claude);
            self.project_links
                .insert(p.clone(), linked.into_iter().collect());
        }
    }

    pub(super) fn filtered_skills(&self) -> Vec<&str> {
        if self.selected_tag == 0 {
            return self.skill_dirs.iter().map(|s| s.as_str()).collect();
        }

        let Some(tag) = self.all_tags.get(self.selected_tag - 1) else {
            return Vec::new();
        };

        self.tag_map
            .iter()
            .filter(|(name, tags)| tags.contains(tag) && self.skill_dir_set.contains(name.as_str()))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    pub(super) fn tags_for_skill(&self, skill: &str) -> &[String] {
        self.tag_map.get(skill).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub(super) fn current_project(&self) -> Option<&str> {
        self.project_paths
            .get(self.selected_project)
            .map(|s| s.as_str())
    }

    pub(super) fn project_name(path: &str) -> &str {
        path.rsplit('/').next().unwrap_or(path)
    }

    pub(super) fn is_skill_linked(&self, project: &str, skill: &str) -> bool {
        self.project_links
            .get(project)
            .is_some_and(|set| set.contains(skill))
    }

    fn clamp_selections(&mut self) {
        let tag_max = self.all_tags.len();
        if self.selected_tag > tag_max {
            self.selected_tag = tag_max;
        }
        let skill_count = self.filtered_skills().len();
        if skill_count > 0 && self.selected_skill >= skill_count {
            self.selected_skill = skill_count - 1;
        }
        if !self.project_paths.is_empty() && self.selected_project >= self.project_paths.len() {
            self.selected_project = self.project_paths.len() - 1;
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        // Tag editing mode intercepts all keys
        if self.editing_tags.is_some() {
            return self.handle_edit_tags_key(code);
        }

        // Clear status message on any keypress
        self.status_msg.clear();

        match code {
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Tags => Focus::Skills,
                    Focus::Skills => Focus::Projects,
                    Focus::Projects => Focus::Tags,
                };
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Focus::Tags => Focus::Projects,
                    Focus::Skills => Focus::Tags,
                    Focus::Projects => Focus::Skills,
                };
            }
            _ => match self.focus {
                Focus::Tags => self.handle_tags_key(code),
                Focus::Skills => self.handle_skills_key(code)?,
                Focus::Projects => self.handle_projects_key(code)?,
            },
        }
        self.clamp_selections();
        Ok(())
    }

    fn handle_tags_key(&mut self, code: KeyCode) {
        let max = self.all_tags.len();
        match code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.selected_tag = self.selected_tag.saturating_sub(1);
                self.selected_skill = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.selected_tag < max {
                    self.selected_tag += 1;
                }
                self.selected_skill = 0;
            }
            _ => {}
        }
    }

    fn handle_skills_key(&mut self, code: KeyCode) -> Result<()> {
        let skills = self.filtered_skills();
        let len = skills.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_skill = self.selected_skill.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_skill + 1 < len {
                    self.selected_skill += 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                if let (Some(&skill), Some(project)) =
                    (skills.get(self.selected_skill), self.current_project())
                {
                    let skill = skill.to_string();
                    let project = project.to_string();
                    if !self.is_skill_linked(&project, &skill) {
                        linker::link_skill(
                            &self.paths,
                            std::path::Path::new(&project),
                            &skill,
                            Tool::Claude,
                        )?;
                        self.reload_project_links(&project);
                        self.status_msg =
                            format!("Linked {} → {}", skill, Self::project_name(&project));
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Delete | KeyCode::Backspace => {
                if let (Some(&skill), Some(project)) =
                    (skills.get(self.selected_skill), self.current_project())
                {
                    let skill = skill.to_string();
                    let project = project.to_string();
                    if self.is_skill_linked(&project, &skill) {
                        linker::unlink_skill(std::path::Path::new(&project), &skill, Tool::Claude)?;
                        self.reload_project_links(&project);
                        self.status_msg =
                            format!("Unlinked {} ← {}", skill, Self::project_name(&project));
                    }
                }
            }
            KeyCode::Char('t') => {
                if let Some(&skill) = skills.get(self.selected_skill) {
                    let current_tags = self.tags_for_skill(skill).join(", ");
                    let cursor = current_tags.len();
                    self.editing_tags = Some(TagEditState {
                        skill_name: skill.to_string(),
                        input: current_tags,
                        cursor,
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_tags_key(&mut self, code: KeyCode) -> Result<()> {
        let state = self.editing_tags.as_mut().unwrap();
        match code {
            KeyCode::Enter => {
                let skill = state.skill_name.clone();
                let tags: Vec<String> = state
                    .input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.editing_tags = None;
                tagger::set_tags(&self.paths, &skill, &tags)?;
                self.reload()?;
                self.status_msg = format!("{}: [{}]", skill, tags.join(", "));
            }
            KeyCode::Esc => {
                self.editing_tags = None;
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

    fn handle_projects_key(&mut self, code: KeyCode) -> Result<()> {
        let len = self.project_paths.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_project = self.selected_project.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_project + 1 < len {
                    self.selected_project += 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                if let Some(project) = self.current_project() {
                    let project = project.to_string();
                    if let Some(tag) = self
                        .all_tags
                        .get(self.selected_tag.wrapping_sub(1))
                        .cloned()
                    {
                        linker::link_by_tags(
                            &self.paths,
                            std::path::Path::new(&project),
                            std::slice::from_ref(&tag),
                            Tool::Claude,
                        )?;
                        self.reload_project_links(&project);
                        self.status_msg =
                            format!("Linked [{}] → {}", tag, Self::project_name(&project));
                    }
                }
            }
            KeyCode::Char('u') => {
                if let Some(project) = self.current_project() {
                    let project = project.to_string();
                    linker::unlink_all(std::path::Path::new(&project), Tool::Claude)?;
                    self.reload_project_links(&project);
                    self.status_msg = format!("Unlinked all ← {}", Self::project_name(&project));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// RAII guard that restores terminal state on drop.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub fn run(paths: Paths, project_paths: Vec<String>) -> Result<()> {
    let _guard = TerminalGuard::new()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(paths, project_paths)?;

    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                app.handle_key(key.code, key.modifiers)?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
