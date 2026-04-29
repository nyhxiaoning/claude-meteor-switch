use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::{error, info, warn};

fn claude_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

fn backup_path() -> PathBuf {
    claude_dir().join(".meteor_env_backup.json")
}

const PROXY_API_KEY: &str = "sk-ant-api03-proxy-placeholder-0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupFile {
    env: Value,
}

fn read_json(path: &PathBuf) -> Result<Value, String> {
    info!("Reading JSON from {:?}", path);
    if !path.exists() {
        info!("  File does not exist, returning empty object");
        return Ok(json!({}));
    }
    let content = std::fs::read_to_string(path).map_err(|e| {
        error!("  Failed to read file: {}", e);
        e.to_string()
    })?;
    let result = serde_json::from_str(&content).map_err(|e| {
        error!("  Failed to parse JSON: {}", e);
        e.to_string()
    })?;
    info!("  Successfully read JSON");
    Ok(result)
}

fn write_json(path: &PathBuf, value: &Value) -> Result<(), String> {
    info!("Writing JSON to {:?}", path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            error!("  Failed to create parent directory: {}", e);
            e.to_string()
        })?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|e| {
        error!("  Failed to serialize JSON: {}", e);
        e.to_string()
    })?;
    std::fs::write(path, content).map_err(|e| {
        error!("  Failed to write to file: {}", e);
        e.to_string()
    })?;
    info!("  Successfully wrote JSON");
    Ok(())
}

fn is_proxy_base_url(value: &str) -> bool {
    value.starts_with("http://127.0.0.1:")
}

fn read_backup() -> Result<Option<Value>, String> {
    let backup = backup_path();
    if !backup.exists() {
        return Ok(None);
    }

    let value = read_json(&backup)?;

    if value.get("env").is_some() {
        return Ok(Some(value.get("env").cloned().unwrap_or_else(|| json!({}))));
    }

    Ok(Some(value))
}

fn write_backup(env: &Value) -> Result<(), String> {
    let payload = json!(BackupFile { env: env.clone() });
    write_json(&backup_path(), &payload)
}

fn remove_backup_file() {
    let backup = backup_path();
    match std::fs::remove_file(&backup) {
        Ok(_) => info!("  Backup file removed"),
        Err(e) => warn!("  Failed to remove backup file {:?}: {}", backup, e),
    }
}

fn set_env(settings: &mut Value, env: Value) {
    let should_remove = env.as_object().map(|obj| obj.is_empty()).unwrap_or(false);
    if should_remove {
        if let Some(settings_obj) = settings.as_object_mut() {
            settings_obj.remove("env");
        }
    } else {
        settings["env"] = env;
    }
}

fn remove_injected_keys(settings: &mut Value) {
    if let Some(env) = settings.get_mut("env").and_then(|e| e.as_object_mut()) {
        env.remove("ANTHROPIC_BASE_URL");
        env.remove("ANTHROPIC_API_KEY");
        env.remove("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC");

        if env.is_empty() {
            if let Some(settings_obj) = settings.as_object_mut() {
                settings_obj.remove("env");
            }
        }
    }
}

pub fn inject_config(port: u16) -> Result<(), String> {
    info!("=== inject_config START (port {}) ===", port);

    let path = settings_path();
    let backup = backup_path();

    info!("Settings path: {:?}", path);
    info!("Backup path: {:?}", backup);

    let mut settings = read_json(&path)?;

    if read_backup()?.is_none() {
        if let Some(env) = settings.get("env") {
            info!("Creating backup of original env");
            write_backup(env)?;
        } else {
            info!("No existing env to backup");
        }
    } else {
        info!("Backup already exists, not overwriting");
    }

    info!("Updating env with proxy settings");
    let mut env = if let Some(existing_env) = settings.get("env").and_then(|e| e.as_object()) {
        info!("  Using existing env and preserving keys");
        existing_env.clone()
    } else {
        info!("  No existing env, creating new");
        serde_json::Map::new()
    };

    env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        json!(format!("http://127.0.0.1:{}", port)),
    );
    env.insert("ANTHROPIC_API_KEY".to_string(), json!(PROXY_API_KEY));
    env.insert(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
        json!("true"),
    );
    env.remove("ANTHROPIC_AUTH_TOKEN");
    env.remove("ANTHROPIC_MODEL");
    env.remove("ANTHROPIC_DEFAULT_OPUS_MODEL");
    env.remove("ANTHROPIC_DEFAULT_SONNET_MODEL");
    env.remove("ANTHROPIC_DEFAULT_HAIKU_MODEL");

    settings["env"] = Value::Object(env);

    write_json(&path, &settings)?;

    info!("=== inject_config COMPLETE ===");
    Ok(())
}

