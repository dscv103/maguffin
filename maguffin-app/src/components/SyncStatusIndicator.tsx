import { useMemo } from "react";
import type { SyncStatus } from "../types";

interface SyncStatusIndicatorProps {
  status: SyncStatus;
  onSyncNow?: () => void;
  loading?: boolean;
}

/**
 * Visual indicator for background sync status.
 */
export function SyncStatusIndicator({
  status,
  onSyncNow,
  loading = false,
}: SyncStatusIndicatorProps) {
  const { icon, label, className } = useMemo(() => {
    switch (status.status) {
      case "idle":
        return {
          icon: "✓",
          label: status.last_sync
            ? `Last synced: ${formatRelativeTime(status.last_sync)}`
            : "Not synced yet",
          className: "sync-idle",
        };
      case "in_progress":
        return {
          icon: "↻",
          label: status.current_task || "Syncing...",
          className: "sync-in-progress",
        };
      case "failed":
        return {
          icon: "✗",
          label: `Sync failed: ${status.error}`,
          className: "sync-failed",
        };
      case "rate_limited":
        return {
          icon: "⏱",
          label: `Rate limited until ${formatTime(status.resets_at)}`,
          className: "sync-rate-limited",
        };
      default: {
        // This case should never be reached - all status values are handled above
        return {
          icon: "?",
          label: "Unknown status",
          className: "sync-unknown",
        };
      }
    }
  }, [status]);

  return (
    <div className={`sync-status-indicator ${className}`}>
      <span className="sync-icon">{icon}</span>
      <span className="sync-label">{label}</span>
      {onSyncNow && status.status !== "in_progress" && (
        <button
          className="sync-now-btn"
          onClick={onSyncNow}
          disabled={loading || status.status === "rate_limited"}
          title="Sync now"
        >
          ↻
        </button>
      )}
    </div>
  );
}

/**
 * Format a date string as a relative time (e.g., "2 minutes ago").
 */
function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);

  if (diffSecs < 60) {
    return "just now";
  } else if (diffMins < 60) {
    return `${diffMins} minute${diffMins === 1 ? "" : "s"} ago`;
  } else if (diffHours < 24) {
    return `${diffHours} hour${diffHours === 1 ? "" : "s"} ago`;
  } else {
    return date.toLocaleString();
  }
}

/**
 * Format a date string as a time (e.g., "3:45 PM").
 */
function formatTime(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleTimeString();
}
