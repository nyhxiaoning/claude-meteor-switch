use crate::proxy::router::match_provider;
use crate::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

pub async fn handle_messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string();

    // Parse request body
    let mut request_json: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return anthropic_error(400, &format!("Invalid request body: {}", e));
        }
    };

    let model = request_json
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();

    // Match provider
    let providers = match crate::config::store::get_enabled_providers(&state.db).await {
        Ok(p) => p,
        Err(e) => {
            return anthropic_error(500, &format!("Failed to load providers: {}", e));
        }
    };

    let provider = match match_provider(&model, &providers) {
        Some(p) => p,
        None => {
            return anthropic_error(400, &format!("No provider configured for model: {}", model));
        }
    };
    let upstream_model = provider
        .model_mapping
        .as_deref()
        .map(str::trim)
        .filter(|mapping| !mapping.is_empty())
        .unwrap_or(&model)
        .to_string();

    // Build upstream URL
    let is_openai = provider.protocol == crate::config::provider::Protocol::OpenAI;
    let upstream_path = if is_openai {
        "/v1/chat/completions"
    } else {
        "/v1/messages"
    };
    let upstream_url = format!(
        "{}{}",
        provider.base_url.trim_end_matches('/'),
        upstream_path
    );

    if let Some(obj) = request_json.as_object_mut() {
        obj.insert("model".to_string(), Value::String(upstream_model.clone()));
    }
    tracing::info!(
        "Routing model {} -> provider {} ({}) upstream model {}",
        model,
        provider.name,
        provider.keyword,
        upstream_model
    );

    // Convert request after all request mutations are applied.
    let upstream_body = if is_openai {
        match crate::adapter::openai::convert_request(&request_json, &provider) {
            Ok(converted) => serde_json::to_vec(&converted).unwrap_or(body.to_vec()),
            Err(e) => {
                return anthropic_error(500, &format!("Request conversion failed: {}", e));
            }
        }
    } else {
        match serde_json::to_vec(&request_json) {
            Ok(serialized) => serialized,
            Err(e) => {
                return anthropic_error(500, &format!("Failed to serialize request: {}", e));
            }
        }
    };

    // Build upstream request
    let client = Client::new();
    let mut req_builder = client
        .post(&upstream_url)
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(120));

    // Auth header
    let api_key = crate::config::store::decrypt_api_key(&provider.api_key_enc);
    tracing::info!(
        "Provider: {}, API Key length: {}, Is empty: {}",
        provider.name,
        api_key.len(),
        api_key.is_empty()
    );
    tracing::info!("Upstream URL: {}", upstream_url);

    match provider.auth_header {
        crate::config::provider::AuthHeader::ApiKey => {
            req_builder = req_builder.header("x-api-key", &api_key);
            if is_openai {
                req_builder = req_builder.header("authorization", format!("Bearer {}", api_key));
            }
        }
        crate::config::provider::AuthHeader::Bearer => {
            req_builder = req_builder.header("authorization", format!("Bearer {}", api_key));
        }
    }

    // Forward anthropic-version header if present
    if let Some(av) = headers.get("anthropic-version") {
        req_builder = req_builder.header("anthropic-version", av);
    }

    let resp = match req_builder.body(upstream_body).send().await {
        Ok(r) => r,
        Err(e) => {
            let latency = start.elapsed().as_millis() as i64;
            let _ = crate::db::logs::insert_log(
                &state.db,
                &crate::db::logs::RequestLog {
                    id: 0,
                    request_id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    model: model.clone(),
                    provider_id: provider.id.clone(),
                    provider_name: provider.name.clone(),
                    protocol: provider.protocol.as_str().to_string(),
                    upstream_url: upstream_url.clone(),
                    status_code: None,
                    latency_ms: Some(latency),
                    input_tokens: 0,
                    output_tokens: 0,
                    error_message: Some(e.to_string()),
                    is_streaming: true,
                },
            )
            .await;
            return anthropic_error(502, &format!("Upstream unreachable: {}", e));
        }
    };

    let status = resp.status();
    let status_code = status.as_u16() as i32;

    // Check if streaming
    let is_streaming = request_json
        .get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(true);

    if is_streaming && status.is_success() {
        // Stream response
        let _content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if is_openai {
            // OpenAI SSE → Anthropic SSE conversion
            let model_for_convert = model.clone();
            let stream = resp.bytes_stream();
            let converted = crate::adapter::openai::convert_sse_stream(
                stream,
                &model_for_convert,
                provider.model_mapping.as_deref(),
            );
            let crate::adapter::MonitoredSseStream {
                stream: response_stream,
                summary,
            } = converted;
            let db = state.db.clone();
            let request_id_for_log = request_id.clone();
            let provider_id = provider.id.clone();
            let provider_name = provider.name.clone();
            let protocol = provider.protocol.as_str().to_string();
            let upstream_url_for_log = upstream_url.clone();
            let model_for_log = model.clone();
            let start_time = start;

            tokio::spawn(async move {
                let summary = summary.await.unwrap_or(crate::adapter::StreamSummary {
                    input_tokens: 0,
                    output_tokens: 0,
                    error_message: Some("Streaming summary unavailable".to_string()),
                });
                let latency = start_time.elapsed().as_millis() as i64;
                let _ = crate::db::logs::insert_log(
                    &db,
                    &crate::db::logs::RequestLog {
                        id: 0,
                        request_id: request_id_for_log,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        model: model_for_log,
                        provider_id,
                        provider_name,
                        protocol,
                        upstream_url: upstream_url_for_log,
                        status_code: Some(status_code),
                        latency_ms: Some(latency),
                        input_tokens: summary.input_tokens,
                        output_tokens: summary.output_tokens,
                        error_message: summary.error_message,
                        is_streaming: true,
                    },
                )
                .await;
            });

            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .body(Body::from_stream(response_stream))
                .unwrap();
        } else {
            // Anthropic pass-through
            let stream = resp.bytes_stream();
            let monitored = crate::adapter::anthropic::monitor_sse_stream(stream);
            let crate::adapter::MonitoredSseStream {
                stream: response_stream,
                summary,
            } = monitored;
            let db = state.db.clone();
            let request_id_for_log = request_id.clone();
            let provider_id = provider.id.clone();
            let provider_name = provider.name.clone();
            let protocol = provider.protocol.as_str().to_string();
            let upstream_url_for_log = upstream_url.clone();
            let model_for_log = model.clone();
            let start_time = start;

            tokio::spawn(async move {
                let summary = summary.await.unwrap_or(crate::adapter::StreamSummary {
                    input_tokens: 0,
                    output_tokens: 0,
                    error_message: Some("Streaming summary unavailable".to_string()),
                });
                let latency = start_time.elapsed().as_millis() as i64;
                let _ = crate::db::logs::insert_log(
                    &db,
                    &crate::db::logs::RequestLog {
                        id: 0,
                        request_id: request_id_for_log,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        model: model_for_log,
                        provider_id,
                        provider_name,
                        protocol,
                        upstream_url: upstream_url_for_log,
                        status_code: Some(status_code),
                        latency_ms: Some(latency),
                        input_tokens: summary.input_tokens,
                        output_tokens: summary.output_tokens,
                        error_message: summary.error_message,
                        is_streaming: true,
                    },
                )
                .await;
            });

            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .body(Body::from_stream(response_stream))
                .unwrap();
        }
    } else {
        // Non-streaming response
        let resp_bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return anthropic_error(502, &format!("Failed to read response: {}", e));
            }
        };

        let latency = start.elapsed().as_millis() as i64;

        if is_openai && status.is_success() {
            // Convert OpenAI response to Anthropic format
            match serde_json::from_slice::<Value>(&resp_bytes) {
                Ok(json) => {
                    let converted = crate::adapter::openai::convert_response(
                        &json,
                        &model,
                        provider.model_mapping.as_deref(),
                    );
                    let resp_body = serde_json::to_vec(&converted).unwrap_or(resp_bytes.to_vec());

                    // Log
                    let input_tokens = converted
                        .get("usage")
                        .and_then(|u| u.get("input_tokens"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let output_tokens = converted
                        .get("usage")
                        .and_then(|u| u.get("output_tokens"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    let _ = crate::db::logs::insert_log(
                        &state.db,
                        &crate::db::logs::RequestLog {
                            id: 0,
                            request_id,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            model,
                            provider_id: provider.id.clone(),
                            provider_name: provider.name.clone(),
                            protocol: provider.protocol.as_str().to_string(),
                            upstream_url,
                            status_code: Some(status_code),
                            latency_ms: Some(latency),
                            input_tokens,
                            output_tokens,
                            error_message: None,
                            is_streaming: false,
                        },
                    )
                    .await;

                    return Response::builder()
                        .status(StatusCode::from_u16(status_code as u16).unwrap())
                        .header("Content-Type", "application/json")
                        .body(Body::from(resp_body))
                        .unwrap();
                }
                Err(_) => {
                    return anthropic_error(500, "Failed to parse upstream response");
                }
            }
        } else if is_openai {
            let converted =
                crate::adapter::openai::convert_error_response(&resp_bytes, status_code as u16);

            let _ = crate::db::logs::insert_log(
                &state.db,
                &crate::db::logs::RequestLog {
                    id: 0,
                    request_id,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    model,
                    provider_id: provider.id.clone(),
                    provider_name: provider.name.clone(),
                    protocol: provider.protocol.as_str().to_string(),
                    upstream_url,
                    status_code: Some(status_code),
                    latency_ms: Some(latency),
                    input_tokens: 0,
                    output_tokens: 0,
                    error_message: converted
                        .get("error")
                        .and_then(|value| value.get("message"))
                        .and_then(|value| value.as_str())
                        .map(|value| value.to_string()),
                    is_streaming: false,
                },
            )
            .await;

            return Response::builder()
                .status(StatusCode::from_u16(status_code as u16).unwrap())
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&converted).unwrap_or_default(),
                ))
                .unwrap();
        } else {
            // Anthropic format or error - pass through
            return Response::builder()
                .status(StatusCode::from_u16(status_code as u16).unwrap())
                .header("Content-Type", "application/json")
                .body(Body::from(resp_bytes.to_vec()))
                .unwrap();
        }
    }
}

pub async fn handle_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let providers = crate::config::store::get_enabled_providers(&state.db)
        .await
        .unwrap_or_default();

    let models: Vec<Value> = providers
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": format!("claude-{}-4-6", p.keyword),
                "display_name": p.name,
            })
        })
        .collect();

    axum::Json(serde_json::json!({
        "data": models,
        "object": "list"
    }))
}

pub async fn handle_health() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "status": "ok" }))
}

fn anthropic_error(status: u16, message: &str) -> Response {
    let body = serde_json::json!({
        "type": "error",
        "error": {
            "type": "invalid_request_error",
            "message": message
        }
    });
    Response::builder()
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap_or_default()))
        .unwrap()
}
