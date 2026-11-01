use crate::models::Agent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame,
};

pub struct AgentList {
    pub state: ListState,
    pub agents: Vec<Agent>,
}

impl AgentList {
    pub fn new(agents: Vec<Agent>) -> Self {
        let mut state = ListState::default();
        if !agents.is_empty() {
            state.select(Some(0));
        }
        Self { state, agents }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.agents.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.agents.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected_agent(&self) -> Option<&Agent> {
        self.state.selected().map(|i| &self.agents[i])
    }
}

pub fn draw_agent_list(f: &mut Frame, app: &mut AgentList, area: Rect) {
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .map(|agent| {
            // Create a line with the agent name and category
            let agent_line = Line::from(vec![
                Span::styled(
                    format!("● {}", agent.name),
                    Style::default()
                        .fg(
                            if app.agents.iter().position(|a| a.id == agent.id)
                                == app.state.selected()
                            {
                                Color::Rgb(34, 197, 94) // #22C55E (CTA) - selected
                            } else {
                                Color::Rgb(248, 250, 252) // #F8FAFC (text) - normal
                            },
                        )
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", agent.category),
                    Style::default()
                        .fg(Color::Rgb(148, 163, 184)) // Gray-400
                        .add_modifier(Modifier::DIM),
                ),
            ]);

            ListItem::new(agent_line).style(Style::default().bg(
                if app.agents.iter().position(|a| a.id == agent.id) == app.state.selected() {
                    Color::Rgb(15, 23, 42) // #0F172A (primary) - selected
                } else {
                    Color::Rgb(2, 6, 23) // #020617 (background) - normal
                },
            ))
        })
        .collect();

    let items = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(30, 41, 59))) // #1E293B (secondary)
                .title("Agents")
                .title_style(Style::default().fg(Color::Rgb(34, 197, 94))), // #22C55E (CTA)
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(15, 23, 42)) // #0F172A (primary)
                .fg(Color::Rgb(34, 197, 94)) // #22C55E (CTA)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    f.render_stateful_widget(items, area, &mut app.state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Agent;

    fn mock_agent(id: &str) -> Agent {
        Agent {
            id: id.to_string(),
            name: "Test".to_string(),
            file: "test.md".to_string(),
            category: "test".to_string(),
            description: "test".to_string(),
            use_cases: vec![],
            tags: vec![],
            complexity: "low".to_string(),
            outputs: vec![],
            personality: "test".to_string(),
        }
    }

    #[test]
    fn test_navigation() {
        let agents = vec![mock_agent("1"), mock_agent("2"), mock_agent("3")];
        let mut list = AgentList::new(agents);

        assert_eq!(list.state.selected(), Some(0));

        list.next();
        assert_eq!(list.state.selected(), Some(1));

        list.next();
        assert_eq!(list.state.selected(), Some(2));

        list.next();
        assert_eq!(list.state.selected(), Some(0)); // Loop back

        list.previous();
        assert_eq!(list.state.selected(), Some(2)); // Loop back
    }
}
