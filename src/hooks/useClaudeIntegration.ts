import { useState, useEffect } from "react";
import { checkClaudeConfig, injectClaudeConfig, revertClaudeConfig } from "@/lib/tauri";
import type { ClaudeConfigStatus } from "@/lib/types";

export function useClaudeIntegration() {
  const [status, setStatus] = useState<ClaudeConfigStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = async () => {
    try {
      const result = await checkClaudeConfig();
      setStatus(result);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to check config");
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const inject = async () => {
    setLoading(true);
    setError(null);
    try {
      await injectClaudeConfig();
      await fetchStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to inject config");
      throw err;
    } finally {
      setLoading(false);
    }
  };

  const revert = async () => {
    setLoading(true);
    setError(null);
    try {
      await revertClaudeConfig();
      await fetchStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to revert config");
      throw err;
    } finally {
      setLoading(false);
    }
  };

  return { status, loading, error, inject, revert, refresh: fetchStatus };
}
