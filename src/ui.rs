//! Terminal UI module using Ratatui
//!
//! Handles:
//! - Rendering the task list
//! - Vim-like keybindings (hjkl, G/gg, space, /)
//! - Beautiful terminal interface
//! - Responsive user interactions

use crate::state::AppState;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct UI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    list_state: ListState,
}

impl UI {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Ok(Self {
            terminal,
            list_state,
        })
    }

    /// Main UI loop - handles events and rendering
    pub fn run(&mut self, app_state: &mut AppState) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Update list state to match app state
            self.list_state.select(Some(app_state.selected_index));

            // Render
            self.terminal.draw(|f| {
                Self::render_ui(&mut self.list_state, f, app_state);
            })?;

            // Handle input
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => app_state.move_down(),
                        KeyCode::Char('k') | KeyCode::Up => app_state.move_up(),
                        KeyCode::Char('G') => app_state.go_to_bottom(),
                        KeyCode::Char('g') => {
                            // Handle gg - go to top (simplified for now)
                            app_state.go_to_top();
                        }
                        KeyCode::Char(' ') => app_state.toggle_selected_task(),
                        KeyCode::Char('/') => {
                            app_state.start_search();
                            // TODO: Implement search input mode
                        }
                        KeyCode::Esc => {
                            if app_state.is_searching {
                                app_state.end_search();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Cleanup
        self.cleanup()?;
        Ok(())
    }

    fn render_ui(list_state: &mut ListState, f: &mut Frame, app_state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.size());

        // Render task list
        Self::render_tasks(list_state, f, chunks[0], app_state);

        // Render status bar
        Self::render_status_bar(f, chunks[1], app_state);
    }

    fn render_tasks(
        list_state: &mut ListState,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        app_state: &AppState,
    ) {
        let tasks = app_state.get_filtered_tasks();

        let items: Vec<ListItem> = tasks
            .iter()
            .map(|task| {
                let status_symbol = if task.is_completed { "✓" } else { " " };
                let content = format!("[{}] {}", status_symbol, task.content);

                let style = if task.is_completed {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Today's Tasks"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, list_state);
    }

    fn render_status_bar(f: &mut Frame, area: ratatui::layout::Rect, app_state: &AppState) {
        let status_text = match &app_state.sync_status {
            crate::state::SyncStatus::Online => "Online",
            crate::state::SyncStatus::Offline => "Offline",
            crate::state::SyncStatus::Syncing => "Syncing...",
            crate::state::SyncStatus::Error(e) => e,
        };

        let search_text = if app_state.is_searching {
            format!(" | Search: {}", app_state.search_query)
        } else {
            String::new()
        };

        let content = format!(
            "Status: {}{} | Tasks: {} | q:quit j/k:move space:toggle /:search",
            status_text,
            search_text,
            app_state.tasks.len()
        );

        let paragraph = Paragraph::new(content).block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
