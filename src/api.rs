//! Todoist API client module
//! 
//! Handles communication with the Todoist REST API, including:
//! - Authentication
//! - Fetching tasks
//! - Updating task completion status
//! - Offline caching and sync logic

use std::collections::HashMap;
use reqwest;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Local};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub is_completed: bool,
    pub due: Option<Due>,
    pub priority: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Due {
    pub date: String,
    pub is_recurring: bool,
    pub datetime: Option<String>,
    pub string: String,
    pub timezone: Option<String>,
}

pub struct TodoistClient {
    api_token: String,
    base_url: String,
    client: reqwest::Client,
}

impl TodoistClient {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            base_url: "https://api.todoist.com/rest/v2".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Fetch today's tasks from the Todoist API
    pub async fn get_todays_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let url = format!("{}/tasks", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&[("filter", format!("due date: {}", today))])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let tasks: Vec<Task> = response.json().await?;
        Ok(tasks)
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/tasks/{}/close", self.base_url, task_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to complete task: {}", response.status()).into());
        }

        Ok(())
    }

    /// Mark a task as uncompleted
    pub async fn uncomplete_task(&self, task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/tasks/{}/reopen", self.base_url, task_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to uncomplete task: {}", response.status()).into());
        }

        Ok(())
    }
}
