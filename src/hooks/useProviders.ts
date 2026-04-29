import { useState, useEffect } from "react";
import { listProviders, createProvider, updateProvider, deleteProvider } from "@/lib/tauri";
import type { Provider } from "@/lib/types";

export function useProviders() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProviders = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listProviders();
      setProviders(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch providers");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchProviders();
  }, []);

  const create = async (params: {
    name: string;
    base_url: string;
    api_key: string;
    protocol: string;
    model_mapping?: string;
    auth_header: string;
    keyword: string;
    enabled: boolean;
  }) => {
    setError(null);
    try {
      await createProvider(params);
      await fetchProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create provider");
      throw err;
    }
  };

  const update = async (params: {
    id: string;
    name: string;
    base_url: string;
    api_key?: string;
    protocol: string;
    model_mapping?: string;
    auth_header: string;
    keyword: string;
    enabled: boolean;
  }) => {
    setError(null);
    try {
      await updateProvider(params);
      await fetchProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update provider");
      throw err;
    }
  };

  const remove = async (id: string) => {
    setError(null);
    try {
      await deleteProvider(id);
      await fetchProviders();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete provider");
      throw err;
    }
  };

  return { providers, loading, error, create, update, remove, refresh: fetchProviders };
}
