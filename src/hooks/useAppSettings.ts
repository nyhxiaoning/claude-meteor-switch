import { useState, useEffect } from "react";
import { getAppSettings, updateAppSettings } from "@/lib/tauri";
import type { AppSettings } from "@/lib/types";

export function useAppSettings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSettings = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getAppSettings();
      setSettings(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch settings");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const update = async (params: AppSettings) => {
    setLoading(true);
    setError(null);
    try {
      const result = await updateAppSettings(params);
      setSettings(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update settings");
      throw err;
    } finally {
      setLoading(false);
    }
  };

  return { settings, loading, error, update, refresh: fetchSettings };
}
