import { useMemo } from "react";
import type { CheckStatus as CheckStatusType, CheckRun, CheckState } from "../types";

interface CheckStatusProps {
  checkStatus: CheckStatusType | null;
}

function getStateIcon(state: CheckState): { icon: string; className: string } {
  switch (state) {
    case "SUCCESS":
      return { icon: "✓", className: "check-success" };
    case "PENDING":
      return { icon: "○", className: "check-pending" };
    case "FAILURE":
      return { icon: "✗", className: "check-failure" };
    default:
      return { icon: "?", className: "check-unknown" };
  }
}

function getCheckIcon(check: CheckRun): { icon: string; className: string } {
  if (check.status === "IN_PROGRESS" || check.status === "QUEUED") {
    return { icon: "○", className: "check-pending" };
  }
  
  if (check.conclusion) {
    switch (check.conclusion) {
      case "SUCCESS":
        return { icon: "✓", className: "check-success" };
      case "FAILURE":
        return { icon: "✗", className: "check-failure" };
      case "NEUTRAL":
      case "SKIPPED":
        return { icon: "−", className: "check-neutral" };
      case "CANCELLED":
      case "TIMED_OUT":
        return { icon: "⚠", className: "check-warning" };
      case "ACTION_REQUIRED":
        return { icon: "!", className: "check-warning" };
      default:
        return { icon: "?", className: "check-unknown" };
    }
  }
  
  return { icon: "?", className: "check-unknown" };
}

function getStatusLabel(state: CheckState): string {
  switch (state) {
    case "SUCCESS":
      return "All checks passed";
    case "PENDING":
      return "Checks in progress";
    case "FAILURE":
      return "Some checks failed";
    default:
      return "Unknown status";
  }
}

export function CheckStatusDisplay({ checkStatus }: CheckStatusProps) {
  // Memoize the passed count calculation
  const passedCount = useMemo(() => {
    if (!checkStatus) return 0;
    return checkStatus.checks.filter((c) => c.conclusion === "SUCCESS").length;
  }, [checkStatus]);

  if (!checkStatus) {
    return (
      <div className="check-status check-status-none">
        <span className="check-icon check-unknown">−</span>
        <span className="check-label">No checks</span>
      </div>
    );
  }

  const { icon, className } = getStateIcon(checkStatus.state);
  const label = getStatusLabel(checkStatus.state);
  const totalCount = checkStatus.checks.length;

  return (
    <div className="check-status-container">
      <div className={`check-status-summary ${className}`}>
        <span className={`check-icon ${className}`}>{icon}</span>
        <span className="check-label">{label}</span>
        <span className="check-count">
          ({passedCount}/{totalCount})
        </span>
      </div>
      
      {checkStatus.checks.length > 0 && (
        <div className="check-list">
          {checkStatus.checks.map((check, index) => {
            const checkIcon = getCheckIcon(check);
            return (
              <div key={`${check.name}-${index}`} className="check-item">
                <span className={`check-icon ${checkIcon.className}`}>
                  {checkIcon.icon}
                </span>
                <span className="check-name">{check.name}</span>
                {check.details_url && (
                  <a
                    href={check.details_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="check-details-link"
                  >
                    Details
                  </a>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
