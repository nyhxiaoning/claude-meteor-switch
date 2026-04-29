import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  ExportedLogsFile,
  Provider,
  RequestLog,
  ProxyStatus,
  StatsResult,
  ClaudeConfigStatus,
} from "./types";

export async function startProxy(): Promise<number> {
  return invoke("start_proxy");
}

export async function stopProxy(): Promise<void> {
  return invoke("stop_proxy");
}

export async function getProxyStatus(): Promise<ProxyStatus> {
  return invoke("proxy_status");
}

export async function listProviders(): Promise<Provider[]> {
  return invoke("list_providers");
}

export async function createProvider(params: {
  name: string;
  base_url: string;
  api_key: string;
  protocol: string;
  model_mapping?: string;
  auth_header: string;
  keyword: string;
  enabled: boolean;
}): Promise<Provider> {
  return invoke("create_provider", {
    name: params.name,
    baseUrl: params.base_url,
    apiKey: params.api_key,
    protocol: params.protocol,
    modelMapping: params.model_mapping,
    authHeader: params.auth_header,
    keyword: params.keyword,
    enabled: params.enabled,
  });
}

export async function updateProvider(params: {
  id: string;
  name: string;
  base_url: string;
  api_key?: string;
  protocol: string;
  model_mapping?: string;
  auth_header: string;
  keyword: string;
  enabled: boolean;
}): Promise<void> {
  return invoke("update_provider", {
    id: params.id,
    name: params.name,
    baseUrl: params.base_url,
    apiKey: params.api_key,
    protocol: params.protocol,
    modelMapping: params.model_mapping,
    authHeader: params.auth_header,
    keyword: params.keyword,
    enabled: params.enabled,
  });
}

export async function deleteProvider(id: string): Promise<void> {
  return invoke("delete_provider", { id });
}

export async function getLogs(params?: {
  provider_id?: string;
  model?: string;
  status_code?: number;
  date_from?: string;
  date_to?: string;
  page?: number;
  page_size?: number;
}): Promise<{ logs: RequestLog[]; total: number; page: number; page_size: number }> {
  if (!params) return invoke("get_logs", {});

  return invoke("get_logs", {
    providerId: params.provider_id,
    model: params.model,
    statusCode: params.status_code,
    dateFrom: params.date_from,
    dateTo: params.date_to,
    page: params.page,
    pageSize: params.page_size,
  });
}

export async function exportLogs(params: {
  format: "json" | "csv";
  provider_id?: string;
  model?: string;
  status_code?: number;
  date_from?: string;
  date_to?: string;
}): Promise<ExportedLogsFile> {
  return invoke("export_logs", {
    format: params.format,
    providerId: params.provider_id,
    model: params.model,
    statusCode: params.status_code,
    dateFrom: params.date_from,
    dateTo: params.date_to,
  });
}

export async function getStats(): Promise<StatsResult> {
  return invoke("get_stats");
}

export async function injectClaudeConfig(): Promise<void> {
  return invoke("inject_claude_config");
}

export async function revertClaudeConfig(): Promise<void> {
  return invoke("revert_claude_config");
}

export async function checkClaudeConfig(): Promise<ClaudeConfigStatus> {
  return invoke("check_claude_config");
}

export async function getAppSettings(): Promise<AppSettings> {
  return invoke("get_app_settings");
}

export async function updateAppSettings(params: AppSettings): Promise<AppSettings> {
  return invoke("update_app_settings", {
    proxyPort: params.proxy_port,
    autoStartProxy: params.auto_start_proxy,
    logRetentionDays: params.log_retention_days,
  });
}

export async function isAutostartEnabled(): Promise<boolean> {
  return invoke("is_autostart_enabled");
}

export async function enableAutostart(): Promise<void> {
  return invoke("enable_autostart");
}

export async function disableAutostart(): Promise<void> {
  return invoke("disable_autostart");
}
