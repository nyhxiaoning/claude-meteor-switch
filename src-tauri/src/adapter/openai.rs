use crate::adapter::{LlmAdapter, MonitoredSseStream, StreamSummary};
use crate::config::provider::Protocol;
use crate::config::provider::Provider;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::StreamExt;
use serde_json::{json, Value};
use tokio::sync::{mpsc, oneshot};

#[allow(dead_code)]
pub struct OpenAIAdapter;

#[async_trait]
impl LlmAdapter for OpenAIAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::OpenAI
    }
}

/// Convert Anthropic Messages API request to OpenAI Chat Completions request
pub fn convert_request(request: &Value, provider: &Provider) -> Result<Value, String> {
    let mut openai_request = json!({});

    // Model mapping
    let model = provider
        .model_mapping
        .as_deref()
        .map(str::trim)
        .filter(|mapping| !mapping.is_empty())
        .unwrap_or(
            request
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("gpt-4"),
        );
    openai_request["model"] = json!(model);

    // max_tokens
    if let Some(mt) = request.get("max_tokens") {
        openai_request["max_tokens"] = mt.clone();
    }

    // stream
    let stream = request
        .get("stream")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    openai_request["stream"] = json!(stream);
    if stream {
        openai_request["stream_options"] = json!({"include_usage": true});
    }

    // Convert messages
    let mut messages = Vec::new();

    // System prompt → first system message
    if let Some(system) = request.get("system") {
        let system_content = match system {
            Value::String(s) => s.clone(),
            Value::Array(blocks) => blocks
                .iter()
                .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        };
        if !system_content.is_empty() {
            messages.push(json!({"role": "system", "content": system_content}));
        }
    }

    // Convert messages
    if let Some(msgs) = request.get("messages").and_then(|m| m.as_array()) {
        for msg in msgs {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = msg.get("content");

            if role == "assistant" {
                // Check for tool_use in content
                if let Some(Value::Array(blocks)) = content {
                    let mut text_parts = Vec::new();
                    let mut tool_calls = Vec::new();

                    for block in blocks {
                        match block.get("type").and_then(|t| t.as_str()) {
                            Some("text") => {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    text_parts.push(text.to_string());
                                }
                            }
                            Some("tool_use") => {
                                let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let default_input = json!({});
                                let input = block.get("input").unwrap_or(&default_input);
                                tool_calls.push(json!({
                                    "id": id,
                                    "type": "function",
                                    "function": {
                                        "name": name,
                                        "arguments": serde_json::to_string(input).unwrap_or_default()
                                    }
                                }));
                            }
                            _ => {}
                        }
                    }

                    let mut assistant_msg = json!({"role": "assistant"});
                    if !text_parts.is_empty() {
                        assistant_msg["content"] = json!(text_parts.join(""));
                    } else {
                        assistant_msg["content"] = Value::Null;
                    }
                    if !tool_calls.is_empty() {
                        assistant_msg["tool_calls"] = json!(tool_calls);
                    }
                    messages.push(assistant_msg);
                } else if let Some(Value::String(text)) = content {
                    messages.push(json!({"role": "assistant", "content": text}));
                }
            } else if role == "user" {
                if let Some(Value::Array(blocks)) = content {
                    let mut parts = Vec::new();
                    for block in blocks {
                        match block.get("type").and_then(|t| t.as_str()) {
                            Some("text") => {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    parts.push(json!({"type": "text", "text": text}));
                                }
                            }
                            Some("image") => {
                                if let Some(source) = block.get("source") {
                                    let media_type = source
                                        .get("media_type")
                                        .and_then(|m| m.as_str())
                                        .unwrap_or("image/png");
                                    let data =
                                        source.get("data").and_then(|d| d.as_str()).unwrap_or("");
                                    parts.push(json!({
                                        "type": "image_url",
                                        "image_url": {"url": format!("data:{};base64,{}", media_type, data)}
                                    }));
                                }
                            }
                            Some("tool_result") => {
                                let tool_use_id = block
                                    .get("tool_use_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let result_content =
                                    block.get("content").and_then(|c| c.as_str()).unwrap_or("");
                                messages.push(json!({
                                    "role": "tool",
                                    "tool_call_id": tool_use_id,
                                    "content": result_content
                                }));
                            }
                            _ => {}
                        }
                    }
                    if !parts.is_empty() {
                        messages.push(json!({"role": "user", "content": parts}));
                    }
                } else if let Some(Value::String(text)) = content {
                    messages.push(json!({"role": "user", "content": text}));
                }
            }
        }
    }

    openai_request["messages"] = json!(messages);

    // Convert tools
    if let Some(tools) = request.get("tools").and_then(|t| t.as_array()) {
        let openai_tools: Vec<Value> = tools
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool["name"],
                        "description": tool.get("description").unwrap_or(&json!("")),
                        "parameters": tool.get("input_schema").unwrap_or(&json!({}))
                    }
                })
            })
            .collect();
        openai_request["tools"] = json!(openai_tools);
    }

    // Convert tool_choice
    if let Some(tc) = request.get("tool_choice") {
        let openai_tc = match tc {
            Value::String(s) if s == "auto" => json!({"type": "auto"}),
            Value::String(s) if s == "any" => json!({"type": "required"}),
            Value::String(s) if s == "none" => json!({"type": "none"}),
            Value::Object(obj) => {
                if obj.get("type").and_then(|t| t.as_str()) == Some("tool") {
                    json!({
                        "type": "function",
                        "function": {"name": obj.get("name").and_then(|n| n.as_str()).unwrap_or("")}
                    })
                } else {
                    tc.clone()
                }
            }
            _ => json!({"type": "auto"}),
        };
        openai_request["tool_choice"] = openai_tc;
    }

    // stop_sequences → stop
    if let Some(ss) = request.get("stop_sequences") {
        openai_request["stop"] = ss.clone();
    }

    Ok(openai_request)
}

