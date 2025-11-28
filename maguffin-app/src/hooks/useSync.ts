import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SyncStatus, SyncConfig } from "../types";

/**
 * Hook for managing background sync service.
 */
export function useSync() {
  const [status, setStatus] = useState<SyncStatus>({ status: "idle", last_sync: null });
  const [config, setConfig] = useState<SyncConfig>({ interval_secs: 60, enabled: true, sync_on_startup: true });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Track if sync has been started to prevent multiple starts
  const syncStartedRef = useRef(false);

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

  // Start background sync - only starts once
  const startSync = useCallback(async () => {
    if (syncStartedRef.current) {
      return; // Already started
    }
    syncStartedRef.current = true;
    setLoading(true);
    try {
      await invoke("start_sync");
      await fetchStatus();
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      syncStartedRef.current = false; // Allow retry on error
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
        setConfig(prev => ({ ...prev, interval_secs: intervalSecs, enabled }));
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
    config,
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
