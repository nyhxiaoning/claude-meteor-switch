use crate::config::provider::{AuthHeader, Protocol, Provider};
use aes_gcm::aead::{rand_core::RngCore, Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use keyring::Entry;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

const KEYRING_SERVICE: &str = "claude-dynamic-meteor";
const KEYRING_PREFIX: &str = "keyring:";
const ENCRYPTED_PREFIX: &str = "enc-v1:";
const LEGACY_ENCRYPTION_KEY: &[u8; 32] = b"claude-dynamic-meteor-k32bytes!!";
const LEGACY_NONCE: &[u8] = b"unique-12byte-n";

fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude-dynamic-meteor")
}

fn fallback_key_path() -> PathBuf {
    app_data_dir().join("master.key")
}

fn load_or_create_fallback_key() -> Result<[u8; 32], String> {
    let path = fallback_key_path();
    if path.exists() {
        let encoded = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let decoded = BASE64.decode(encoded.trim()).map_err(|e| e.to_string())?;
        return decoded
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid fallback encryption key length".to_string());
    }

    std::fs::create_dir_all(path.parent().unwrap_or_else(|| std::path::Path::new(".")))
        .map_err(|e| e.to_string())?;

    let mut key = [0_u8; 32];
    OsRng.fill_bytes(&mut key);

    std::fs::write(&path, BASE64.encode(key)).map_err(|e| e.to_string())?;
    Ok(key)
}

fn encrypt_api_key_fallback(plaintext: &str) -> Result<String, String> {
    let key = load_or_create_fallback_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "{}{}:{}",
        ENCRYPTED_PREFIX,
        BASE64.encode(nonce),
        BASE64.encode(ciphertext)
    ))
}

fn decrypt_api_key_fallback(enc: &str) -> Result<String, String> {
    let payload = enc
        .strip_prefix(ENCRYPTED_PREFIX)
        .ok_or("Missing encrypted payload prefix")?;
    let (nonce_b64, ciphertext_b64) = payload
        .split_once(':')
        .ok_or("Invalid encrypted payload format")?;

    let nonce_bytes = BASE64.decode(nonce_b64).map_err(|e| e.to_string())?;
    let ciphertext = BASE64.decode(ciphertext_b64).map_err(|e| e.to_string())?;
    let key = load_or_create_fallback_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_slice())
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

fn decrypt_legacy_api_key(enc: &str) -> Option<String> {
    let cipher = Aes256Gcm::new_from_slice(LEGACY_ENCRYPTION_KEY).ok()?;
    let nonce_bytes: &[u8; 12] = LEGACY_NONCE.get(..12)?.try_into().ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let ciphertext = BASE64.decode(enc).ok()?;
    let plaintext = cipher.decrypt(nonce, ciphertext.as_slice()).ok()?;
    String::from_utf8(plaintext).ok()
}

fn keyring_entry(provider_id: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, provider_id).map_err(|e| e.to_string())
}

pub fn store_api_key(provider_id: &str, plaintext: &str) -> String {
    tracing::debug!("Storing API key for provider {}", provider_id);

    // Try keyring first
    match keyring_entry(provider_id).and_then(|entry| {
        entry
            .set_password(plaintext)
            .map(|_| format!("{}{}", KEYRING_PREFIX, provider_id))
            .map_err(|e| e.to_string())
    }) {
        Ok(reference) => {
            tracing::debug!("Successfully stored API key in keyring");
            // Verify we can get it back
            if let Ok(verify_entry) = keyring_entry(provider_id) {
                if let Ok(verify_pw) = verify_entry.get_password() {
                    if verify_pw == plaintext {
                        tracing::debug!("Keyring verification passed");
                        return reference;
                    }
                }
            }
            tracing::warn!("Keyring verification failed, falling back");
        }
        Err(error) => {
            tracing::warn!(
                "Failed to store API key in keyring for provider {}: {}. Falling back to encrypted file storage.",
                provider_id,
                error
            );
        }
    }

    // Fallback to encrypted storage
    match encrypt_api_key_fallback(plaintext) {
        Ok(encrypted) => {
            tracing::debug!("Successfully stored API key via fallback encryption");
            encrypted
        }
        Err(fallback_error) => {
            tracing::error!(
                "Failed to store API key for provider {} via fallback encryption: {}",
                provider_id,
                fallback_error
            );
            // Last resort: just store plaintext (better than nothing)
            tracing::warn!("Storing API key as plaintext as last resort");
            plaintext.to_string()
        }
    }
}

