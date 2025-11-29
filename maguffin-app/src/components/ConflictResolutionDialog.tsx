import React from "react";
import type { RestackResult, RestackConflict } from "../types";

interface ConflictResolutionDialogProps {
  result: RestackResult;
  onClose: () => void;
  onRetry?: () => void;
}

export function ConflictResolutionDialog({
  result,
  onClose,
  onRetry,
}: ConflictResolutionDialogProps) {
  const hasConflicts = result.status === "conflicts" && result.conflicts.length > 0;
  const hasFailed = result.status === "failed";
  const isSuccess = result.status === "success";

  const getStatusIcon = () => {
    if (isSuccess) return "✓";
    if (hasConflicts) return "⚠";
    if (hasFailed) return "✗";
    return "?";
  };

  const getStatusColor = () => {
    if (isSuccess) return "green";
    if (hasConflicts) return "yellow";
    if (hasFailed) return "red";
    return "gray";
  };

  const getTitle = () => {
    if (isSuccess) return "Restack Complete";
    if (hasConflicts) return "Conflicts Detected";
    if (hasFailed) return "Restack Failed";
    return "Restack Result";
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog conflict-resolution-dialog" onClick={(e) => e.stopPropagation()}>
        <header className="dialog-header">
          <span className={`status-icon status-${getStatusColor()}`}>{getStatusIcon()}</span>
          <h2>{getTitle()}</h2>
          <button className="close-btn" onClick={onClose} aria-label="Close">×</button>
        </header>

        <div className="dialog-content">
          {/* Success message */}
          {isSuccess && (
            <div className="success-message">
              <p>All branches have been successfully restacked.</p>
              {result.restacked.length > 0 && (
                <div className="restacked-branches">
                  <h4>Restacked branches:</h4>
                  <ul>
                    {result.restacked.map((branch) => (
                      <li key={branch} className="branch-item">
                        <span className="branch-icon">✓</span>
                        <code>{branch}</code>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {/* Conflict details */}
          {hasConflicts && (
            <div className="conflict-details">
              <p className="conflict-intro">
                The restack operation was paused due to merge conflicts. 
                Please resolve the conflicts manually and try again.
              </p>

              {result.restacked.length > 0 && (
                <div className="restacked-branches">
                  <h4>Successfully restacked:</h4>
                  <ul>
                    {result.restacked.map((branch) => (
                      <li key={branch} className="branch-item">
                        <span className="branch-icon success">✓</span>
                        <code>{branch}</code>
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="conflict-branches">
                <h4>Branches with conflicts:</h4>
                {result.conflicts.map((conflict) => (
                  <ConflictBranchItem key={conflict.branch} conflict={conflict} />
                ))}
              </div>

              <div className="resolution-instructions">
                <h4>How to resolve:</h4>
                <ol>
                  <li>Open your terminal in the repository directory</li>
                  <li>Run: <code>git checkout {result.conflicts[0]?.branch}</code></li>
                  <li>Resolve the conflicts manually using your preferred merge tool</li>
                  <li>Stage the resolved files: <code>git add .</code></li>
                  <li>Complete any pending rebase with: <code>git rebase --continue</code></li>
                  <li>Once resolved, click "Retry Restack" below</li>
                </ol>
              </div>
            </div>
          )}

          {/* Failed message */}
          {hasFailed && (
            <div className="error-message">
              <p>The restack operation failed with an error:</p>
              <pre className="error-details">{result.error}</pre>

              {result.restacked.length > 0 && (
                <div className="restacked-branches">
                  <h4>Branches restacked before failure:</h4>
                  <ul>
                    {result.restacked.map((branch) => (
                      <li key={branch} className="branch-item">
                        <span className="branch-icon success">✓</span>
                        <code>{branch}</code>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>

        <footer className="dialog-footer">
          {(hasConflicts || hasFailed) && onRetry && (
            <button className="btn btn-primary" onClick={onRetry}>
              Retry Restack
            </button>
          )}
          <button className="btn btn-secondary" onClick={onClose}>
            {isSuccess ? "Done" : "Close"}
          </button>
        </footer>
      </div>
    </div>
  );
}

interface ConflictBranchItemProps {
  conflict: RestackConflict;
}

function ConflictBranchItem({ conflict }: ConflictBranchItemProps) {
  const [expanded, setExpanded] = React.useState(true);

  return (
    <div className="conflict-branch">
      <div 
        className="conflict-branch-header" 
        onClick={() => setExpanded(!expanded)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === "Enter" && setExpanded(!expanded)}
      >
        <span className="expand-icon">{expanded ? "▼" : "▶"}</span>
        <span className="branch-icon conflict">⚠</span>
        <code className="branch-name">{conflict.branch}</code>
        {conflict.files.length > 0 && (
          <span className="file-count">({conflict.files.length} file{conflict.files.length !== 1 ? "s" : ""})</span>
        )}
      </div>
      
      {expanded && conflict.files.length > 0 && (
        <ul className="conflict-files">
          {conflict.files.map((file) => (
            <li key={file} className="conflict-file">
              <span className="file-icon">📄</span>
              <code>{file}</code>
            </li>
          ))}
        </ul>
      )}

      {expanded && conflict.files.length === 0 && (
        <p className="no-files-note">Run <code>git status</code> to see conflicting files</p>
      )}
    </div>
  );
}
