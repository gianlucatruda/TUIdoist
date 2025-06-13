//! Application state management
//!
//! Handles:
//! - Current task list and selection
//! - Pending changes (for 30-second cache before sync)
//! - Offline mode and sync status
//! - Undo functionality

use crate::api::Task;
use chrono::{DateTime, Local, NaiveDate};

#[derive(Clone)]
pub struct AppState {
    pub tasks: Vec<Task>,
    pub completed_tasks: Vec<Task>,
    pub selected_index: usize,
    pub search_query: String,
    pub is_searching: bool,
    pub sync_status: SyncStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SyncStatus {
    Online,
    Offline,
    Syncing,
    Error(String),
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            completed_tasks: Vec::new(),
            selected_index: 0,
            search_query: String::new(),
            is_searching: false,
            sync_status: SyncStatus::Offline,
        }
    }

    /// Load tasks into the application state
    pub fn load_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        self.selected_index = 0;
    }

    /// Load completed tasks into the application state
    pub fn load_completed_tasks(&mut self, tasks: Vec<Task>) {
        // Force each completed task's is_completed flag to true.
        self.completed_tasks = tasks
            .into_iter()
            .map(|mut t| {
                t.is_completed = true;
                t
            })
            .collect();
    }

    /// Returns the number of tasks in the unified today view.
    pub fn unified_today_count(&self) -> usize {
        self.today_tasks().len()
    }

    /// Toggle a task by its ID
    pub fn toggle_task_by_id(&mut self, selected_id: &str) {
        // Toggle in the active tasks list if found.
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == selected_id) {
            task.is_completed = !task.is_completed;
        } else if let Some(task) = self
            .completed_tasks
            .iter_mut()
            .find(|t| t.id == selected_id)
        {
            task.is_completed = !task.is_completed;
        }
    }

    /// Move selection up within the unified today list.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down within the unified today list.
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.unified_today_count() {
            self.selected_index += 1;
        }
    }

    /// Set selection to the top (index 0) of the unified today list.
    pub fn go_to_top(&mut self) {
        self.selected_index = 0;
    }

    /// Set selection to the bottom of the unified today list.
    pub fn go_to_bottom(&mut self) {
        let count = self.unified_today_count();
        if count > 0 {
            self.selected_index = count - 1;
        }
    }

    /// Returns tasks whose due date equals today.
    pub fn tasks_due_today(&self) -> Vec<&Task> {
        let today = Local::now().naive_local().date();
        self.tasks
            .iter()
            .filter(|task| {
                if let Some(due) = &task.due {
                    // If due.date is in "YYYY-MM-DD" format:
                    if due.date.len() == 10 {
                        if let Ok(d) = NaiveDate::parse_from_str(&due.date, "%Y-%m-%d") {
                            return d == today;
                        }
                    } else if due.date.contains('T') {
                        // Try to parse as a date-time from RFC3339.
                        if let Ok(dt) = DateTime::parse_from_rfc3339(&due.date) {
                            return dt.with_timezone(&Local).date_naive() == today;
                        }
                    }
                }
                false
            })
            .collect()
    }

    /// Returns tasks that are NOT due today.
    /// Tasks with no due date are considered upcoming.
    pub fn tasks_upcoming(&self) -> Vec<&Task> {
        let today = Local::now().naive_local().date();
        self.tasks
            .iter()
            .filter(|task| {
                if let Some(due) = &task.due {
                    if due.date.len() == 10 {
                        if let Ok(d) = NaiveDate::parse_from_str(&due.date, "%Y-%m-%d") {
                            return d > today;
                        }
                    } else if due.date.contains('T') {
                        if let Ok(dt) = DateTime::parse_from_rfc3339(&due.date) {
                            return dt.with_timezone(&Local).date_naive() > today;
                        }
                    }
                    // If parsing fails, do not display in "today"
                    false
                } else {
                    // No due date → upcoming
                    true
                }
            })
            .collect()
    }

    pub fn today_tasks(&self) -> Vec<&Task> {
        let mut combined = self.tasks_due_today();
        combined.extend(self.completed_tasks.iter());
        // Optionally sort so that active tasks appear first
        combined.sort_by_key(|task| task.is_completed);
        combined
    }
}
