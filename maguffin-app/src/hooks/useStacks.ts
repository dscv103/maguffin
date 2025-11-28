import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Stack, Repository } from "../types";

export function useStacks(repository: Repository | null) {
  const [stacks, setStacks] = useState<Stack[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  const restackStack = useCallback(async (stackId: string) => {
    try {
      setError(null);
      const result = await invoke("restack", { stackId });
      await fetchStacks(); // Refresh to get updated stack
      return result;
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
    }
  }, [repository, fetchStacks]);

  return {
    stacks,
    loading,
    error,
    refresh: fetchStacks,
    createStack,
    createStackBranch,
    restackStack,
  };
}
