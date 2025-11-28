import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PullRequestDetails } from "../types";

export function usePullRequestDetails(number: number | null) {
  const [details, setDetails] = useState<PullRequestDetails | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchDetails = useCallback(async () => {
    if (number === null) {
      setDetails(null);
      return;
    }

    try {
      setLoading(true);
      const prDetails = await invoke<PullRequestDetails>("get_pull_request_details", {
        number,
      });
      setDetails(prDetails);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [number]);

  useEffect(() => {
    fetchDetails();
  }, [fetchDetails]);

  return {
    details,
    loading,
    error,
    refresh: fetchDetails,
  };
}
