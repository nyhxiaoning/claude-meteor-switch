use crate::db::stats;
use crate::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_stats(state: State<'_, Arc<AppState>>) -> Result<stats::StatsResult, String> {
    stats::get_stats(&state.db).await
}
