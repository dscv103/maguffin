import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AuthState } from "../types";

export function useAuth() {
  const [authState, setAuthState] = useState<AuthState>({ type: "unauthenticated" });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchAuthState = useCallback(async () => {
    try {
      setLoading(true);
      const state = await invoke<AuthState>("get_auth_state");
      setAuthState(state);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const startDeviceFlow = useCallback(async () => {
    try {
      setLoading(true);
      const state = await invoke<AuthState>("start_device_flow");
      setAuthState(state);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const pollDeviceFlow = useCallback(async () => {
    try {
      const state = await invoke<AuthState>("poll_device_flow");
      setAuthState(state);
      return state;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return authState;
    }
  }, [authState]);

  const logout = useCallback(async () => {
    try {
      await invoke("logout");
      setAuthState({ type: "unauthenticated" });
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    fetchAuthState();
  }, [fetchAuthState]);

  return {
    authState,
    loading,
    error,
    startDeviceFlow,
    pollDeviceFlow,
    logout,
    refresh: fetchAuthState,
  };
}
