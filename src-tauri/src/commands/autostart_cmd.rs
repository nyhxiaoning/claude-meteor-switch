use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app.autolaunch();
    autostart_manager
        .is_enabled()
        .map_err(|e| format!("Failed to check autostart status: {}", e))
}

#[tauri::command]
pub async fn enable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app.autolaunch();
    autostart_manager
        .enable()
        .map_err(|e| format!("Failed to enable autostart: {}", e))
}

#[tauri::command]
pub async fn disable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app.autolaunch();
    autostart_manager
        .disable()
        .map_err(|e| format!("Failed to disable autostart: {}", e))
}