/// Convert non-streaming OpenAI response to Anthropic format
pub fn convert_response(
    openai_resp: &Value,
    original_model: &str,
    model_mapping: Option<&str>,
) -> Value {
    let default_choice = json!({});
    let choice = openai_resp
        .get("choices")
        .and_then(|c| c.get(0))
        .unwrap_or(&default_choice);
    let default_message = json!({});
    let message = choice.get("message").unwrap_or(&default_message);
    let finish_reason = choice
        .get("finish_reason")
        .and_then(|f| f.as_str())
        .unwrap_or("stop");

    let stop_reason = match finish_reason {
        "stop" => "end_turn",
        "tool_calls" => "tool_use",
        "length" => "max_tokens",
        _ => "end_turn",
    };

    let mut content = Vec::new();

    // Text content
    if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
        if !text.is_empty() {
            content.push(json!({"type": "text", "text": text}));
        }
    }

    // Tool calls
    if let Some(tool_calls) = message.get("tool_calls").and_then(|t| t.as_array()) {
        for tc in tool_calls {
            let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let name = tc
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments = tc
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|a| a.as_str())
                .unwrap_or("{}");
            let input: Value = serde_json::from_str(arguments).unwrap_or(json!({}));

            content.push(json!({
                "type": "tool_use",
                "id": id,
                "name": name,
                "input": input
            }));
        }
    }

    let default_usage = json!({});
    let usage = openai_resp.get("usage").unwrap_or(&default_usage);
    let input_tokens = usage
        .get("prompt_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let output_tokens = usage
        .get("completion_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let msg_id = format!(
        "msg-{}",
        uuid::Uuid::new_v4().to_string().replace("-", "")[..24].to_string()
    );

    let response_model = model_mapping.unwrap_or(original_model);

    json!({
        "id": msg_id,
        "type": "message",
        "role": "assistant",
        "content": content,
        "model": response_model,
        "stop_reason": stop_reason,
        "stop_sequence": Value::Null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    })
}

pub fn convert_error_response(openai_error: &[u8], status: u16) -> Value {
    let parsed = serde_json::from_slice::<Value>(openai_error).ok();
    let error = parsed.as_ref().and_then(|value| value.get("error"));

    let message = error
        .and_then(|value| value.get("message"))
        .and_then(|value| value.as_str())
        .unwrap_or("Upstream request failed");

    let error_type = error
        .and_then(|value| value.get("type"))
        .and_then(|value| value.as_str())
        .map(map_openai_error_type)
        .unwrap_or_else(|| default_anthropic_error_type(status));

    json!({
        "type": "error",
        "error": {
            "type": error_type,
            "message": message
        }
    })
}

