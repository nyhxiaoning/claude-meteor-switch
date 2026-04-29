use crate::config::provider::{AuthHeader, Protocol, Provider};
use crate::config::store;
use crate::AppState;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_providers(state: State<'_, Arc<AppState>>) -> Result<Vec<Provider>, String> {
    store::list_providers(&state.db).await
}

#[tauri::command]
pub async fn create_provider(
    state: State<'_, Arc<AppState>>,
    name: String,
    base_url: String,
    api_key: String,
    protocol: String,
    model_mapping: Option<String>,
    auth_header: String,
    keyword: String,
    enabled: bool,
) -> Result<Provider, String> {
    let provider = Provider {
        id: Uuid::new_v4().to_string(),
        name,
        base_url,
        api_key_enc: String::new(),
        protocol: Protocol::from_str(&protocol).unwrap_or(Protocol::Anthropic),
        model_mapping,
        auth_header: AuthHeader::from_str(&auth_header).unwrap_or(AuthHeader::ApiKey),
        keyword,
        enabled,
        sort_order: 0,
    };
    let mut provider = provider;
    provider.api_key_enc = store::store_api_key(&provider.id, &api_key);

    store::create_provider(&state.db, &provider).await?;
    Ok(provider)
}

#[tauri::command]
pub async fn update_provider(
    state: State<'_, Arc<AppState>>,
    id: String,
    name: String,
    base_url: String,
    api_key: Option<String>,
    protocol: String,
    model_mapping: Option<String>,
    auth_header: String,
    keyword: String,
    enabled: bool,
) -> Result<(), String> {
    let mut providers = store::list_providers(&state.db).await?;
    let provider = providers
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("Provider not found")?;

    provider.name = name;
    provider.base_url = base_url;
    if let Some(key) = api_key {
        provider.api_key_enc = store::store_api_key(&provider.id, &key);
    }
    provider.protocol = Protocol::from_str(&protocol).unwrap_or(Protocol::Anthropic);
    provider.model_mapping = model_mapping;
    provider.auth_header = AuthHeader::from_str(&auth_header).unwrap_or(AuthHeader::ApiKey);
    provider.keyword = keyword;
    provider.enabled = enabled;

    store::update_provider(&state.db, provider).await
}

#[tauri::command]
pub async fn delete_provider(state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    store::delete_provider(&state.db, &id).await
}
