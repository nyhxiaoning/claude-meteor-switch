import { useState, useEffect } from "react";
import { getLogs } from "@/lib/tauri";
import type { RequestLog } from "@/lib/types";

interface UseLogsParams {
  provider_id?: string;
  model?: string;
  status_code?: number;
  date_from?: string;
  date_to?: string;
  page?: number;
  page_size?: number;
}

export function useLogs(params?: UseLogsParams) {
  const [logs, setLogs] = useState<RequestLog[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(params?.page || 1);
  const [pageSize, setPageSize] = useState(params?.page_size || 20);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchLogs = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getLogs({ ...params, page, page_size: pageSize });
      setLogs(result.logs);
      setTotal(result.total);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch logs");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchLogs();
  }, [page, pageSize, params?.provider_id, params?.model, params?.status_code, params?.date_from, params?.date_to]);

  return { logs, total, page, pageSize, loading, error, setPage, setPageSize, refresh: fetchLogs };
}
