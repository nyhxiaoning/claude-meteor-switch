use crate::db::logs;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type DbConn = Arc<Mutex<rusqlite::Connection>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub proxy_port: u16,
    pub auto_start_proxy: bool,
    pub log_retention_days: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            proxy_port: 9876,
            auto_start_proxy: false,
            log_retention_days: 90,
        }
    }
}

fn parse_bool(value: Option<String>, default: bool) -> bool {
    value
        .as_deref()
        .and_then(|raw| match raw {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn parse_u16(value: Option<String>, default: u16) -> u16 {
    value
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(default)
}

fn parse_u32(value: Option<String>, default: u32) -> u32 {
    value
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(default)
}

pub fn get_app_settings_sync(conn: &rusqlite::Connection) -> Result<AppSettings, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut proxy_port = None;
    let mut auto_start_proxy = None;
    let mut log_retention_days = None;

    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        match key.as_str() {
            "proxy_port" => proxy_port = Some(value),
            "auto_start_proxy" => auto_start_proxy = Some(value),
            "log_retention_days" => log_retention_days = Some(value),
            _ => {}
        }
    }

    let defaults = AppSettings::default();
    Ok(AppSettings {
        proxy_port: parse_u16(proxy_port, defaults.proxy_port),
        auto_start_proxy: parse_bool(auto_start_proxy, defaults.auto_start_proxy),
        log_retention_days: parse_u32(log_retention_days, defaults.log_retention_days),
    })
}

pub async fn get_app_settings(db: &DbConn) -> Result<AppSettings, String> {
    let db = db.clone();
    tokio::task::spawn_blocking(move || {
        let db = db.blocking_lock();
        get_app_settings_sync(&db)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn update_app_settings(db: &DbConn, settings: &AppSettings) -> Result<(), String> {
    let db = db.clone();
    let db_for_update = db.clone();
    let settings = settings.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let db = db_for_update.blocking_lock();
        let tx = db.unchecked_transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
            rusqlite::params!["proxy_port", settings.proxy_port.to_string()],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
            rusqlite::params!["auto_start_proxy", settings.auto_start_proxy.to_string()],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=datetime('now')",
            rusqlite::params![
                "log_retention_days",
                settings.log_retention_days.to_string()
            ],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    logs::cleanup_old_logs(&db, settings.log_retention_days).await
}
