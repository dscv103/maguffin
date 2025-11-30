import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Stack, Repository, RestackResult, ReconcileReport, RestackPreview, RebaseState } from "../types";

export function useStacks(repository: Repository | null) {
  const [stacks, setStacks] = useState<Stack[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isRebaseInProgress, setIsRebaseInProgress] = useState(false);

  const fetchStacks = useCallback(async () => {
    if (!repository) {
      setStacks([]);
      setError(null);
      return;
    }
    
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<Stack[]>("list_stacks");
      setStacks(result);
      
      // Also check if a rebase is in progress
      const rebaseInProgress = await invoke<boolean>("is_rebase_in_progress");
      setIsRebaseInProgress(rebaseInProgress);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [repository]);

  const createStack = useCallback(async (rootBranch: string): Promise<Stack | null> => {
    try {
      setError(null);
      const stack = await invoke<Stack>("create_stack", { rootBranch });
      setStacks((prev) => [...prev, stack]);
      return stack;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, []);

  const createStackBranch = useCallback(
    async (stackId: string, branchName: string, parentName: string): Promise<boolean> => {
      try {
        setError(null);
        await invoke("create_stack_branch", { stackId, branchName, parentName });
        await fetchStacks(); // Refresh to get updated stack
        return true;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return false;
      }
    },
    [fetchStacks]
  );

  const createStackPR = useCallback(
    async (
      stackId: string,
      branchName: string,
      title: string,
      body?: string,
      draft: boolean = false
    ): Promise<number | null> => {
      try {
        setError(null);
        const prNumber = await invoke<number>("create_stack_pr", {
          stack_id: stackId,
          branch_name: branchName,
          title,
          body: body ?? null,
          draft,
        });
        await fetchStacks(); // Refresh to get updated stack with PR number
        return prNumber;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return null;
      }
    },
    [fetchStacks]
  );

  const restackStack = useCallback(async (stackId: string): Promise<RestackResult | null> => {
    try {
      setError(null);
      const result = await invoke<RestackResult>("restack", { stackId });
      await fetchStacks(); // Refresh to get updated stack
      return result;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, [fetchStacks]);

  const previewRestack = useCallback(async (stackId: string): Promise<RestackPreview | null> => {
    try {
      setError(null);
      const preview = await invoke<RestackPreview>("preview_restack", { stackId });
      return preview;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, []);

  const continueRestack = useCallback(async (stackId: string): Promise<RestackResult | null> => {
    try {
      setError(null);
      const result = await invoke<RestackResult>("continue_restack", { stackId });
      await fetchStacks(); // Refresh to get updated stack
      return result;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, [fetchStacks]);

  const getRebaseState = useCallback(async (): Promise<RebaseState | null> => {
    try {
      const state = await invoke<RebaseState | null>("get_rebase_state");
      return state;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, []);

  const reconcileStacks = useCallback(async (): Promise<ReconcileReport | null> => {
    try {
      setError(null);
      const report = await invoke<ReconcileReport>("reconcile_stacks");
      await fetchStacks(); // Refresh to get updated stacks
      return report;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, [fetchStacks]);

  // Fetch stacks when repository changes
  useEffect(() => {
    if (repository) {
      fetchStacks();
    } else {
      setStacks([]);
      setLoading(false);
      setError(null);
      setIsRebaseInProgress(false);
    }
  }, [repository, fetchStacks]);

  return {
    stacks,
    loading,
    error,
    isRebaseInProgress,
    refresh: fetchStacks,
    createStack,
    createStackBranch,
    createStackPR,
    restackStack,
    previewRestack,
    continueRestack,
    getRebaseState,
    reconcileStacks,
  };
}