pub fn decrypt_api_key(enc: &str) -> String {
    tracing::debug!(
        "Decrypting API key, prefix: {:?}",
        &enc[..std::cmp::min(20, enc.len())]
    );

    if let Some(provider_id) = enc.strip_prefix(KEYRING_PREFIX) {
        match keyring_entry(provider_id)
            .and_then(|entry| entry.get_password().map_err(|e| e.to_string()))
        {
            Ok(pw) if !pw.is_empty() => {
                tracing::debug!("Successfully retrieved API key from keyring");
                return pw;
            }
            Ok(_) => {
                tracing::warn!("API key from keyring is empty");
            }
            Err(error) => {
                tracing::error!(
                    "Failed to load API key from keyring for provider {}: {}",
                    provider_id,
                    error
                );
            }
        }
        // Fall through - don't return empty string
    }

    if enc.starts_with(ENCRYPTED_PREFIX) {
        match decrypt_api_key_fallback(enc) {
            Ok(pw) if !pw.is_empty() => {
                tracing::debug!("Successfully decrypted API key from fallback");
                return pw;
            }
            Ok(_) => {
                tracing::warn!("Decrypted API key from fallback is empty");
            }
            Err(error) => {
                tracing::error!("Failed to decrypt fallback API key: {}", error);
            }
        }
        // Fall through
    }

    // Last resort: check if it's plaintext
    if let Some(pw) = decrypt_legacy_api_key(enc) {
        if !pw.is_empty() {
            tracing::debug!("Using legacy decrypted API key");
            return pw;
        }
    }

    // If nothing worked, return the original (maybe it's plaintext?)
    tracing::warn!("All decryption methods failed, returning original as-is");
    enc.to_string()
}

pub type DbConn = Arc<Mutex<rusqlite::Connection>>;

pub async fn list_providers(db: &DbConn) -> Result<Vec<Provider>, String> {
    let db = db.clone();

    tokio::task::spawn_blocking(move || {
        let db = db.blocking_lock();
        let mut stmt = db
            .prepare("SELECT id, name, base_url, api_key_enc, protocol, model_mapping, auth_header, keyword, enabled, sort_order FROM providers ORDER BY sort_order")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let protocol_str: String = row.get(4)?;
                let auth_str: String = row.get(6)?;
                let model_mapping: Option<String> = row.get(5)?;
                Ok(Provider {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    base_url: row.get(2)?,
                    api_key_enc: row.get(3)?,
                    protocol: Protocol::from_str(&protocol_str).unwrap_or(Protocol::Anthropic),
                    model_mapping,
                    auth_header: AuthHeader::from_str(&auth_str).unwrap_or(AuthHeader::ApiKey),
                    keyword: row.get(7)?,
                    enabled: row.get(8)?,
                    sort_order: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut providers = Vec::new();
        for row in rows {
            providers.push(row.map_err(|e| e.to_string())?);
        }
        Ok(providers)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn create_provider(db: &DbConn, provider: &Provider) -> Result<(), String> {
    let db = db.clone();
    let provider = provider.clone();

    tokio::task::spawn_blocking(move || {
        let db = db.blocking_lock();
        db.execute(
            "INSERT INTO providers (id, name, base_url, api_key_enc, protocol, model_mapping, auth_header, keyword, enabled, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                provider.id,
                provider.name,
                provider.base_url,
                provider.api_key_enc,
                provider.protocol.as_str(),
                provider.model_mapping,
                provider.auth_header.as_str(),
                provider.keyword,
                provider.enabled,
                provider.sort_order,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn update_provider(db: &DbConn, provider: &Provider) -> Result<(), String> {
    let db = db.clone();
    let provider = provider.clone();

    tokio::task::spawn_blocking(move || {
        let db = db.blocking_lock();
        db.execute(
            "UPDATE providers SET name=?1, base_url=?2, api_key_enc=?3, protocol=?4, model_mapping=?5, auth_header=?6, keyword=?7, enabled=?8, sort_order=?9, updated_at=datetime('now') WHERE id=?10",
            rusqlite::params![
                provider.name,
                provider.base_url,
                provider.api_key_enc,
                provider.protocol.as_str(),
                provider.model_mapping,
                provider.auth_header.as_str(),
                provider.keyword,
                provider.enabled,
                provider.sort_order,
                provider.id,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn delete_provider(db: &DbConn, id: &str) -> Result<(), String> {
    let db = db.clone();
    let id = id.to_string();

    tokio::task::spawn_blocking(move || {
        let db = db.blocking_lock();
        db.execute("DELETE FROM providers WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn get_enabled_providers(db: &DbConn) -> Result<Vec<Provider>, String> {
    let providers = list_providers(db).await?;
    Ok(providers.into_iter().filter(|p| p.enabled).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_encryption_round_trip() {
        let plaintext = "sk-test-123";
        let encrypted =
            encrypt_api_key_fallback(plaintext).expect("fallback encryption should work");
        assert!(encrypted.starts_with(ENCRYPTED_PREFIX));
        assert_eq!(
            decrypt_api_key_fallback(&encrypted).expect("fallback decryption should work"),
            plaintext
        );
    }

    #[test]
    fn unencrypted_value_is_left_as_is() {
        assert_eq!(decrypt_api_key("plain-text-key"), "plain-text-key");
    }
}
