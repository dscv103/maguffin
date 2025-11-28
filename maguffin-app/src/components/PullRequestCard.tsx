import type { PullRequest } from "../types";

interface PullRequestCardProps {
  pr: PullRequest;
  onClick?: (pr: PullRequest) => void;
}

export function PullRequestCard({ pr, onClick }: PullRequestCardProps) {
  const getStatusColor = (pr: PullRequest): string => {
    if (pr.is_draft) return "gray";
    if (pr.review_decision === "APPROVED") return "green";
    if (pr.review_decision === "CHANGES_REQUESTED") return "red";
    if (pr.mergeable === "CONFLICTING") return "orange";
    return "blue";
  };

  const getStatusLabel = (pr: PullRequest): string => {
    if (pr.is_draft) return "Draft";
    if (pr.review_decision === "APPROVED") return "Approved";
    if (pr.review_decision === "CHANGES_REQUESTED") return "Changes requested";
    if (pr.mergeable === "CONFLICTING") return "Conflicts";
    return "Open";
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "today";
    if (diffDays === 1) return "yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    return date.toLocaleDateString();
  };

  return (
    <div
      className="pr-card"
      onClick={() => onClick?.(pr)}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === "Enter" && onClick?.(pr)}
    >
      <div className="pr-header">
        <span className="pr-number">#{pr.number}</span>
        <span className={`pr-status pr-status-${getStatusColor(pr)}`}>
          {getStatusLabel(pr)}
        </span>
      </div>

      <h3 className="pr-title">{pr.title}</h3>

      <div className="pr-meta">
        <img
          src={pr.author.avatar_url}
          alt={pr.author.login}
          className="pr-avatar"
        />
        <span className="pr-author">{pr.author.login}</span>
        <span className="pr-branch">
          {pr.head_ref} → {pr.base_ref}
        </span>
      </div>

      <div className="pr-stats">
        <span className="pr-additions">+{pr.additions}</span>
        <span className="pr-deletions">-{pr.deletions}</span>
        <span className="pr-files">{pr.changed_files} files</span>
        <span className="pr-commits">{pr.commit_count} commits</span>
      </div>

      {pr.labels.length > 0 && (
        <div className="pr-labels">
          {pr.labels.map((label) => (
            <span
              key={label.name}
              className="pr-label"
              style={{ backgroundColor: `#${label.color}` }}
            >
              {label.name}
            </span>
          ))}
        </div>
      )}

      <div className="pr-updated">Updated {formatDate(pr.updated_at)}</div>
    </div>
  );
}