fn map_openai_error_type(error_type: &str) -> &str {
    match error_type {
        "invalid_request_error" => "invalid_request_error",
        "authentication_error" | "invalid_api_key" => "authentication_error",
        "permission_error" => "permission_error",
        "rate_limit_error" | "insufficient_quota" => "rate_limit_error",
        "server_error" => "api_error",
        _ => "api_error",
    }
}

fn default_anthropic_error_type(status: u16) -> &'static str {
    match status {
        400 => "invalid_request_error",
        401 => "authentication_error",
        403 => "permission_error",
        429 => "rate_limit_error",
        500..=599 => "api_error",
        _ => "api_error",
    }
}

/// SSE event frame for Anthropic format
struct AnthropicSseFrame {
    event: String,
    data: String,
}

impl std::fmt::Display for AnthropicSseFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "event: {}\ndata: {}\n\n", self.event, self.data)
    }
}

/// Convert OpenAI SSE stream to Anthropic SSE stream
pub fn convert_sse_stream(
    stream: impl futures::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
    model: &str,
    model_mapping: Option<&str>,
) -> MonitoredSseStream {
    let response_model = model_mapping.unwrap_or(model).to_string();
    let msg_id = format!(
        "msg-{}",
        uuid::Uuid::new_v4().to_string().replace("-", "")[..24].to_string()
    );

    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(32);
    let (summary_tx, summary_rx) = oneshot::channel::<StreamSummary>();

    tokio::spawn(async move {
        let mut buffer = String::new();
        let mut message_started = false;
        let mut message_finished = false;
        let mut text_block_index: Option<u32> = None;
        let mut content_block_index: u32 = 0;
        let mut active_tool_calls: std::collections::HashMap<usize, ToolCallState> =
            std::collections::HashMap::new();
        let mut input_tokens: u64 = 0;
        let mut output_tokens: u64 = 0;
        let mut error_message = None;

        let mut stream = Box::pin(stream);

        while let Some(chunk_result) = stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(e) => {
                    error_message = Some(e.to_string());
                    let _ = tx.send(Err(std::io::Error::other(e.to_string()))).await;
                    break;
                }
            };

            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find("\n\n") {
                let event_text = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                for line in event_text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            if !message_finished {
                                for tc_state in active_tool_calls.values() {
                                    let close_frame = AnthropicSseFrame {
                                        event: "content_block_stop".to_string(),
                                        data: json!({
                                            "type": "content_block_stop",
                                            "index": tc_state.index
                                        })
                                        .to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(close_frame.to_string()))).await;
                                }
                                active_tool_calls.clear();

                                if let Some(index) = text_block_index.take() {
                                    let frame = AnthropicSseFrame {
                                        event: "content_block_stop".to_string(),
                                        data: json!({
                                            "type": "content_block_stop",
                                            "index": index
                                        })
                                        .to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(frame.to_string()))).await;
                                }

                                let stop_frame = AnthropicSseFrame {
                                    event: "message_delta".to_string(),
                                    data: json!({
                                        "type": "message_delta",
                                        "delta": {"stop_reason": "end_turn"},
                                        "usage": {"output_tokens": output_tokens}
                                    })
                                    .to_string(),
                                };
                                let _ = tx.send(Ok(Bytes::from(stop_frame.to_string()))).await;

                                let end_frame = AnthropicSseFrame {
                                    event: "message_stop".to_string(),
                                    data: json!({"type": "message_stop"}).to_string(),
                                };
                                let _ = tx.send(Ok(Bytes::from(end_frame.to_string()))).await;
                                message_finished = true;
                            }
                            continue;
                        }

                        let parsed: Value = match serde_json::from_str(data) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };

                        // Extract usage if present
                        if let Some(usage) = parsed.get("usage") {
                            if let Some(pt) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
                                input_tokens = pt;
                            }
                            if let Some(ct) =
                                usage.get("completion_tokens").and_then(|v| v.as_u64())
                            {
                                output_tokens = ct;
                            }
                        }

                        let choice = parsed.get("choices").and_then(|c| c.get(0));
                        if choice.is_none() {
                            continue;
                        }
                        let choice = choice.unwrap();
                        let default_delta = json!({});
                        let delta = choice.get("delta").unwrap_or(&default_delta);
                        let finish_reason = choice.get("finish_reason").and_then(|f| f.as_str());

                        // message_start on first chunk
                        if !message_started {
                            let frame = AnthropicSseFrame {
                                event: "message_start".to_string(),
                                data: json!({
                                    "type": "message_start",
                                    "message": {
                                        "id": msg_id,
                                        "type": "message",
                                        "role": "assistant",
                                        "content": [],
                                        "model": response_model,
                                        "usage": {"input_tokens": input_tokens, "output_tokens": 0}
                                    }
                                })
                                .to_string(),
                            };
                            let _ = tx.send(Ok(Bytes::from(frame.to_string()))).await;
                            message_started = true;
                        }

                        // Handle text content
                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                if text_block_index.is_none() {
                                    let block_index = content_block_index;
                                    let start_frame = AnthropicSseFrame {
                                        event: "content_block_start".to_string(),
                                        data: json!({
                                            "type": "content_block_start",
                                            "index": block_index,
                                            "content_block": {"type": "text", "text": ""}
                                        })
                                        .to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(start_frame.to_string()))).await;
                                    text_block_index = Some(block_index);
                                }

                                let block_index = text_block_index.unwrap_or(content_block_index);
                                let delta_frame = AnthropicSseFrame {
                                    event: "content_block_delta".to_string(),
                                    data: json!({
                                        "type": "content_block_delta",
                                        "index": block_index,
                                        "delta": {"type": "text_delta", "text": content}
                                    })
                                    .to_string(),
                                };
                                let _ = tx.send(Ok(Bytes::from(delta_frame.to_string()))).await;
                            }
                        }

                        // Handle tool calls
                        if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array())
                        {
                            for tc in tool_calls {
                                let tc_index =
                                    tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                                let tc_id = tc.get("id").and_then(|i| i.as_str()).unwrap_or("");
                                let fn_name = tc
                                    .get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("");
                                let fn_args = tc
                                    .get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|a| a.as_str())
                                    .unwrap_or("");

                                // New tool call
                                if !tc_id.is_empty() {
                                    // Close text block if open
                                    if let Some(index) = text_block_index.take() {
                                        let close_frame = AnthropicSseFrame {
                                            event: "content_block_stop".to_string(),
                                            data: json!({
                                                "type": "content_block_stop",
                                                "index": index
                                            })
                                            .to_string(),
                                        };
                                        let _ =
                                            tx.send(Ok(Bytes::from(close_frame.to_string()))).await;
                                        content_block_index += 1;
                                    }

                                    let start_frame = AnthropicSseFrame {
                                        event: "content_block_start".to_string(),
                                        data: json!({
                                            "type": "content_block_start",
                                            "index": content_block_index,
                                            "content_block": {
                                                "type": "tool_use",
                                                "id": tc_id,
                                                "name": fn_name,
                                                "input": {}
                                            }
                                        })
                                        .to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(start_frame.to_string()))).await;

                                    active_tool_calls.insert(
                                        tc_index,
                                        ToolCallState {
                                            index: content_block_index,
                                            id: tc_id.to_string(),
                                            name: fn_name.to_string(),
                                        },
                                    );

                                    content_block_index += 1;
                                }

                                // Arguments delta
                                if !fn_args.is_empty() {
                                    let tc_state = active_tool_calls.get(&tc_index);
                                    let block_index = tc_state
                                        .map(|s| s.index)
                                        .unwrap_or(content_block_index - 1);
                                    let delta_frame = AnthropicSseFrame {
                                        event: "content_block_delta".to_string(),
                                        data: json!({
                                            "type": "content_block_delta",
                                            "index": block_index,
                                            "delta": {"type": "input_json_delta", "partial_json": fn_args}
                                        }).to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(delta_frame.to_string()))).await;
                                }
                            }
                        }

                        // Finish reason
                        if let Some(reason) = finish_reason {
                            if !message_finished {
                                if let Some(index) = text_block_index.take() {
                                    let close_frame = AnthropicSseFrame {
                                        event: "content_block_stop".to_string(),
                                        data: json!({
                                            "type": "content_block_stop",
                                            "index": index
                                        })
                                        .to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(close_frame.to_string()))).await;
                                }

                                for tc_state in active_tool_calls.values() {
                                    let close_frame = AnthropicSseFrame {
                                        event: "content_block_stop".to_string(),
                                        data: json!({
                                            "type": "content_block_stop",
                                            "index": tc_state.index
                                        })
                                        .to_string(),
                                    };
                                    let _ = tx.send(Ok(Bytes::from(close_frame.to_string()))).await;
                                }
                                active_tool_calls.clear();

                                let stop_reason = match reason {
                                    "stop" => "end_turn",
                                    "tool_calls" => "tool_use",
                                    "length" => "max_tokens",
                                    _ => "end_turn",
                                };

                                let delta_frame = AnthropicSseFrame {
                                    event: "message_delta".to_string(),
                                    data: json!({
                                        "type": "message_delta",
                                        "delta": {"stop_reason": stop_reason},
                                        "usage": {"output_tokens": output_tokens}
                                    })
                                    .to_string(),
                                };
                                let _ = tx.send(Ok(Bytes::from(delta_frame.to_string()))).await;

                                let stop_frame = AnthropicSseFrame {
                                    event: "message_stop".to_string(),
                                    data: json!({"type": "message_stop"}).to_string(),
                                };
                                let _ = tx.send(Ok(Bytes::from(stop_frame.to_string()))).await;
                                message_finished = true;
                            }
                        }
                    }
                }
            }
        }

        // If stream ended without [DONE], force close
        if message_started && !message_finished {
            if let Some(index) = text_block_index.take() {
                let _ = tx
                    .send(Ok(Bytes::from(
                        AnthropicSseFrame {
                            event: "content_block_stop".to_string(),
                            data: json!({"type": "content_block_stop", "index": index}).to_string(),
                        }
                        .to_string(),
                    )))
                    .await;
            }
            let _ = tx.send(Ok(Bytes::from(AnthropicSseFrame {
                event: "message_delta".to_string(),
                data: json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": output_tokens}}).to_string(),
            }.to_string()))).await;
            let _ = tx
                .send(Ok(Bytes::from(
                    AnthropicSseFrame {
                        event: "message_stop".to_string(),
                        data: json!({"type": "message_stop"}).to_string(),
                    }
                    .to_string(),
                )))
                .await;
        }

        let _ = summary_tx.send(StreamSummary {
            input_tokens: input_tokens as i64,
            output_tokens: output_tokens as i64,
            error_message,
        });
    });

    MonitoredSseStream {
        stream: Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx).map(|item| item)),
        summary: summary_rx,
    }
}

