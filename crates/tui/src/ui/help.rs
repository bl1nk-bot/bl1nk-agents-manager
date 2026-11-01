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

pub fn draw_help_popup(f: &mut Frame, area: Rect) {
    let area = centered_rect(70, 60, area);
    f.render_widget(Clear, area); // Clear the area underneath

    let block = Block::default()
        .title("Help - Bl1nk Agents Manager")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(30, 41, 59))) // #1E293B (secondary)
        .title_style(Style::default().fg(Color::Rgb(34, 197, 94))) // #22C55E (CTA)
        .style(Style::default().bg(Color::Rgb(15, 23, 42))); // #0F172A (primary)

    let text = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Rgb(255, 206, 84))
                .add_modifier(ratatui::style::Modifier::BOLD),
        )), // Yellow-300
        Line::from(""),
        Line::from(Span::raw("  ↑/↓ Arrow Keys").fg(Color::Rgb(34, 197, 94)))
            .bg(Color::Rgb(2, 6, 23)), // #22C55E (CTA)
        Line::from(Span::raw("     Navigate between agents").fg(Color::Rgb(248, 250, 252))), // #F8FAFC (text)
        Line::from(""),
        Line::from(Span::raw("  Enter").fg(Color::Rgb(34, 197, 94))).bg(Color::Rgb(2, 6, 23)), // #22C55E (CTA)
        Line::from(
            Span::raw("         Select an agent and show command").fg(Color::Rgb(248, 250, 252)),
        ), // #F8FAFC (text)
        Line::from(""),
        Line::from(Span::raw("  F1").fg(Color::Rgb(34, 197, 94))).bg(Color::Rgb(2, 6, 23)), // #22C55E (CTA)
        Line::from(Span::raw("          Show this help screen").fg(Color::Rgb(248, 250, 252))), // #F8FAFC (text)
        Line::from(""),
        Line::from(Span::raw("  Q or Esc").fg(Color::Rgb(34, 197, 94))).bg(Color::Rgb(2, 6, 23)), // #22C55E (CTA)
        Line::from(Span::raw("      Quit the application").fg(Color::Rgb(248, 250, 252))), // #F8FAFC (text)
        Line::from(""),
        Line::from(Span::styled(
            "About",
            Style::default()
                .fg(Color::Rgb(255, 206, 84))
                .add_modifier(ratatui::style::Modifier::BOLD),
        )), // Yellow-300
        Line::from(""),
        Line::from(
            Span::raw("Bl1nk Agents Manager helps you discover and use specialized AI agents")
                .fg(Color::Rgb(248, 250, 252)),
        ), // #F8FAFC (text)
        Line::from(
            Span::raw("for different tasks. Each agent has unique capabilities and expertise.")
                .fg(Color::Rgb(248, 250, 252)),
        ), // #F8FAFC (text)
        Line::from(""),
        Line::from(Span::raw("Press any key to close this help.").fg(Color::Rgb(148, 163, 184))), // Gray-400
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Rgb(15, 23, 42))); // #0F172A (primary)

    f.render_widget(paragraph, area);
}
