mod api;
mod ui;
mod state;

use api::TodoistClient;
use state::AppState;
use ui::UI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Get API token from environment or config
    let api_token = std::env::var("TODOIST_API_TOKEN")
        .unwrap_or_else(|_| "placeholder_token".to_string());
    
    let client = TodoistClient::new(api_token);
    let mut app_state = AppState::new();
    
    // Fetch today's tasks
    match client.get_todays_tasks().await {
        Ok(tasks) => {
            app_state.load_tasks(tasks);
            app_state.sync_status = state::SyncStatus::Online;
        }
        Err(e) => {
            eprintln!("Failed to fetch tasks: {}", e);
            app_state.sync_status = state::SyncStatus::Error(e.to_string());
        }
    }
    
    // Initialize and run UI
    let mut ui = UI::new()?;
    ui.run(&mut app_state)?;
    
    Ok(())
}
