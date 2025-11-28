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

/**
 * Check if modifier key matches the shortcut requirement.
 * If shortcut requires the modifier, check if it's pressed (or meta on Mac for Ctrl).
 * If shortcut doesn't require the modifier, ensure it's not pressed.
 */
function modifierMatches(
  required: boolean | undefined,
  pressed: boolean,
  metaPressed = false
): boolean {
  if (required) {
    // For Ctrl, also accept Cmd (meta) on Mac
    return pressed || metaPressed;
  }
  // When not required, ensure neither the key nor meta is pressed
  return !pressed && !metaPressed;
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

        // Check modifiers - for Ctrl, also accept Cmd on Mac
        const ctrlMatches = modifierMatches(shortcut.ctrl, event.ctrlKey, event.metaKey);
        const shiftMatches = modifierMatches(shortcut.shift, event.shiftKey);
        const altMatches = modifierMatches(shortcut.alt, event.altKey);

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
