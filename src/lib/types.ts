export interface Provider {
  id: string;
  name: string;
  base_url: string;
  api_key_enc: string;
  protocol: "anthropic" | "openai";
  model_mapping: string | null;
  auth_header: "x-api-key" | "bearer";
  keyword: string;
  enabled: boolean;
  sort_order: number;
}

export interface RequestLog {
  id: number;
  request_id: string;
  timestamp: string;
  model: string;
  provider_id: string;
  provider_name: string;
  protocol: string;
  upstream_url: string;
  status_code: number | null;
  latency_ms: number | null;
  input_tokens: number;
  output_tokens: number;
  error_message: string | null;
  is_streaming: boolean;
}

export interface ProxyStatus {
  running: boolean;
  port: number | null;
}

export interface StatsResult {
  today: {
    date: string;
    total_requests: number;
    total_errors: number;
    total_input_tokens: number;
    total_output_tokens: number;
    avg_latency_ms: number;
  };
  provider_breakdown: Array<{
    provider_name: string;
    requests: number;
    input_tokens: number;
    output_tokens: number;
  }>;
  model_breakdown: Array<{
    model: string;
    requests: number;
    input_tokens: number;
    output_tokens: number;
  }>;
  trend: Array<{
    date: string;
    total_requests: number;
    total_errors: number;
    total_input_tokens: number;
    total_output_tokens: number;
    avg_latency_ms: number;
  }>;
}

export interface ClaudeConfigStatus {
  is_configured: boolean;
  base_url: string;
}

export interface AppSettings {
  proxy_port: number;
  auto_start_proxy: boolean;
  log_retention_days: number;
}

export interface ExportedLogsFile {
  filename: string;
  mime_type: string;
  content: string;
}
