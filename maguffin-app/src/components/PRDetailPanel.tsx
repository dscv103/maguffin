import { useState } from "react";
import type { PullRequest, MergeMethod } from "../types";
import { usePullRequestActions } from "../hooks";

interface PRDetailPanelProps {
  pr: PullRequest;
  prId: string; // The GraphQL node ID for the PR
  onClose: () => void;
  onActionComplete?: () => void;
}

export function PRDetailPanel({ pr, prId, onClose, onActionComplete }: PRDetailPanelProps) {
  const { loading, error, mergePR, closePR, checkoutPR, clearError } = usePullRequestActions();
  const [showMergeOptions, setShowMergeOptions] = useState(false);

  const handleCheckout = async () => {
    const success = await checkoutPR(pr.number);
    if (success) {
      onActionComplete?.();
    }
  };

  const handleMerge = async (method: MergeMethod) => {
    const success = await mergePR(prId, method);
    if (success) {
      setShowMergeOptions(false);
      onActionComplete?.();
      onClose();
    }
  };

  const handleClose = async () => {
    const success = await closePR(prId);
    if (success) {
      onActionComplete?.();
      onClose();
    }
  };

  const openInBrowser = () => {
    // Open the PR in the browser
    const url = `https://github.com/${encodeURIComponent(pr.base_ref)}/../pull/${pr.number}`;
    window.open(url, "_blank");
  };

  const canMerge = pr.mergeable === "MERGEABLE" && !pr.is_draft;

  return (
    <aside className="pr-detail-panel">
      <header>
        <h2>
          #{pr.number} {pr.title}
        </h2>
        <button onClick={onClose} aria-label="Close">
          ×
        </button>
      </header>

      {error && (
        <div className="pr-detail-error">
          <p className="error-message">{error}</p>
          <button onClick={clearError} className="dismiss-btn">
            Dismiss
          </button>
        </div>
      )}

      <div className="pr-detail-content">
        <p className="pr-body">{pr.body || "No description provided."}</p>

        <div className="pr-detail-meta">
          <p>
            <strong>Author:</strong> {pr.author.login}
          </p>
          <p>
            <strong>Branch:</strong> {pr.head_ref} → {pr.base_ref}
          </p>
          <p>
            <strong>Changes:</strong> +{pr.additions} -{pr.deletions} ({pr.changed_files} files)
          </p>
          <p>
            <strong>Status:</strong>{" "}
            {pr.is_draft
              ? "Draft"
              : pr.review_decision === "APPROVED"
              ? "Approved"
              : pr.review_decision === "CHANGES_REQUESTED"
              ? "Changes Requested"
              : "Pending Review"}
          </p>
          <p>
            <strong>Mergeable:</strong>{" "}
            {pr.mergeable === "MERGEABLE"
              ? "Yes"
              : pr.mergeable === "CONFLICTING"
              ? "No (has conflicts)"
              : "Unknown"}
          </p>
        </div>
      </div>

      <div className="pr-detail-actions">
        <button
          className="action-btn checkout-btn"
          onClick={handleCheckout}
          disabled={loading}
        >
          {loading ? "..." : "Checkout Branch"}
        </button>

        <button
          className="action-btn browser-btn"
          onClick={openInBrowser}
          disabled={loading}
        >
          Open in Browser
        </button>

        {showMergeOptions ? (
          <div className="merge-options">
            <button
              className="action-btn merge-option"
              onClick={() => handleMerge("MERGE")}
              disabled={loading || !canMerge}
            >
              Merge
            </button>
            <button
              className="action-btn merge-option"
              onClick={() => handleMerge("SQUASH")}
              disabled={loading || !canMerge}
            >
              Squash & Merge
            </button>
            <button
              className="action-btn merge-option"
              onClick={() => handleMerge("REBASE")}
              disabled={loading || !canMerge}
            >
              Rebase & Merge
            </button>
            <button
              className="action-btn cancel-btn"
              onClick={() => setShowMergeOptions(false)}
            >
              Cancel
            </button>
          </div>
        ) : (
          <button
            className="action-btn merge-btn"
            onClick={() => setShowMergeOptions(true)}
            disabled={loading || !canMerge}
            title={!canMerge ? "PR cannot be merged (draft or has conflicts)" : ""}
          >
            Merge
          </button>
        )}

        <button
          className="action-btn close-btn"
          onClick={handleClose}
          disabled={loading}
        >
          Close PR
        </button>
      </div>
    </aside>
  );
}
