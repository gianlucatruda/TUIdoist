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

/// Minimal markdown parser: strips common markdown symbols and converts link syntax.
fn parse_markdown(text: &str) -> String {
    // Remove bold & italic markers and underscores.
    let mut cleaned = text.replace("**", "").replace("*", "").replace("_", "");
    // Very basic handling of markdown links: convert `[label](url)` into "label (url)"
    // This naive approach replaces "](" with ") (" and then removes the leading "[".
    cleaned = cleaned.replace("](", ") (");
    if cleaned.starts_with('[') {
        cleaned = cleaned[1..].to_string();
    }
    cleaned
}

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

        // Render two sections for tasks
        let task_area = chunks[0];
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(task_area);

        // Render merged "Today" tasks (active + completed)
        Self::render_tasks_section(
            "Today",
            &app_state.today_tasks(),
            f,
            vertical_chunks[0],
            list_state,
            0,
            app_state.selected_index,
        );

        // Render Upcoming tasks; offset equals the count of today_tasks
        Self::render_tasks_section(
            "Upcoming",
            &app_state.tasks_upcoming(),
            f,
            vertical_chunks[1],
            list_state,
            app_state.today_tasks().len(),
            app_state.selected_index,
        );

        // Render status bar
        Self::render_status_bar(f, chunks[1], app_state);
    }

    fn render_tasks_section(
        title: &str,
        tasks: &[&crate::api::Task],
        f: &mut Frame,
        area: ratatui::layout::Rect,
        list_state: &mut ListState,
        offset: usize,
        global_selected_index: usize,
    ) {
        let items: Vec<ListItem> = tasks
            .iter()
            .map(|task| {
                let status_symbol = if task.is_completed { "✓" } else { " " };

                // Process markdown from both content and description.
                let content_md = parse_markdown(&task.content);
                let desc_md = parse_markdown(&task.description);
                let desc_truncated = if !desc_md.is_empty() {
                    if desc_md.len() > 100 {
                        format!(" - {}...", &desc_md[..100])
                    } else {
                        format!(" - {}", desc_md)
                    }
                } else {
                    String::new()
                };

                let combined = format!("[{}] {}{}", status_symbol, content_md, desc_truncated);
                let style = if task.is_completed {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(combined, style)))
            })
            .collect();

        // Compute local selection index for this section if needed:
        let local_selected =
            if global_selected_index >= offset && global_selected_index < offset + tasks.len() {
                Some(global_selected_index - offset)
            } else {
                None
            };

        // Create a temporary ListState for this section:
        let mut section_state = ListState::default();
        section_state.select(local_selected);

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut section_state);
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
