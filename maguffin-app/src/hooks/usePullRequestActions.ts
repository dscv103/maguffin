import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MergeMethod } from "../types";

export interface CreatePRParams {
  title: string;
  body?: string;
  head: string;
  base: string;
  draft?: boolean;
}

export function usePullRequestActions() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createPR = useCallback(async (params: CreatePRParams): Promise<number | null> => {
    try {
      setLoading(true);
      setError(null);
      const prNumber = await invoke<number>("create_pull_request", {
        title: params.title,
        body: params.body ?? null,
        head: params.head,
        base: params.base,
        draft: params.draft ?? false,
      });
      return prNumber;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  const mergePR = useCallback(async (prId: string, method: MergeMethod = "SQUASH"): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      const merged = await invoke<boolean>("merge_pull_request", {
        pr_id: prId,
        merge_method: method,
      });
      return merged;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const closePR = useCallback(async (prId: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      const closed = await invoke<boolean>("close_pull_request", {
        pr_id: prId,
      });
      return closed;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const checkoutPR = useCallback(async (number: number): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke("checkout_pull_request", { number });
      return true;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    loading,
    error,
    createPR,
    mergePR,
    closePR,
    checkoutPR,
    clearError,
  };
}
