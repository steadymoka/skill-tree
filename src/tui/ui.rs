use ratatui::prelude::*;
use ratatui::widgets::*;

use super::app::{App, Screen};
use super::ui_projects;
use super::ui_skills;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .split(area);

    render_tab_bar(frame, outer[0], app);

    if app.screen.is_projects() {
        ui_projects::render(frame, outer[1], app);
    } else {
        ui_skills::render(frame, outer[1], app);
    }

    render_status_bar(frame, outer[2], app);

    if app.text_input.is_some() {
        render_text_input_modal(frame, area, app);
    }
}

fn render_tab_bar(frame: &mut Frame, area: Rect, app: &App) {
    let active = Style::new().fg(Color::White).bg(Color::Blue).bold();
    let inactive = Style::new().fg(Color::DarkGray);
    let tab_style = |s: Screen| if app.screen == s { active } else { inactive };

    let line = Line::from(vec![
        Span::styled(" Skill Tree ", Style::new().fg(Color::White).bold()),
        Span::styled(
            format!(
                "  {} skills  {} tags  ",
                app.skill_count(),
                app.all_tags.len()
            ),
            Style::new().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(" 1:Skills ", tab_style(Screen::Skills)),
        Span::raw(" "),
        Span::styled(" 2:Claude ", tab_style(Screen::Claude)),
        Span::raw(" "),
        Span::styled(" 3:Codex ", tab_style(Screen::Codex)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let help = if app.text_input.is_some() {
        " Enter:save  Esc:cancel"
    } else if app.screen.is_projects() {
        " 1/2/3:screen  \u{2190}\u{2192}:focus  \u{2191}\u{2193}:select  Space:link/unlink  Enter:fold  q:quit"
    } else {
        " 1/2/3:screen  \u{2190}\u{2192}:focus  \u{2191}\u{2193}:select  Space:toggle  a:new tag  q:quit"
    };

    let line = if app.status_msg.is_empty() {
        Line::from(Span::styled(help, Style::new().fg(Color::DarkGray)))
    } else {
        Line::from(vec![
            Span::styled(&app.status_msg, Style::new().fg(Color::Yellow)),
            Span::styled("  \u{2502}  ", Style::new().fg(Color::DarkGray)),
            Span::styled(help, Style::new().fg(Color::DarkGray)),
        ])
    };

    frame.render_widget(Paragraph::new(line), area);
}

fn render_text_input_modal(frame: &mut Frame, area: Rect, app: &App) {
    let state = app.text_input.as_ref().unwrap();

    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 3u16;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let modal_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, modal_area);

    let block = Block::bordered()
        .title(" New Tag ")
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let display = Line::from(vec![
        Span::raw(&state.input),
        Span::styled("\u{2588}", Style::new().fg(Color::Yellow)),
    ]);

    frame.render_widget(Paragraph::new(display), inner);
}

pub(super) fn panel_block(title: &str, focused: bool) -> Block<'_> {
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
