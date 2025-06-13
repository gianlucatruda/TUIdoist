//! Todoist API client module
//!
//! Handles communication with the Todoist REST API, including:
//! - Authentication
//! - Fetching tasks
//! - Updating task completion status
//! - Offline caching and sync logic

use chrono::Local;
use reqwest;
use serde::{Deserialize, Serialize};

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
            base_url: "https://api.todoist.com/api/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Fetch today's tasks from the Todoist API
    pub async fn get_todays_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let url = format!("{}/tasks", self.base_url);

        // Log the URL and query parameters
        log::debug!(
            "Sending GET request to {} with query filter=due date: {}",
            url,
            today
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&[("filter", format!("due date: {}", today))])
            .send()
            .await?;

        // Store status code before consuming the response
        let status = response.status();
        log::debug!("Response HTTP status: {}", status);

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No body".to_string());
            log::error!("Error response body: {}", error_text);
            return Err(format!(
                "API request failed with status: {} - {}",
                status, error_text
            )
            .into());
        }

        let tasks: Vec<Task> = response.json().await?;
        log::debug!("Retrieved {} tasks", tasks.len());
        Ok(tasks)
    }
}
