import { useState, useEffect } from "react";
import { startProxy, stopProxy, getProxyStatus } from "@/lib/tauri";
import type { ProxyStatus } from "@/lib/types";

export function useProxyServer() {
  const [status, setStatus] = useState<ProxyStatus>({ running: false, port: null });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = async () => {
    try {
      const result = await getProxyStatus();
      setStatus(result);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch status");
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  const start = async () => {
    setLoading(true);
    setError(null);
    try {
      const port = await startProxy();
      setStatus({ running: true, port });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to start proxy");
      throw err;
    } finally {
      setLoading(false);
    }
  };

  const stop = async () => {
    setLoading(true);
    setError(null);
    try {
      await stopProxy();
      setStatus({ running: false, port: null });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to stop proxy");
      throw err;
    } finally {
      setLoading(false);
    }
  };

  return { status, loading, error, start, stop, refresh: fetchStatus };
}
