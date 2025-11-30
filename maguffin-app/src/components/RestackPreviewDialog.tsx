import type { RestackPreview, RestackBranchPreview } from "../types";

interface RestackPreviewDialogProps {
  preview: RestackPreview;
  onConfirm: () => void;
  onCancel: () => void;
  isLoading?: boolean;
}

export function RestackPreviewDialog({
  preview,
  onConfirm,
  onCancel,
  isLoading = false,
}: RestackPreviewDialogProps) {
  const hasWork = preview.will_rebase.length > 0;
  const allUpToDate = !hasWork && preview.up_to_date.length > 0;

  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div className="dialog restack-preview-dialog" onClick={(e) => e.stopPropagation()}>
        <header className="dialog-header">
          <span className="status-icon status-blue">👁</span>
          <h2>Restack Preview</h2>
          <button className="close-btn" onClick={onCancel} aria-label="Close">×</button>
        </header>

        <div className="dialog-content">
          {allUpToDate ? (
            <div className="success-message">
              <p>✓ All branches are already up to date. No restack needed.</p>
              <div className="up-to-date-branches">
                <h4>Up-to-date branches:</h4>
                <ul>
                  {preview.up_to_date.map((branch) => (
                    <li key={branch} className="branch-item">
                      <span className="branch-icon success">✓</span>
                      <code>{branch}</code>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          ) : hasWork ? (
            <div className="restack-preview">
              <p className="preview-intro">
                The following branches will be rebased. This will replay 
                <strong> {preview.total_commits} commit{preview.total_commits !== 1 ? "s" : ""}</strong> 
                {" "}across <strong>{preview.will_rebase.length} branch{preview.will_rebase.length !== 1 ? "es" : ""}</strong>.
              </p>

              <div className="will-rebase-branches">
                <h4>Branches to rebase:</h4>
                {preview.will_rebase.map((branch) => (
                  <BranchPreviewItem key={branch.branch} branch={branch} />
                ))}
              </div>

              {preview.up_to_date.length > 0 && (
                <div className="up-to-date-branches">
                  <h4>Already up to date:</h4>
                  <ul>
                    {preview.up_to_date.map((branch) => (
                      <li key={branch} className="branch-item">
                        <span className="branch-icon success">✓</span>
                        <code>{branch}</code>
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="preview-warning">
                <span className="warning-icon">⚠</span>
                <p>
                  This operation will force-push the rebased branches. Make sure no one else 
                  is working on these branches before proceeding.
                </p>
              </div>
            </div>
          ) : (
            <div className="empty-message">
              <p>No branches found in this stack.</p>
            </div>
          )}
        </div>

        <footer className="dialog-footer">
          {hasWork && (
            <button 
              className="btn btn-primary" 
              onClick={onConfirm}
              disabled={isLoading}
            >
              {isLoading ? "Restacking..." : "Confirm Restack"}
            </button>
          )}
          <button className="btn btn-secondary" onClick={onCancel} disabled={isLoading}>
            {hasWork ? "Cancel" : "Close"}
          </button>
        </footer>
      </div>
    </div>
  );
}

interface BranchPreviewItemProps {
  branch: RestackBranchPreview;
}

function BranchPreviewItem({ branch }: BranchPreviewItemProps) {
  return (
    <div className="branch-preview-item">
      <div className="branch-preview-header">
        <span className="branch-icon rebase">↻</span>
        <code className="branch-name">{branch.branch}</code>
        <span className="arrow">→</span>
        <code className="onto-name">{branch.onto}</code>
        {branch.has_pr && <span className="pr-badge" title="Has associated PR">PR</span>}
      </div>
      <div className="branch-preview-details">
        <span className="commits-count">
          {branch.commits_to_replay} commit{branch.commits_to_replay !== 1 ? "s" : ""} to replay
        </span>
      </div>
    </div>
  );
}
