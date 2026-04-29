use crate::config::app_settings::{self, AppSettings};
use crate::proxy::server;
use crate::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_app_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    app_settings::get_app_settings(&state.db).await
}

#[tauri::command]
pub async fn update_app_settings(
    state: State<'_, Arc<AppState>>,
    proxy_port: u16,
    auto_start_proxy: bool,
    log_retention_days: u32,
) -> Result<AppSettings, String> {
    let settings = AppSettings {
        proxy_port,
        auto_start_proxy,
        log_retention_days,
    };

    let previous_port = *state.proxy_port.lock().await;
    let should_restart = {
        let handle = state.proxy_handle.lock().await;
        handle.is_some() && previous_port != proxy_port
    };

    if should_restart {
        let mut handle = state.proxy_handle.lock().await;
        if let Some(existing) = handle.take() {
            existing.shutdown.abort();
        }
    }

    app_settings::update_app_settings(&state.db, &settings).await?;
    *state.proxy_port.lock().await = proxy_port;

    if should_restart {
        let proxy_handle = server::start_proxy_server(state.inner().clone(), proxy_port).await?;
        let actual_port = proxy_handle.port;
        *state.proxy_port.lock().await = actual_port;
        let mut handle = state.proxy_handle.lock().await;
        *handle = Some(proxy_handle);
        return Ok(AppSettings {
            proxy_port: actual_port,
            auto_start_proxy,
            log_retention_days,
        });
    }

    Ok(settings)
}
