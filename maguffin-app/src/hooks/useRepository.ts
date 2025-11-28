import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Repository } from "../types";

export function useRepository() {
  const [repository, setRepository] = useState<Repository | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openRepository = useCallback(async (path: string): Promise<Repository | null> => {
    try {
      setLoading(true);
      setError(null);
      const repo = await invoke<Repository>("open_repository", { path });
      setRepository(repo);
      return repo;
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const clearRepository = useCallback(() => {
    setRepository(null);
    setError(null);
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    repository,
    loading,
    error,
    openRepository,
    clearRepository,
    clearError,
  };
}
