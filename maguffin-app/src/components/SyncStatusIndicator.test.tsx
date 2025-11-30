import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SyncStatusIndicator } from "./SyncStatusIndicator";
import type { SyncStatus } from "../types";

describe("SyncStatusIndicator", () => {
  describe("idle status", () => {
    it("renders idle status with last sync time", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: new Date().toISOString(),
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText("✓")).toBeInTheDocument();
      expect(screen.getByText(/Last synced:/)).toBeInTheDocument();
    });

    it("renders idle status without sync history", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: null,
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText("✓")).toBeInTheDocument();
      expect(screen.getByText("Not synced yet")).toBeInTheDocument();
    });

    it("shows 'just now' for recent sync", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: new Date().toISOString(),
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText(/just now/)).toBeInTheDocument();
    });

    it("shows minutes ago for older sync", () => {
      const fiveMinutesAgo = new Date(Date.now() - 5 * 60 * 1000).toISOString();
      const status: SyncStatus = {
        status: "idle",
        last_sync: fiveMinutesAgo,
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText(/5 minutes ago/)).toBeInTheDocument();
    });

    it("shows hours ago for much older sync", () => {
      const twoHoursAgo = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString();
      const status: SyncStatus = {
        status: "idle",
        last_sync: twoHoursAgo,
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText(/2 hours ago/)).toBeInTheDocument();
    });
  });

  describe("in_progress status", () => {
    it("renders syncing indicator", () => {
      const status: SyncStatus = {
        status: "in_progress",
        started_at: new Date().toISOString(),
        current_task: null,
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText("↻")).toBeInTheDocument();
      expect(screen.getByText("Syncing...")).toBeInTheDocument();
    });

    it("renders current task when provided", () => {
      const status: SyncStatus = {
        status: "in_progress",
        started_at: new Date().toISOString(),
        current_task: "Fetching pull requests",
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText("Fetching pull requests")).toBeInTheDocument();
    });

    it("does not show sync button during sync", () => {
      const status: SyncStatus = {
        status: "in_progress",
        started_at: new Date().toISOString(),
        current_task: null,
      };
      const onSyncNow = vi.fn();
      render(<SyncStatusIndicator status={status} onSyncNow={onSyncNow} />);
      expect(screen.queryByTitle("Sync now")).not.toBeInTheDocument();
    });
  });

  describe("failed status", () => {
    it("renders failed status with error message", () => {
      const status: SyncStatus = {
        status: "failed",
        error: "Network error",
        failed_at: new Date().toISOString(),
        failure_count: 1,
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText("✗")).toBeInTheDocument();
      expect(screen.getByText(/Sync failed: Network error/)).toBeInTheDocument();
    });
  });

  describe("rate_limited status", () => {
    it("renders rate limited status", () => {
      const status: SyncStatus = {
        status: "rate_limited",
        resets_at: new Date(Date.now() + 60000).toISOString(),
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.getByText("⏱")).toBeInTheDocument();
      expect(screen.getByText(/Rate limited until/)).toBeInTheDocument();
    });
  });

  describe("sync button", () => {
    it("renders sync button when onSyncNow is provided", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: null,
      };
      const onSyncNow = vi.fn();
      render(<SyncStatusIndicator status={status} onSyncNow={onSyncNow} />);
      expect(screen.getByTitle("Sync now")).toBeInTheDocument();
    });

    it("does not render sync button when onSyncNow is not provided", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: null,
      };
      render(<SyncStatusIndicator status={status} />);
      expect(screen.queryByTitle("Sync now")).not.toBeInTheDocument();
    });

    it("calls onSyncNow when button is clicked", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: null,
      };
      const onSyncNow = vi.fn();
      render(<SyncStatusIndicator status={status} onSyncNow={onSyncNow} />);
      fireEvent.click(screen.getByTitle("Sync now"));
      expect(onSyncNow).toHaveBeenCalledOnce();
    });

    it("disables sync button when loading", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: null,
      };
      const onSyncNow = vi.fn();
      render(
        <SyncStatusIndicator
          status={status}
          onSyncNow={onSyncNow}
          loading={true}
        />
      );
      expect(screen.getByTitle("Sync now")).toBeDisabled();
    });

    it("disables sync button when rate limited", () => {
      const status: SyncStatus = {
        status: "rate_limited",
        resets_at: new Date(Date.now() + 60000).toISOString(),
      };
      const onSyncNow = vi.fn();
      render(<SyncStatusIndicator status={status} onSyncNow={onSyncNow} />);
      expect(screen.getByTitle("Sync now")).toBeDisabled();
    });
  });

  describe("CSS classes", () => {
    it("applies sync-idle class for idle status", () => {
      const status: SyncStatus = {
        status: "idle",
        last_sync: null,
      };
      const { container } = render(<SyncStatusIndicator status={status} />);
      expect(container.firstChild).toHaveClass("sync-idle");
    });

    it("applies sync-in-progress class for in_progress status", () => {
      const status: SyncStatus = {
        status: "in_progress",
        started_at: new Date().toISOString(),
        current_task: null,
      };
      const { container } = render(<SyncStatusIndicator status={status} />);
      expect(container.firstChild).toHaveClass("sync-in-progress");
    });

    it("applies sync-failed class for failed status", () => {
      const status: SyncStatus = {
        status: "failed",
        error: "Error",
        failed_at: new Date().toISOString(),
        failure_count: 1,
      };
      const { container } = render(<SyncStatusIndicator status={status} />);
      expect(container.firstChild).toHaveClass("sync-failed");
    });

    it("applies sync-rate-limited class for rate_limited status", () => {
      const status: SyncStatus = {
        status: "rate_limited",
        resets_at: new Date().toISOString(),
      };
      const { container } = render(<SyncStatusIndicator status={status} />);
      expect(container.firstChild).toHaveClass("sync-rate-limited");
    });
  });
});
