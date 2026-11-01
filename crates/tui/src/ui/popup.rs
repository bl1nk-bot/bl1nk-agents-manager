use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_command_popup(f: &mut Frame, command: &str, area: Rect) {
    let area = centered_rect(60, 25, area);
    f.render_widget(Clear, area); // Clear the area underneath

    let block = Block::default()
        .title("Switch Agent Command")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(30, 41, 59))) // #1E293B (secondary)
        .title_style(Style::default().fg(Color::Rgb(34, 197, 94))) // #22C55E (CTA)
        .style(Style::default().bg(Color::Rgb(15, 23, 42))); // #0F172A (primary)

    let text = vec![
        Line::from("Run this command in your terminal to switch:").fg(Color::Rgb(248, 250, 252)), // #F8FAFC (text)
        Line::from(""),
        Line::from(Span::raw("Command: ").fg(Color::Rgb(255, 206, 84))), // Yellow-300
        Line::from(Span::styled(
            command,
            Style::default()
                .fg(Color::Rgb(34, 197, 94)) // #22C55E (CTA)
                .bg(Color::Rgb(2, 6, 23)), // #020617 (background)
        )),
        Line::from(""),
        Line::from("Press 'Esc' or 'Enter' to close.").fg(Color::Rgb(148, 163, 184)), // Gray-400
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Rgb(15, 23, 42))); // #0F172A (primary)

    f.render_widget(paragraph, area);
}
