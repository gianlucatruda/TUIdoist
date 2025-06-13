//! Todoist API client module
//!
//! Handles communication with the Todoist REST API, including:
//! - Authentication
//! - Fetching tasks
//! - Updating task completion status
//! - Offline caching and sync logic

use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
struct TasksResponse {
    results: Vec<Task>,
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    pub content: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
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
        let url = format!("{}/tasks", self.base_url);

        // Log the URL and query parameters
        log::debug!("Sending GET request to {} with query filter=today", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&[("filter", "today")])
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

        let tasks_resp: TasksResponse = response.json().await?;
        log::debug!("Retrieved {} tasks", tasks_resp.results.len());
        Ok(tasks_resp.results)
    }

    /// Fetch today's completed tasks from the Todoist API
    pub async fn get_todays_completed_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        // Use the completed-by-completion-date endpoint.
        let url = format!("{}/tasks/completed/by_completion_date", self.base_url);
        let today = Local::today();
        let start = today.and_hms(0, 0, 0);
        let end = today.succ().and_hms(0, 0, 0); // tomorrow 00:00:00
        let since = start.to_rfc3339();
        let until = end.to_rfc3339();

        log::debug!("Fetching completed tasks from {} to {}", since, until);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&[("since", since.as_str()), ("until", until.as_str())])
            .send()
            .await?;

        let status = response.status();
        log::debug!("Completed tasks response status: {}", status);
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!(
                "Error fetching completed tasks: {} - {}",
                status, error_text
            )
            .into());
        }

        #[derive(Debug, Deserialize)]
        struct CompletedTasksResponse {
            items: Vec<Task>,
            next_cursor: Option<String>,
        }
        let comp_resp: CompletedTasksResponse = response.json().await?;
        log::debug!("Fetched {} completed tasks", comp_resp.items.len());
        Ok(comp_resp.items)
    }
}
