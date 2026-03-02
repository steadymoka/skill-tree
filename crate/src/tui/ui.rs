use ratatui::prelude::*;
use ratatui::widgets::*;

use super::app::{App, Focus};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(frame, outer[0], app);
    render_body(frame, outer[1], app);
    render_status(frame, outer[2], app);
}

fn panel_block(title: &str, focused: bool) -> Block<'_> {
    let style = if focused {
        Style::new().fg(Color::Blue)
    } else {
        Style::new().fg(Color::DarkGray)
    };
    Block::bordered()
        .title(title)
        .border_style(style)
        .border_type(BorderType::Rounded)
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        " Skill Tree  ·  {} skills  ·  {} tags ",
        app.skill_count(),
        app.all_tags.len()
    );

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded);

    let tag_focused = app.focus == Focus::Tags;
    let unselected_style = if tag_focused {
        Style::new().fg(Color::Gray)
    } else {
        Style::new().fg(Color::DarkGray)
    };
    let selected_style = Style::new().bg(Color::Blue).fg(Color::White).bold();

    let mut spans = vec![Span::styled(
        " All ",
        if app.selected_tag == 0 {
            selected_style
        } else {
            unselected_style
        },
    )];

    for (i, tag) in app.all_tags.iter().enumerate() {
        spans.push(Span::raw(" "));
        let style = if app.selected_tag == i + 1 {
            selected_style
        } else {
            unselected_style
        };
        spans.push(Span::styled(format!(" {} ", tag), style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area);

    render_skills(frame, chunks[0], app);
    render_projects(frame, chunks[1], app);
}

fn render_skills(frame: &mut Frame, area: Rect, app: &App) {
    let editing = app.editing_tags.is_some();
    let skills = app.filtered_skills();
    let focused = app.focus == Focus::Skills;
    let current_project = app.current_project();

    let items: Vec<ListItem> = skills
        .iter()
        .enumerate()
        .map(|(i, &skill)| {
            let linked = current_project.is_some_and(|p| app.is_skill_linked(p, skill));
            let is_selected = i == app.selected_skill && focused;

            // Check if this skill is being edited
            let is_editing = app
                .editing_tags
                .as_ref()
                .is_some_and(|e| e.skill_name == skill);

            if is_editing {
                let state = app.editing_tags.as_ref().unwrap();
                return ListItem::new(Line::from(vec![
                    Span::styled(" ▸ ", Style::new().fg(Color::Yellow)),
                    Span::styled(skill, Style::new().fg(Color::Yellow)),
                    Span::styled("  ", Style::new()),
                    Span::styled(
                        &state.input,
                        Style::new().bg(Color::DarkGray).fg(Color::White),
                    ),
                    Span::styled("▎", Style::new().fg(Color::Yellow)),
                ]));
            }

            let marker = if linked { "✓" } else { " " };
            let tags = app.tags_for_skill(skill);
            let tag_str = if tags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", tags.join(", "))
            };

            let base = if is_selected {
                Style::new().bg(Color::Blue).fg(Color::White)
            } else if linked {
                Style::new().fg(Color::Green)
            } else {
                Style::new()
            };

            let tag_style = if is_selected {
                Style::new().bg(Color::Blue).fg(Color::Cyan)
            } else {
                Style::new().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", marker),
                    if linked && !is_selected {
                        Style::new().fg(Color::Green)
                    } else {
                        base
                    },
                ),
                Span::styled(skill, base),
                Span::styled(tag_str, tag_style),
            ]))
        })
        .collect();

    let title = if editing {
        " Skills (editing tags) "
    } else {
        " Skills "
    };
    frame.render_widget(
        List::new(items).block(panel_block(title, focused || editing)),
        area,
    );
}

fn render_projects(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Projects;

    let items: Vec<ListItem> = app
        .project_paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let name = App::project_name(path);
            let linked_count = app.project_links.get(path).map_or(0, |s| s.len());
            let is_selected = i == app.selected_project;

            let style = if is_selected && focused {
                Style::new().bg(Color::Blue).fg(Color::White)
            } else if is_selected {
                Style::new().fg(Color::White).bold()
            } else {
                Style::new()
            };

            let count_style = if is_selected && focused {
                Style::new().bg(Color::Blue).fg(Color::Cyan)
            } else if linked_count > 0 {
                Style::new().fg(Color::Green)
            } else {
                Style::new().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}", name), style),
                Span::styled(format!("  {} linked", linked_count), count_style),
            ]))
        })
        .collect();

    frame.render_widget(
        List::new(items).block(panel_block(" Projects ", focused)),
        area,
    );
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let help = if app.editing_tags.is_some() {
        " Enter:save  Esc:cancel"
    } else {
        " Tab:focus  ←→:tag  ↑↓:select  l:link  u:unlink  t:tags  q:quit"
    };

    let line = if app.status_msg.is_empty() {
        Line::from(Span::styled(help, Style::new().fg(Color::DarkGray)))
    } else {
        Line::from(vec![
            Span::styled(&app.status_msg, Style::new().fg(Color::Yellow)),
            Span::styled("  │  ", Style::new().fg(Color::DarkGray)),
            Span::styled(help, Style::new().fg(Color::DarkGray)),
        ])
    };

    frame.render_widget(Paragraph::new(line), area);
}
