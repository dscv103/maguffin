import React from "react";
import type { Stack, StackBranch } from "../types";

interface StackViewProps {
  stack: Stack;
  onBranchClick?: (branch: StackBranch) => void;
  onRestack?: (stack: Stack) => void;
}

export function StackView({ stack, onBranchClick, onRestack }: StackViewProps) {
  const getStatusIcon = (status: StackBranch["status"]): string => {
    switch (status) {
      case "up_to_date":
        return "✓";
      case "needs_rebase":
        return "↻";
      case "conflicted":
        return "⚠";
      case "orphaned":
        return "✗";
      default:
        return "?";
    }
  };

  const getStatusColor = (status: StackBranch["status"]): string => {
    switch (status) {
      case "up_to_date":
        return "green";
      case "needs_rebase":
        return "yellow";
      case "conflicted":
        return "red";
      case "orphaned":
        return "gray";
      default:
        return "gray";
    }
  };

  const needsRestack = stack.branches.some(
    (b) => b.status === "needs_rebase" || b.status === "conflicted"
  );

  // Build tree structure
  const buildTree = () => {
    const nodes: React.ReactNode[] = [];
    const visited = new Set<string>();

    const renderBranch = (branchName: string, depth: number = 0) => {
      const branch = stack.branches.find((b) => b.name === branchName);
      if (!branch || visited.has(branchName)) return null;
      visited.add(branchName);

      const children = stack.branches.filter((b) => b.parent === branchName);

      return (
        <div
          key={branchName}
          className="stack-branch"
          style={{ marginLeft: `${depth * 20}px` }}
        >
          <div
            className="branch-content"
            onClick={() => branch && onBranchClick?.(branch)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === "Enter" && branch && onBranchClick?.(branch)}
          >
            <span
              className={`branch-status status-${getStatusColor(branch.status)}`}
            >
              {getStatusIcon(branch.status)}
            </span>
            <span className="branch-name">{branch.name}</span>
            {branch.pr_number && (
              <span className="branch-pr">#{branch.pr_number}</span>
            )}
          </div>
          {children.map((child) => renderBranch(child.name, depth + 1))}
        </div>
      );
    };

    // Start from branches whose parent is the root
    const rootChildren = stack.branches.filter((b) => b.parent === stack.root);
    rootChildren.forEach((child) => {
      nodes.push(renderBranch(child.name, 0));
    });

    return nodes;
  };

  return (
    <div className="stack-view">
      <header className="stack-header">
        <div className="stack-root">
          <span className="root-icon">⬤</span>
          <span className="root-name">{stack.root}</span>
        </div>
        {needsRestack && (
          <button
            className="restack-btn"
            onClick={() => onRestack?.(stack)}
          >
            Restack
          </button>
        )}
      </header>

      <div className="stack-tree">{buildTree()}</div>
    </div>
  );
}

interface StackListProps {
  stacks: Stack[];
  onStackSelect?: (stack: Stack) => void;
  onBranchClick?: (branch: StackBranch) => void;
  onRestack?: (stack: Stack) => void;
}

export function StackList({
  stacks,
  onStackSelect: _onStackSelect,
  onBranchClick,
  onRestack,
}: StackListProps) {
  if (stacks.length === 0) {
    return (
      <div className="stack-list empty">
        <p>No stacks found</p>
        <p className="hint">Create a stack to organize your branches</p>
      </div>
    );
  }

  return (
    <div className="stack-list">
      {stacks.map((stack) => (
        <StackView
          key={stack.id}
          stack={stack}
          onBranchClick={onBranchClick}
          onRestack={onRestack}
        />
      ))}
    </div>
  );
}
