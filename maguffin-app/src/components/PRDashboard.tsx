import React from "react";
import type { PullRequest } from "../types";
import { PullRequestCard } from "./PullRequestCard";
import { usePullRequests } from "../hooks";

interface PRDashboardProps {
  onSelectPR?: (pr: PullRequest) => void;
}

export function PRDashboard({ onSelectPR }: PRDashboardProps) {
  const { pullRequests, loading, error, refresh } = usePullRequests();
  const [filter, setFilter] = React.useState<"all" | "mine" | "review">("all");

  if (loading) {
    return (
      <div className="dashboard loading">
        <div className="spinner" />
        <p>Loading pull requests...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="dashboard error">
        <p className="error-message">Failed to load pull requests: {error}</p>
        <button onClick={refresh}>Retry</button>
      </div>
    );
  }

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>Pull Requests</h1>
        <div className="dashboard-actions">
          <div className="filter-tabs">
            <button
              className={filter === "all" ? "active" : ""}
              onClick={() => setFilter("all")}
            >
              All
            </button>
            <button
              className={filter === "mine" ? "active" : ""}
              onClick={() => setFilter("mine")}
            >
              Created by me
            </button>
            <button
              className={filter === "review" ? "active" : ""}
              onClick={() => setFilter("review")}
            >
              Needs review
            </button>
          </div>
          <button onClick={refresh} className="refresh-btn">
            ↻ Refresh
          </button>
        </div>
      </header>

      <div className="pr-list">
        {pullRequests.length === 0 ? (
          <div className="empty-state">
            <p>No pull requests found</p>
          </div>
        ) : (
          pullRequests.map((pr) => (
            <PullRequestCard key={pr.number} pr={pr} onClick={onSelectPR} />
          ))
        )}
      </div>
    </div>
  );
}
