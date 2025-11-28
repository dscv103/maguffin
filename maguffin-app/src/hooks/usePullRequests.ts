import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PullRequest } from "../types";

export function usePullRequests(baseBranch?: string) {
  const [pullRequests, setPullRequests] = useState<PullRequest[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchPullRequests = useCallback(async () => {
    try {
      setLoading(true);
      const prs = await invoke<PullRequest[]>("list_pull_requests", {
        base_branch: baseBranch,
      });
      setPullRequests(prs);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [baseBranch]);

  useEffect(() => {
    fetchPullRequests();
  }, [fetchPullRequests]);

  return {
    pullRequests,
    loading,
    error,
    refresh: fetchPullRequests,
  };
}
