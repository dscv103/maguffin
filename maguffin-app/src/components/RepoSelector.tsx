import React, { useState } from "react";
import type { Repository } from "../types";

interface RepoSelectorProps {
  repository: Repository | null;
  loading: boolean;
  error: string | null;
  onOpenRepository: (path: string) => Promise<Repository | null>;
  onClearRepository: () => void;
  onClearError?: () => void;
}

export function RepoSelector({
  repository,
  loading,
  error,
  onOpenRepository,
  onClearRepository,
  onClearError,
}: RepoSelectorProps) {
  const [path, setPath] = useState("");
  const [isEditing, setIsEditing] = useState(false);

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
      }
    }
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
    </div>
  );
}
