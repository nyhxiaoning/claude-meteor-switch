import { useState, useEffect } from "react";
import { getStats } from "@/lib/tauri";
import type { StatsResult } from "@/lib/types";

export function useStats() {
  const [stats, setStats] = useState<StatsResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getStats();
      setStats(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch stats");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
    const interval = setInterval(fetchStats, 10000);
    return () => clearInterval(interval);
  }, []);

  return { stats, loading, error, refresh: fetchStats };
}
