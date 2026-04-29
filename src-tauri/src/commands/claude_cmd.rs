use crate::claude::settings;
use crate::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn inject_claude_config(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let port = *state.proxy_port.lock().await;
    settings::inject_config(port)
}

#[tauri::command]
pub async fn revert_claude_config() -> Result<(), String> {
    settings::revert_config()
}

#[tauri::command]
pub async fn check_claude_config() -> Result<serde_json::Value, String> {
    settings::check_config()
}
