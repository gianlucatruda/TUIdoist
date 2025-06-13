//! Application state management
//!
//! Handles:
//! - Current task list and selection
//! - Pending changes (for 30-second cache before sync)
//! - Offline mode and sync status
//! - Undo functionality

use crate::api::Task;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PendingChange {
    pub task_id: String,
    pub change_type: ChangeType,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub enum ChangeType {
    Complete,
    Uncomplete,
}

pub struct AppState {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub pending_changes: Vec<PendingChange>,
    pub search_query: String,
    pub is_searching: bool,
    pub sync_status: SyncStatus,
}

#[derive(Debug, PartialEq)]
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
            selected_index: 0,
            pending_changes: Vec::new(),
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

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.tasks.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Go to top of list
    pub fn go_to_top(&mut self) {
        self.selected_index = 0;
    }

    /// Go to bottom of list
    pub fn go_to_bottom(&mut self) {
        self.selected_index = self.tasks.len().saturating_sub(1);
    }

    /// Toggle completion status of selected task
    pub fn toggle_selected_task(&mut self) {
        if let Some(task) = self.tasks.get_mut(self.selected_index) {
            task.is_completed = !task.is_completed;

            // Add to pending changes for sync
            let change_type = if task.is_completed {
                ChangeType::Complete
            } else {
                ChangeType::Uncomplete
            };

            self.pending_changes.push(PendingChange {
                task_id: task.id.clone(),
                change_type,
                timestamp: Instant::now(),
            });
        }
    }

    /// Get changes that are ready to sync (older than 30 seconds)
    pub fn get_ready_to_sync(&self) -> Vec<&PendingChange> {
        let threshold = Duration::from_secs(30);
        let now = Instant::now();

        self.pending_changes
            .iter()
            .filter(|change| now.duration_since(change.timestamp) >= threshold)
            .collect()
    }

    /// Remove synced changes from pending list
    pub fn mark_synced(&mut self, task_ids: &[String]) {
        self.pending_changes
            .retain(|change| !task_ids.contains(&change.task_id));
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.is_searching = true;
        self.search_query.clear();
    }

    /// End search mode
    pub fn end_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
    }

    /// Update search query
    pub fn update_search(&mut self, query: String) {
        self.search_query = query;
    }

    /// Get filtered tasks based on search query
    pub fn get_filtered_tasks(&self) -> Vec<&Task> {
        if self.search_query.is_empty() {
            self.tasks.iter().collect()
        } else {
            self.tasks
                .iter()
                .filter(|task| {
                    task.content
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .collect()
        }
    }
}
