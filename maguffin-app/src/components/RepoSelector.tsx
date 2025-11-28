import React, { useState } from "react";
import type { Repository, RecentRepository } from "../types";

interface RepoSelectorProps {
  repository: Repository | null;
  recentRepositories?: RecentRepository[];
  loading: boolean;
  error: string | null;
  onOpenRepository: (path: string) => Promise<Repository | null>;
  onClearRepository: () => void;
  onRemoveRecentRepository?: (path: string) => void;
  onClearError?: () => void;
}

export function RepoSelector({
  repository,
  recentRepositories = [],
  loading,
  error,
  onOpenRepository,
  onClearRepository,
  onRemoveRecentRepository,
  onClearError,
}: RepoSelectorProps) {
  const [path, setPath] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [showRecent, setShowRecent] = useState(false);

  const handlePathChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setPath(e.target.value);
    // Clear error when user starts typing a new path
    if (error && onClearError) {
      onClearError();
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (path.trim()) {
      const result = await onOpenRepository(path.trim());
      if (result) {
        setIsEditing(false);
        setPath("");
        setShowRecent(false);
      }
    }
  };

  const handleOpenRecent = async (recentPath: string) => {
    const result = await onOpenRepository(recentPath);
    if (result) {
      setIsEditing(false);
      setPath("");
      setShowRecent(false);
    }
  };

  const handleRemoveRecent = (e: React.MouseEvent, recentPath: string) => {
    e.stopPropagation();
    onRemoveRecentRepository?.(recentPath);
  };

  if (loading) {
    return (
      <div className="repo-selector loading">
        <div className="spinner" />
        <span>Opening repository...</span>
      </div>
    );
  }

  if (repository && !isEditing) {
    return (
      <div className="repo-selector selected">
        <div className="repo-info">
          <span className="repo-icon">📁</span>
          <div className="repo-details">
            <span className="repo-name">
              {repository.owner}/{repository.name}
            </span>
            <span className="repo-branch">{repository.current_branch}</span>
          </div>
        </div>
        <button
          className="change-repo-btn"
          onClick={() => {
            setIsEditing(true);
            onClearRepository();
          }}
          title="Change repository"
        >
          ✕
        </button>
      </div>
    );
  }

  return (
    <div className="repo-selector empty">
      <form onSubmit={handleSubmit}>
        <div className="input-group">
          <span className="input-icon">📁</span>
          <input
            type="text"
            value={path}
            onChange={handlePathChange}
            onFocus={() => setShowRecent(true)}
            placeholder="Enter repository path..."
            className="repo-path-input"
            autoFocus
          />
          <button type="submit" className="open-btn" disabled={!path.trim()}>
            Open
          </button>
        </div>
        {error && <p className="error-message">{error}</p>}
      </form>

      {showRecent && recentRepositories.length > 0 && !path && (
        <div className="recent-repos">
          <div className="recent-repos-header">
            <span>Recent Repositories</span>
            <button
              className="close-recent-btn"
              onClick={() => setShowRecent(false)}
              title="Close"
            >
              ✕
            </button>
          </div>
          <ul className="recent-repos-list">
            {recentRepositories.map((repo) => (
              <li key={repo.path}>
                <button
                  className="recent-repo-item"
                  onClick={() => handleOpenRecent(repo.path)}
                >
                  <span className="recent-repo-icon">📁</span>
                  <div className="recent-repo-details">
                    <span className="recent-repo-name">
                      {repo.owner}/{repo.name}
                    </span>
                    <span className="recent-repo-path">{repo.path}</span>
                  </div>
                  <button
                    className="remove-recent-btn"
                    onClick={(e) => handleRemoveRecent(e, repo.path)}
                    title="Remove from recent"
                  >
                    ✕
                  </button>
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
