import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Repository, RecentRepository } from "../types";

export function useRepository() {
  const [repository, setRepository] = useState<Repository | null>(null);
  const [recentRepositories, setRecentRepositories] = useState<RecentRepository[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load recent repositories on mount
  useEffect(() => {
    const loadRecentRepos = async () => {
      try {
        const recent = await invoke<RecentRepository[]>("get_recent_repositories");
        setRecentRepositories(recent);
      } catch (e) {
        // Silently fail - recent repos are not critical
        console.error("Failed to load recent repositories:", e);
      }
    };
    loadRecentRepos();
  }, []);

  const openRepository = useCallback(async (path: string): Promise<Repository | null> => {
    try {
      setLoading(true);
      setError(null);
      const repo = await invoke<Repository>("open_repository", { path });
      setRepository(repo);
      
      // Refresh recent repositories
      try {
        const recent = await invoke<RecentRepository[]>("get_recent_repositories");
        setRecentRepositories(recent);
      } catch {
        // Silently fail
      }
      
      return repo;
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const removeRecentRepository = useCallback(async (path: string) => {
    try {
      await invoke("remove_recent_repository", { path });
      setRecentRepositories(prev => prev.filter(r => r.path !== path));
    } catch (e) {
      console.error("Failed to remove recent repository:", e);
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
    recentRepositories,
    loading,
    error,
    openRepository,
    removeRecentRepository,
    clearRepository,
    clearError,
  };
}
