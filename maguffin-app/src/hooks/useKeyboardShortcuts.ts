import { useEffect, useCallback } from "react";

interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
  action: () => void;
  description: string;
}

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[], enabled = true) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;
      
      // Don't trigger shortcuts when typing in input fields
      const target = event.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        return;
      }

      for (const shortcut of shortcuts) {
        const keyMatches =
          event.key.toLowerCase() === shortcut.key.toLowerCase() ||
          event.code.toLowerCase() === shortcut.key.toLowerCase();

        const ctrlMatches = shortcut.ctrl ? event.ctrlKey || event.metaKey : !event.ctrlKey && !event.metaKey;
        const shiftMatches = shortcut.shift ? event.shiftKey : !event.shiftKey;
        const altMatches = shortcut.alt ? event.altKey : !event.altKey;

        if (keyMatches && ctrlMatches && shiftMatches && altMatches) {
          event.preventDefault();
          shortcut.action();
          return;
        }
      }
    },
    [shortcuts, enabled]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

// Pre-defined app shortcuts
export interface AppShortcuts {
  onNavigateDashboard?: () => void;
  onNavigateStacks?: () => void;
  onNavigateSettings?: () => void;
  onRefresh?: () => void;
  onToggleTheme?: () => void;
  onEscape?: () => void;
}

export function useAppKeyboardShortcuts(handlers: AppShortcuts, enabled = true) {
  const shortcuts: KeyboardShortcut[] = [];

  if (handlers.onNavigateDashboard) {
    shortcuts.push({
      key: "1",
      action: handlers.onNavigateDashboard,
      description: "Go to Pull Requests",
    });
  }

  if (handlers.onNavigateStacks) {
    shortcuts.push({
      key: "2",
      action: handlers.onNavigateStacks,
      description: "Go to Stacks",
    });
  }

  if (handlers.onNavigateSettings) {
    shortcuts.push({
      key: "3",
      action: handlers.onNavigateSettings,
      description: "Go to Settings",
    });
  }

  if (handlers.onRefresh) {
    shortcuts.push({
      key: "r",
      action: handlers.onRefresh,
      description: "Refresh data",
    });
  }

  if (handlers.onToggleTheme) {
    shortcuts.push({
      key: "t",
      action: handlers.onToggleTheme,
      description: "Toggle theme",
    });
  }

  if (handlers.onEscape) {
    shortcuts.push({
      key: "Escape",
      action: handlers.onEscape,
      description: "Close panel / Cancel",
    });
  }

  useKeyboardShortcuts(shortcuts, enabled);
}

// Export the list of available shortcuts for help display
export const AVAILABLE_SHORTCUTS = [
  { key: "1", description: "Go to Pull Requests" },
  { key: "2", description: "Go to Stacks" },
  { key: "3", description: "Go to Settings" },
  { key: "R", description: "Refresh data" },
  { key: "T", description: "Toggle theme" },
  { key: "Esc", description: "Close panel / Cancel" },
  { key: "?", description: "Show keyboard shortcuts" },
];
