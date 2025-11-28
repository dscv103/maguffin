import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Stack } from "../types";

export function useStacks() {
  const [stacks, setStacks] = useState<Stack[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchStacks = useCallback(async () => {
    try {
      setLoading(true);
      const result = await invoke<Stack[]>("list_stacks");
      setStacks(result);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const createStack = useCallback(async (rootBranch: string): Promise<Stack | null> => {
    try {
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
      const result = await invoke("restack", { stackId });
      await fetchStacks(); // Refresh to get updated stack
      return result;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, [fetchStacks]);

  useEffect(() => {
    fetchStacks();
  }, [fetchStacks]);

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