#[allow(dead_code)]
struct ToolCallState {
    index: u32,
    id: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures::stream;

    #[test]
    fn convert_request_preserves_stream_setting() {
        let provider = Provider {
            id: "provider-1".to_string(),
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com".to_string(),
            api_key_enc: "secret".to_string(),
            protocol: Protocol::OpenAI,
            model_mapping: Some("gpt-4o-mini".to_string()),
            auth_header: crate::config::provider::AuthHeader::Bearer,
            keyword: "opus".to_string(),
            enabled: true,
            sort_order: 0,
        };

        let request = json!({
            "model": "claude-opus-4-6",
            "stream": false,
            "messages": [{"role": "user", "content": "hello"}]
        });

        let converted = convert_request(&request, &provider).expect("request should convert");

        assert_eq!(converted.get("stream"), Some(&Value::Bool(false)));
        assert!(converted.get("stream_options").is_none());
    }

    #[tokio::test]
    async fn convert_sse_stream_finishes_once() {
        let openai_events = vec![
            Ok(Bytes::from(
                "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"},\"index\":0}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\",\"index\":0}],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":5}}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let monitored = convert_sse_stream(stream::iter(openai_events), "claude-opus-4-6", None);
        let output = monitored
            .stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|item| item.expect("stream item should succeed"))
            .map(|bytes| String::from_utf8(bytes.to_vec()).expect("valid utf8"))
            .collect::<String>();

        let summary = monitored.summary.await.expect("summary should resolve");

        assert_eq!(output.matches("event: message_stop").count(), 1);
        assert_eq!(output.matches("event: message_delta").count(), 1);
        assert_eq!(summary.input_tokens, 10);
        assert_eq!(summary.output_tokens, 5);
    }

    #[test]
    fn convert_error_response_maps_openai_shape() {
        let openai_error = br#"{
            "error": {
                "message": "Rate limit exceeded",
                "type": "rate_limit_error"
            }
        }"#;

        let converted = convert_error_response(openai_error, 429);

        assert_eq!(converted["type"], "error");
        assert_eq!(converted["error"]["type"], "rate_limit_error");
        assert_eq!(converted["error"]["message"], "Rate limit exceeded");
    }
}
