use crate::models::Agent;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_agent_detail(f: &mut Frame, agent: Option<&Agent>, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(30, 41, 59))) // #1E293B (secondary)
        .title("Agent Details")
        .title_style(Style::default().fg(Color::Rgb(34, 197, 94))); // #22C55E (CTA)

    let text = match agent {
        Some(a) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Rgb(255, 206, 84))), // Yellow-300
                    Span::raw(&a.name).fg(Color::Rgb(248, 250, 252)), // #F8FAFC (text)
                ]),
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::Rgb(255, 206, 84))),
                    Span::raw(&a.id).fg(Color::Rgb(248, 250, 252)),
                ]),
                Line::from(vec![
                    Span::styled("Category: ", Style::default().fg(Color::Rgb(255, 206, 84))),
                    Span::raw(a.display_category()).fg(Color::Rgb(248, 250, 252)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Complexity: ",
                        Style::default().fg(Color::Rgb(255, 206, 84)),
                    ),
                    Span::raw(a.display_complexity()).fg(Color::Rgb(248, 250, 252)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "Personality: ",
                        Style::default().fg(Color::Rgb(255, 206, 84)),
                    ),
                    Span::raw(&a.personality).fg(Color::Rgb(248, 250, 252)),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Description:",
                    Style::default().fg(Color::Rgb(255, 206, 84)),
                )]),
                Line::from(a.description.clone()).fg(Color::Rgb(248, 250, 252)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("File: ", Style::default().fg(Color::Rgb(255, 206, 84))),
                    Span::raw(&a.file).fg(Color::Rgb(248, 250, 252)),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Use Cases:",
                    Style::default().fg(Color::Rgb(255, 206, 84)),
                )]),
            ];

            // Add use cases
            for use_case in &a.use_cases {
                lines.push(Line::from(format!("- {}", use_case)).fg(Color::Rgb(248, 250, 252)));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Tags:",
                Style::default().fg(Color::Rgb(255, 206, 84)),
            )]));

            // Add tags as inline elements
            if !a.tags.is_empty() {
                let mut tag_spans = vec![Span::raw("")];
                for (i, tag) in a.tags.iter().enumerate() {
                    if i > 0 {
                        tag_spans.push(Span::raw(" "));
                    }
                    tag_spans.push(
                        Span::raw(format!("[{}]", tag))
                            .bg(Color::Rgb(15, 23, 42)) // #0F172A (primary)
                            .fg(Color::Rgb(34, 197, 94)) // #22C55E (CTA)
                            .bold(),
                    );
                }
                lines.push(Line::from(tag_spans));
            } else {
                lines.push(Line::from("No tags").fg(Color::Rgb(148, 163, 184)));
                // Gray-400
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Outputs:",
                Style::default().fg(Color::Rgb(255, 206, 84)),
            )]));

            // Add outputs
            if !a.outputs.is_empty() {
                for output in &a.outputs {
                    lines.push(Line::from(format!("- {}", output)).fg(Color::Rgb(248, 250, 252)));
                }
            } else {
                lines.push(Line::from("No specific outputs defined").fg(Color::Rgb(148, 163, 184))); // Gray-400
            }

            lines
        }
        None => vec![Line::from("Select an agent to view details.").fg(Color::Rgb(148, 163, 184))], // Gray-400
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Rgb(2, 6, 23))); // #020617 (background)

    f.render_widget(paragraph, area);
}
