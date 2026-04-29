use crate::db::logs;
use crate::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_logs(
    state: State<'_, Arc<AppState>>,
    provider_id: Option<String>,
    model: Option<String>,
    status_code: Option<i32>,
    date_from: Option<String>,
    date_to: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<serde_json::Value, String> {
    let filter = logs::LogFilter {
        provider_id,
        model,
        status_code,
        date_from,
        date_to,
        page: page.unwrap_or(1),
        page_size: page_size.unwrap_or(50),
    };

    let (logs_list, total) = logs::query_logs(&state.db, &filter).await?;

    Ok(serde_json::json!({
        "logs": logs_list,
        "total": total,
        "page": filter.page,
        "page_size": filter.page_size
    }))
}

#[tauri::command]
pub async fn export_logs(
    state: State<'_, Arc<AppState>>,
    format: String,
    provider_id: Option<String>,
    model: Option<String>,
    status_code: Option<i32>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<serde_json::Value, String> {
    let filter = logs::LogFilter {
        provider_id,
        model,
        status_code,
        date_from,
        date_to,
        page: 1,
        page_size: 50,
    };

    let exported = logs::query_logs_all(&state.db, &filter).await?;

    match format.as_str() {
        "json" => Ok(serde_json::json!({
            "filename": format!("claude-dynamic-meteor-logs-{}.json", chrono::Local::now().format("%Y%m%d-%H%M%S")),
            "mime_type": "application/json",
            "content": serde_json::to_string_pretty(&exported).map_err(|e| e.to_string())?
        })),
        "csv" => {
            let mut lines = vec![
                "id,request_id,timestamp,model,provider_id,provider_name,protocol,upstream_url,status_code,latency_ms,input_tokens,output_tokens,error_message,is_streaming".to_string()
            ];

            for log in exported {
                let escape = |value: &str| format!("\"{}\"", value.replace('"', "\"\""));
                lines.push(format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    log.id,
                    escape(&log.request_id),
                    escape(&log.timestamp),
                    escape(&log.model),
                    escape(&log.provider_id),
                    escape(&log.provider_name),
                    escape(&log.protocol),
                    escape(&log.upstream_url),
                    log.status_code.map(|v| v.to_string()).unwrap_or_default(),
                    log.latency_ms.map(|v| v.to_string()).unwrap_or_default(),
                    log.input_tokens,
                    log.output_tokens,
                    escape(log.error_message.as_deref().unwrap_or("")),
                    log.is_streaming
                ));
            }

            Ok(serde_json::json!({
                "filename": format!("claude-dynamic-meteor-logs-{}.csv", chrono::Local::now().format("%Y%m%d-%H%M%S")),
                "mime_type": "text/csv;charset=utf-8",
                "content": lines.join("\n")
            }))
        }
        _ => Err("Unsupported export format".to_string()),
    }
}
