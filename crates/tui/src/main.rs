use anyhow::Result;
use bl1nk_core::agents::register::AgentRegistry;
use bl1nk_core::agents::types::AgentOverrides;
use bl1nk_core::agents::utils::create_builtin_agents;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::io;

mod event;
mod models;
mod ui;

use event::{Event, EventHandler};
use models::Agent;
use ui::detail::draw_agent_detail;
use ui::help::draw_help_popup;
use ui::list::{draw_agent_list, AgentList};
use ui::popup::draw_command_popup; // UI Model

#[tokio::main]
async fn main() -> Result<()> {
    // Load ALL Agents via Core Logic
    // We strictly use bl1nk-core to discover and configure agents.
    // The Core logic scans the provided directory (".") for the 'agents/' folder and 'agents.json'.
    let disabled_agents = vec![];
    let agent_overrides = AgentOverrides::default();

    let agents_map = create_builtin_agents(
        &disabled_agents,
        &agent_overrides,
        Some("."), // Core will look for ./agents/agents.json here
        Some("gemini-2.0-flash-exp"),
        None,
        None,
    )
    .await?;

    // Convert HashMap to Vec for Registry
    let core_agents: Vec<bl1nk_core::agents::types::AgentConfig> =
        agents_map.values().cloned().collect();
    let registry = AgentRegistry::new(core_agents.clone());

    // Convert to UI Agents
    let mut ui_agents: Vec<Agent> = core_agents
        .into_iter()
        .map(|c| {
            // Map Core fields to UI fields.
            // Note: Core AgentConfig might miss some metadata like 'personality' or 'category' string.
            // We use defaults or infer where possible.
            Agent {
                id: c.id.clone(),
                name: c.name.clone(),
                file: c.extension_name.unwrap_or_default(),
                category: "Available".to_string(), // AgentConfig doesn't expose category enum easily here
                description: c.description.unwrap_or_default(),
                use_cases: c.skills.unwrap_or_default(), // skills field often used for capabilities/use-cases
                tags: c.capabilities,
                complexity: "Unknown".to_string(),
                outputs: vec![],
                personality: "Standard".to_string(),
            }
        })
        .collect();

    // Sort by name for better UX
    ui_agents.sort_by(|a, b| a.name.cmp(&b.name));

    let mut agent_list = AgentList::new(ui_agents);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize event handler
    let events = EventHandler::new(250);

    // Run app
    let res = run_app(&mut terminal, events, &mut agent_list, &registry).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

struct AppState {
    show_popup: bool,
    show_help: bool,
    popup_command: String,
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    events: EventHandler,
    agent_list: &mut AgentList,
    _registry: &AgentRegistry,
) -> Result<()> {
    let mut app_state = AppState {
        show_popup: false,
        show_help: false,
        popup_command: String::new(),
    };

    loop {
        terminal.draw(|f| {
            // Main layout - Header, Main Content, Footer
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Main content
                    Constraint::Length(3), // Footer
                ])
                .split(f.area());

            // Draw header
            let header_block = Block::default()
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(30, 41, 59))); // #1E293B (secondary)

            let header_text = vec![
                Line::from(Span::styled(
                    "Bl1nk Agents Manager",
                    Style::default()
                        .fg(Color::Rgb(34, 197, 94)) // #22C55E (CTA)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Navigate with ↑ ↓, Select with Enter, Quit with Q",
                    Style::default().fg(Color::Rgb(148, 163, 184)), // Gray-400
                )),
            ];

            let header_paragraph = Paragraph::new(header_text)
                .block(header_block)
                .style(Style::default().bg(Color::Rgb(2, 6, 23))); // #020617 (background)

            f.render_widget(header_paragraph, main_chunks[0]);

            // Main content layout - Agent list and details
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(main_chunks[1]);

            draw_agent_list(f, agent_list, content_chunks[0]);
            draw_agent_detail(f, agent_list.selected_agent(), content_chunks[1]);

            // Draw footer
            let footer_block = Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(30, 41, 59))); // #1E293B (secondary)

            let footer_text = vec![Line::from(
                Span::raw("F1: Help ").fg(Color::Rgb(148, 163, 184)), // Gray-400
            )
            .alignment(ratatui::layout::Alignment::Right)];

            let footer_paragraph = Paragraph::new(footer_text)
                .block(footer_block)
                .style(Style::default().bg(Color::Rgb(2, 6, 23))); // #020617 (background)

            f.render_widget(footer_paragraph, main_chunks[2]);

            if app_state.show_popup && !app_state.show_help {
                draw_command_popup(f, &app_state.popup_command, f.area());
            } else if app_state.show_help {
                draw_help_popup(f, f.area());
            }
        })?;

        match events.next()? {
            Event::Input(key) => {
                if app_state.show_help {
                    // Any key closes the help popup
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::F(1) => {
                            app_state.show_help = false;
                        }
                        _ => {
                            app_state.show_help = false;
                        }
                    }
                } else if app_state.show_popup {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                            app_state.show_popup = false;
                        }
                        KeyCode::F(1) => {
                            // Switch to help popup
                            app_state.show_popup = false;
                            app_state.show_help = true;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(())
                        }
                        KeyCode::Down => agent_list.next(),
                        KeyCode::Up => agent_list.previous(),
                        KeyCode::Enter => {
                            if let Some(agent) = agent_list.selected_agent() {
                                let command = if !agent.file.is_empty() {
                                    format!("export GEMINI_SYSTEM_MD=~/.gemini/extensions/agents-manager/agents/{}", agent.file)
                                } else {
                                    format!(
                                        "Built-in agent '{}' selected. (Internal Core Agent)",
                                        agent.name
                                    )
                                };

                                app_state.popup_command = command;
                                app_state.show_popup = true;
                            }
                        }
                        KeyCode::F(1) => {
                            // Show help popup
                            app_state.show_help = true;
                        }
                        _ => {}
                    }
                }
            }
            Event::Tick => {}
        }
    }
}
