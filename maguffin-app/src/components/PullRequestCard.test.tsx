import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PullRequestCard } from "./PullRequestCard";
import type { PullRequest } from "../types";

// Helper function to create a test PR
function createTestPR(overrides: Partial<PullRequest> = {}): PullRequest {
  return {
    id: "PR_123",
    number: 42,
    title: "Test PR Title",
    body: "Test body",
    state: "OPEN",
    is_draft: false,
    author: {
      login: "testuser",
      avatar_url: "https://example.com/avatar.png",
    },
    head_ref: "feature-branch",
    base_ref: "main",
    labels: [],
    review_decision: null,
    mergeable: "UNKNOWN",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    commit_count: 3,
    additions: 100,
    deletions: 50,
    changed_files: 5,
    ...overrides,
  };
}

describe("PullRequestCard", () => {
  it("renders PR number and title", () => {
    const pr = createTestPR({ number: 42, title: "My Test PR" });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("#42")).toBeInTheDocument();
    expect(screen.getByText("My Test PR")).toBeInTheDocument();
  });

  it("renders author information", () => {
    const pr = createTestPR({
      author: { login: "johndoe", avatar_url: "https://example.com/john.png" },
    });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("johndoe")).toBeInTheDocument();
    const avatar = screen.getByAltText("johndoe");
    expect(avatar).toHaveAttribute("src", "https://example.com/john.png");
  });

  it("renders branch information", () => {
    const pr = createTestPR({ head_ref: "feature", base_ref: "develop" });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("feature → develop")).toBeInTheDocument();
  });

  it("renders stats correctly", () => {
    const pr = createTestPR({
      additions: 200,
      deletions: 75,
      changed_files: 10,
      commit_count: 5,
    });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("+200")).toBeInTheDocument();
    expect(screen.getByText("-75")).toBeInTheDocument();
    expect(screen.getByText("10 files")).toBeInTheDocument();
    expect(screen.getByText("5 commits")).toBeInTheDocument();
  });

  it("shows Draft status for draft PRs", () => {
    const pr = createTestPR({ is_draft: true });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("Draft")).toBeInTheDocument();
  });

  it("shows Approved status for approved PRs", () => {
    const pr = createTestPR({ review_decision: "APPROVED" });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("Approved")).toBeInTheDocument();
  });

  it("shows Changes requested status", () => {
    const pr = createTestPR({ review_decision: "CHANGES_REQUESTED" });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("Changes requested")).toBeInTheDocument();
  });

  it("shows Conflicts status for conflicting PRs", () => {
    const pr = createTestPR({ mergeable: "CONFLICTING" });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("Conflicts")).toBeInTheDocument();
  });

  it("shows Open status for normal PRs", () => {
    const pr = createTestPR();
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText("Open")).toBeInTheDocument();
  });

  it("renders labels with correct colors", () => {
    const pr = createTestPR({
      labels: [
        { name: "bug", color: "ff0000" },
        { name: "enhancement", color: "00ff00" },
      ],
    });
    render(<PullRequestCard pr={pr} />);

    const bugLabel = screen.getByText("bug");
    expect(bugLabel).toBeInTheDocument();
    expect(bugLabel).toHaveStyle({ backgroundColor: "#ff0000" });

    const enhancementLabel = screen.getByText("enhancement");
    expect(enhancementLabel).toBeInTheDocument();
    expect(enhancementLabel).toHaveStyle({ backgroundColor: "#00ff00" });
  });

  it("does not render labels section when no labels", () => {
    const pr = createTestPR({ labels: [] });
    render(<PullRequestCard pr={pr} />);

    const labelsContainer = document.querySelector(".pr-labels");
    expect(labelsContainer).toBeNull();
  });

  it("calls onClick when clicked", () => {
    const pr = createTestPR();
    const onClick = vi.fn();
    render(<PullRequestCard pr={pr} onClick={onClick} />);

    fireEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledWith(pr);
  });

  it("calls onClick on Enter key press", () => {
    const pr = createTestPR();
    const onClick = vi.fn();
    render(<PullRequestCard pr={pr} onClick={onClick} />);

    fireEvent.keyDown(screen.getByRole("button"), { key: "Enter" });
    expect(onClick).toHaveBeenCalledWith(pr);
  });

  it("does not call onClick on other key presses", () => {
    const pr = createTestPR();
    const onClick = vi.fn();
    render(<PullRequestCard pr={pr} onClick={onClick} />);

    fireEvent.keyDown(screen.getByRole("button"), { key: "Space" });
    expect(onClick).not.toHaveBeenCalled();
  });

  it("shows 'today' for PRs updated today", () => {
    const pr = createTestPR({ updated_at: new Date().toISOString() });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText(/Updated today/)).toBeInTheDocument();
  });

  it("shows 'yesterday' for PRs updated yesterday", () => {
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    const pr = createTestPR({ updated_at: yesterday.toISOString() });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText(/Updated yesterday/)).toBeInTheDocument();
  });

  it("shows 'N days ago' for recent PRs", () => {
    const threeDaysAgo = new Date();
    threeDaysAgo.setDate(threeDaysAgo.getDate() - 3);
    const pr = createTestPR({ updated_at: threeDaysAgo.toISOString() });
    render(<PullRequestCard pr={pr} />);

    expect(screen.getByText(/Updated 3 days ago/)).toBeInTheDocument();
  });
});
