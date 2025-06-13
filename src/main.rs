mod api;
mod state;
mod ui;

use api::TodoistClient;
use dotenv::dotenv;
use state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;
use ui::UI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from .env (ignore errors)
    env_logger::init();

    // TODO: Get API token from environment or config
    let api_token =
        std::env::var("TODOIST_API_TOKEN").unwrap_or_else(|_| "placeholder_token".to_string());

    let client = TodoistClient::new(api_token);
    let client = Arc::new(client);

    let app_state = AppState::new();
    let app_state = Arc::new(Mutex::new(app_state));

    // Initial fetch of tasks
    {
        let mut state = app_state.lock().await;
        match client.get_todays_tasks().await {
            Ok(tasks) => {
                state.load_tasks(tasks);
                state.sync_status = state::SyncStatus::Online;
            }
            Err(e) => {
                eprintln!("Failed to fetch tasks: {}", e);
                state.sync_status = state::SyncStatus::Error(e.to_string());
            }
        }
        match client.get_todays_completed_tasks().await {
            Ok(completed) => {
                state.load_completed_tasks(completed);
            }
            Err(e) => {
                eprintln!("Failed to fetch completed tasks: {}", e);
            }
        }
    }

    // Initialize and run UI
    let mut ui = UI::new()?;
    ui.run(app_state.clone(), client.clone()).await?;

    Ok(())
}
