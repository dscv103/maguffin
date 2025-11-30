import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PrTemplate, TemplateContext } from "../types";

export function useTemplates() {
  const [templates, setTemplates] = useState<PrTemplate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchTemplates = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<PrTemplate[]>("get_templates");
      setTemplates(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const getTemplate = useCallback(async (id: string): Promise<PrTemplate | null> => {
    try {
      setError(null);
      const template = await invoke<PrTemplate | null>("get_template", { id });
      return template;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, []);

  const getDefaultTemplate = useCallback(async (): Promise<PrTemplate | null> => {
    try {
      setError(null);
      const template = await invoke<PrTemplate | null>("get_default_template");
      return template;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, []);

  const createTemplate = useCallback(
    async (name: string, body: string, isDefault: boolean = false): Promise<PrTemplate | null> => {
      try {
        setError(null);
        const template = await invoke<PrTemplate>("create_template", {
          name,
          body,
          is_default: isDefault,
        });
        await fetchTemplates();
        return template;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return null;
      }
    },
    [fetchTemplates]
  );

  const updateTemplate = useCallback(
    async (
      id: string,
      name: string,
      body: string,
      isDefault: boolean
    ): Promise<PrTemplate | null> => {
      try {
        setError(null);
        const template = await invoke<PrTemplate>("update_template", {
          id,
          name,
          body,
          is_default: isDefault,
        });
        await fetchTemplates();
        return template;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return null;
      }
    },
    [fetchTemplates]
  );

  const deleteTemplate = useCallback(
    async (id: string): Promise<boolean> => {
      try {
        setError(null);
        const deleted = await invoke<boolean>("delete_template", { id });
        if (deleted) {
          await fetchTemplates();
        }
        return deleted;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return false;
      }
    },
    [fetchTemplates]
  );

  const renderTemplate = useCallback(
    async (id: string, context: TemplateContext): Promise<string | null> => {
      try {
        setError(null);
        const rendered = await invoke<string>("render_template", { id, context });
        return rendered;
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return null;
      }
    },
    []
  );

  // Fetch templates on mount
  useEffect(() => {
    fetchTemplates();
  }, [fetchTemplates]);

  return {
    templates,
    loading,
    error,
    refresh: fetchTemplates,
    getTemplate,
    getDefaultTemplate,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    renderTemplate,
  };
}
