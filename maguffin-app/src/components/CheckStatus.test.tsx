import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { CheckStatusDisplay } from "./CheckStatus";
import type { CheckStatus, CheckState, CheckConclusion } from "../types";

describe("CheckStatusDisplay", () => {
  describe("no checks", () => {
    it("renders 'No checks' when checkStatus is null", () => {
      render(<CheckStatusDisplay checkStatus={null} />);
      expect(screen.getByText("−")).toBeInTheDocument();
      expect(screen.getByText("No checks")).toBeInTheDocument();
    });
  });

  describe("check states", () => {
    it("renders success state correctly", () => {
      const checkStatus: CheckStatus = {
        state: "SUCCESS",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("All checks passed")).toBeInTheDocument();
      expect(screen.getByText("(1/1)")).toBeInTheDocument();
    });

    it("renders pending state correctly", () => {
      const checkStatus: CheckStatus = {
        state: "PENDING",
        checks: [
          { name: "build", status: "IN_PROGRESS", conclusion: null, details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("Checks in progress")).toBeInTheDocument();
    });

    it("renders failure state correctly", () => {
      const checkStatus: CheckStatus = {
        state: "FAILURE",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "FAILURE", details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("Some checks failed")).toBeInTheDocument();
    });

    it("renders unknown state correctly", () => {
      const checkStatus: CheckStatus = {
        state: "UNKNOWN" as CheckState,
        checks: [],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("Unknown status")).toBeInTheDocument();
    });
  });

  describe("check icons", () => {
    it("shows success icon for successful checks", () => {
      const checkStatus: CheckStatus = {
        state: "SUCCESS",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-success").length).toBeGreaterThan(0);
    });

    it("shows failure icon for failed checks", () => {
      const checkStatus: CheckStatus = {
        state: "FAILURE",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "FAILURE", details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-failure").length).toBeGreaterThan(0);
    });

    it("shows pending icon for in-progress checks", () => {
      const checkStatus: CheckStatus = {
        state: "PENDING",
        checks: [
          { name: "build", status: "IN_PROGRESS", conclusion: null, details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-pending").length).toBeGreaterThan(0);
    });

    it("shows pending icon for queued checks", () => {
      const checkStatus: CheckStatus = {
        state: "PENDING",
        checks: [
          { name: "build", status: "QUEUED", conclusion: null, details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-pending").length).toBeGreaterThan(0);
    });

    it("shows neutral icon for skipped checks", () => {
      const checkStatus: CheckStatus = {
        state: "SUCCESS",
        checks: [
          { name: "optional", status: "COMPLETED", conclusion: "SKIPPED", details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-neutral").length).toBeGreaterThan(0);
    });

    it("shows warning icon for timed out checks", () => {
      const checkStatus: CheckStatus = {
        state: "FAILURE",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "TIMED_OUT", details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-warning").length).toBeGreaterThan(0);
    });

    it("shows warning icon for cancelled checks", () => {
      const checkStatus: CheckStatus = {
        state: "FAILURE",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "CANCELLED", details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-warning").length).toBeGreaterThan(0);
    });

    it("shows warning icon for action required checks", () => {
      const checkStatus: CheckStatus = {
        state: "PENDING",
        checks: [
          { name: "review", status: "COMPLETED", conclusion: "ACTION_REQUIRED", details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-warning").length).toBeGreaterThan(0);
    });
  });

  describe("check list", () => {
    it("renders all checks in the list", () => {
      const checkStatus: CheckStatus = {
        state: "SUCCESS",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
          { name: "test", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
          { name: "lint", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("build")).toBeInTheDocument();
      expect(screen.getByText("test")).toBeInTheDocument();
      expect(screen.getByText("lint")).toBeInTheDocument();
    });

    it("renders details link when available", () => {
      const checkStatus: CheckStatus = {
        state: "SUCCESS",
        checks: [
          {
            name: "build",
            status: "COMPLETED",
            conclusion: "SUCCESS",
            details_url: "https://github.com/example/actions/123",
          },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      const link = screen.getByText("Details");
      expect(link).toBeInTheDocument();
      expect(link).toHaveAttribute("href", "https://github.com/example/actions/123");
      expect(link).toHaveAttribute("target", "_blank");
      expect(link).toHaveAttribute("rel", "noopener noreferrer");
    });

    it("does not render details link when not available", () => {
      const checkStatus: CheckStatus = {
        state: "SUCCESS",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.queryByText("Details")).not.toBeInTheDocument();
    });
  });

  describe("passed count", () => {
    it("correctly counts passed checks", () => {
      const checkStatus: CheckStatus = {
        state: "FAILURE",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
          { name: "test", status: "COMPLETED", conclusion: "FAILURE", details_url: null },
          { name: "lint", status: "COMPLETED", conclusion: "SUCCESS", details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("(2/3)")).toBeInTheDocument();
    });

    it("shows 0 passed when all fail", () => {
      const checkStatus: CheckStatus = {
        state: "FAILURE",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "FAILURE", details_url: null },
        ],
      };
      render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(screen.getByText("(0/1)")).toBeInTheDocument();
    });
  });

  describe("unknown conclusion", () => {
    it("handles unknown conclusion gracefully", () => {
      const checkStatus: CheckStatus = {
        state: "PENDING",
        checks: [
          { name: "build", status: "COMPLETED", conclusion: "UNKNOWN_CONCLUSION" as CheckConclusion, details_url: null },
        ],
      };
      const { container } = render(<CheckStatusDisplay checkStatus={checkStatus} />);
      expect(container.querySelectorAll(".check-unknown").length).toBeGreaterThan(0);
    });
  });
});
