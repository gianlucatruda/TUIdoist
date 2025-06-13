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
use std::time::Duration;

/// Returns a spinner frame using OSC 8. Uses a simple 4-frame spinner.
fn spinner_frame() -> &'static str {
    // Define a simple spinner with 4 frames.
    let frames = ["⠋", "⠙", "⠹", "⠸"];
    // Determine current frame based on system time.
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let index = (millis / 100) as usize % frames.len();
    frames[index]
}

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
    pub async fn run(
        &mut self,
        app_state: std::sync::Arc<tokio::sync::Mutex<crate::state::AppState>>,
        client: std::sync::Arc<crate::api::TodoistClient>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            {
                // Lock state for rendering
                let state = app_state.lock().await;
                self.list_state.select(Some(state.selected_index));
            }

            // Render using a nonblocking try_lock snapshot, so that UI always refreshes
            self.terminal.draw(|f| {
                let state_copy = if let Ok(guard) = app_state.try_lock() {
                    guard.clone()
                } else {
                    AppState::new()
                };
                Self::render_ui(&mut self.list_state, f, &state_copy);
            })?;

            // Handle input with timeout polling
            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('j') | KeyCode::Down => {
                                let mut state = app_state.lock().await;
                                state.move_down();
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                let mut state = app_state.lock().await;
                                state.move_up();
                            }
                            KeyCode::Char('G') => {
                                let mut state = app_state.lock().await;
                                state.go_to_bottom();
                            }
                            KeyCode::Char('g') => {
                                let mut state = app_state.lock().await;
                                state.go_to_top();
                            }
                            KeyCode::Char(' ') => {
                                // First, obtain a snapshot of the unified today view and extract the selected task ID.
                                let selected_id_opt = {
                                    let state = app_state.lock().await;
                                    let unified_ids: Vec<String> = state
                                        .today_tasks()
                                        .into_iter()
                                        .map(|t| t.id.clone())
                                        .collect();
                                    unified_ids.get(state.selected_index).cloned()
                                };

                                // If a task id was found, lock mutably and toggle that task.
                                if let Some(selected_id) = selected_id_opt {
                                    let mut state = app_state.lock().await;
                                    state.toggle_task_by_id(&selected_id);
                                }
                            }
                            KeyCode::Char('r') => {
                                {
                                    // Immediately mark state as syncing
                                    let mut state = app_state.lock().await;
                                    state.sync_status = crate::state::SyncStatus::Syncing;
                                }
                                // Spawn a background task for refresh so UI rendering is not blocked
                                let app_state_clone = app_state.clone();
                                let client_clone = client.clone();
                                tokio::spawn(async move {
                                    use tokio::time::{timeout, Duration};
                                    // Refresh active tasks with timeout
                                    let active_result = timeout(
                                        Duration::from_secs(5),
                                        client_clone.get_todays_tasks(),
                                    )
                                    .await;
                                    // Refresh completed tasks with timeout
                                    let completed_result = timeout(
                                        Duration::from_secs(5),
                                        client_clone.get_todays_completed_tasks(),
                                    )
                                    .await;
                                    let mut state = app_state_clone.lock().await;
                                    match active_result {
                                        Ok(Ok(tasks)) => {
                                            state.load_tasks(tasks);
                                            state.sync_status = crate::state::SyncStatus::Online;
                                        }
                                        Ok(Err(e)) => {
                                            eprintln!("Error refreshing tasks: {}", e);
                                            state.sync_status =
                                                crate::state::SyncStatus::Error(e.to_string());
                                        }
                                        Err(_) => {
                                            eprintln!("Refresh tasks timed out");
                                            state.sync_status = crate::state::SyncStatus::Error(
                                                "Timeout".to_string(),
                                            );
                                        }
                                    }
                                    match completed_result {
                                        Ok(Ok(completed)) => {
                                            state.load_completed_tasks(completed);
                                        }
                                        Ok(Err(e)) => {
                                            eprintln!("Error refreshing completed tasks: {}", e);
                                        }
                                        Err(_) => {
                                            eprintln!("Refresh completed tasks timed out");
                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
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
            crate::state::SyncStatus::Online => "Online".to_string(),
            crate::state::SyncStatus::Offline => "Offline".to_string(),
            crate::state::SyncStatus::Syncing => format!("{} Syncing...", spinner_frame()),
            crate::state::SyncStatus::Error(e) => format!("ERR: {}", e),
        };

        let search_text = if app_state.is_searching {
            format!(" | Search: {}", app_state.search_query)
        } else {
            String::new()
        };

        let content = format!(
            "Status: {}{} | Tasks: {} | q: quit, r: refresh, j/k: move, space: (un)check",
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
