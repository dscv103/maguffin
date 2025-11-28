import React from "react";
import type { PullRequest } from "../types";
import { PullRequestCard } from "./PullRequestCard";
import { usePullRequests } from "../hooks";

interface PRDashboardProps {
  onSelectPR?: (pr: PullRequest) => void;
}

type SortField = "updated" | "created" | "title" | "comments";
type SortDirection = "asc" | "desc";

export function PRDashboard({ onSelectPR }: PRDashboardProps) {
  const { pullRequests, loading, error, refresh } = usePullRequests();
  const [filter, setFilter] = React.useState<"all" | "mine" | "review">("all");
  const [sortField, setSortField] = React.useState<SortField>("updated");
  const [sortDirection, setSortDirection] = React.useState<SortDirection>("desc");

  // Sort pull requests based on current sort settings
  const sortedPRs = React.useMemo(() => {
    if (!pullRequests.length) return pullRequests;

    return [...pullRequests].sort((a, b) => {
      let comparison = 0;
      
      switch (sortField) {
        case "updated":
          comparison = new Date(a.updated_at).getTime() - new Date(b.updated_at).getTime();
          break;
        case "created":
          comparison = new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
          break;
        case "title":
          comparison = a.title.localeCompare(b.title);
          break;
        case "comments":
          // Sort by number of commits as a proxy for activity
          comparison = a.commit_count - b.commit_count;
          break;
      }

      return sortDirection === "desc" ? -comparison : comparison;
    });
  }, [pullRequests, sortField, sortDirection]);

  const handleSortChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setSortField(e.target.value as SortField);
  };

  const toggleSortDirection = () => {
    setSortDirection(prev => prev === "desc" ? "asc" : "desc");
  };

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
          <div className="sort-controls">
            <select
              value={sortField}
              onChange={handleSortChange}
              className="sort-select"
              aria-label="Sort by"
            >
              <option value="updated">Updated</option>
              <option value="created">Created</option>
              <option value="title">Title</option>
              <option value="comments">Activity</option>
            </select>
            <button
              onClick={toggleSortDirection}
              className="sort-direction-btn"
              aria-label={`Sort ${sortDirection === "desc" ? "descending" : "ascending"}`}
              title={`Sort ${sortDirection === "desc" ? "newest first" : "oldest first"}`}
            >
              {sortDirection === "desc" ? "↓" : "↑"}
            </button>
          </div>
          <button onClick={refresh} className="refresh-btn">
            ↻ Refresh
          </button>
        </div>
      </header>

      <div className="pr-list">
        {sortedPRs.length === 0 ? (
          <div className="empty-state">
            <p>No pull requests found</p>
          </div>
        ) : (
          sortedPRs.map((pr) => (
            <PullRequestCard key={pr.number} pr={pr} onClick={onSelectPR} />
          ))
        )}
      </div>
    </div>
  );
}
