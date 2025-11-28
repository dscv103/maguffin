import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SyncStatus } from "../types";

/**
 * Hook for managing background sync service.
 */
export function useSync() {
  const [status, setStatus] = useState<SyncStatus>({ status: "idle", last_sync: null });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch current sync status
  const fetchStatus = useCallback(async () => {
    try {
      const syncStatus = await invoke<SyncStatus>("get_sync_status");
      setStatus(syncStatus);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  // Start background sync
  const startSync = useCallback(async () => {
    setLoading(true);
    try {
      await invoke("start_sync");
      await fetchStatus();
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [fetchStatus]);

  // Stop background sync
  const stopSync = useCallback(async () => {
    setLoading(true);
    try {
      await invoke("stop_sync");
      await fetchStatus();
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [fetchStatus]);

  // Trigger immediate sync
  const syncNow = useCallback(async () => {
    setLoading(true);
    try {
      await invoke("sync_now");
      // Fetch status after triggering sync
      await fetchStatus();
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [fetchStatus]);

  // Update sync configuration
  const updateConfig = useCallback(
    async (intervalSecs: number, enabled: boolean) => {
      setLoading(true);
      try {
        await invoke("update_sync_config", {
          interval_secs: intervalSecs,
          enabled,
        });
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    },
    []
  );

  // Clear error
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Fetch status on mount and periodically
  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000); // Poll every 5 seconds
    return () => clearInterval(interval);
  }, [fetchStatus]);

  return {
    status,
    loading,
    error,
    startSync,
    stopSync,
    syncNow,
    updateConfig,
    clearError,
    refetch: fetchStatus,
  };
}