pub fn revert_config() -> Result<(), String> {
    info!("=== revert_config START ===");

    let path = settings_path();
    let backup = backup_path();

    info!("Settings path: {:?}", path);
    info!("Backup path: {:?}", backup);

    let mut settings = read_json(&path)?;

    if let Some(original_env) = read_backup()? {
        info!("Backup found, restoring original env");
        set_env(&mut settings, original_env);
        remove_backup_file();
    } else {
        info!("No backup found, removing injected keys");
        remove_injected_keys(&mut settings);
    }

    write_json(&path, &settings)?;

    info!("=== revert_config COMPLETE ===");
    Ok(())
}

pub fn check_config() -> Result<serde_json::Value, String> {
    info!("=== check_config START ===");

    let settings = read_json(&settings_path())?;
    let base_url = settings
        .get("env")
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();

    let is_configured = is_proxy_base_url(&base_url);

    info!("  is_configured: {}, base_url: {}", is_configured, base_url);
    info!("=== check_config COMPLETE ===");

    Ok(json!({
        "is_configured": is_configured,
        "base_url": base_url
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn temp_home(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "claude-dynamic-meteor-{}-{}",
            name,
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn revert_restores_legacy_backup_env() {
        let _guard = env_lock().lock().expect("env lock");
        let original_home = std::env::var("HOME").ok();
        let home = temp_home("claude-settings");
        let claude_path = home.join(".claude");
        std::fs::create_dir_all(&claude_path).expect("create temp claude dir");
        std::env::set_var("HOME", &home);

        let legacy_settings = settings_path();
        write_json(
            &legacy_settings,
            &json!({
                "env": {
                    "ANTHROPIC_BASE_URL": "https://example.com",
                    "ANTHROPIC_AUTH_TOKEN": "oauth-token",
                    "ANTHROPIC_MODEL": "GLM-5.1",
                    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "custom-haiku"
                }
            }),
        )
        .expect("seed legacy settings");

        inject_config(9876).expect("inject config");
        let injected = read_json(&legacy_settings).expect("read injected settings");
        assert_eq!(
            injected["env"]["ANTHROPIC_BASE_URL"],
            json!("http://127.0.0.1:9876")
        );
        assert_eq!(injected["env"]["ANTHROPIC_API_KEY"], json!(PROXY_API_KEY));
        assert_eq!(
            injected["env"]["CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"],
            json!("true")
        );
        assert!(injected["env"].get("ANTHROPIC_AUTH_TOKEN").is_none());
        assert!(injected["env"].get("ANTHROPIC_MODEL").is_none());
        assert!(injected["env"]
            .get("ANTHROPIC_DEFAULT_HAIKU_MODEL")
            .is_none());

        revert_config().expect("revert config");

        let reverted = read_json(&legacy_settings).expect("read reverted settings");
        assert_eq!(
            reverted["env"]["ANTHROPIC_BASE_URL"],
            json!("https://example.com")
        );
        assert_eq!(
            reverted["env"]["ANTHROPIC_AUTH_TOKEN"],
            json!("oauth-token")
        );
        assert_eq!(reverted["env"]["ANTHROPIC_MODEL"], json!("GLM-5.1"));
        assert_eq!(
            reverted["env"]["ANTHROPIC_DEFAULT_HAIKU_MODEL"],
            json!("custom-haiku")
        );
        assert!(!backup_path().exists());

        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }

        let _ = std::fs::remove_dir_all(&home);
    }
}
