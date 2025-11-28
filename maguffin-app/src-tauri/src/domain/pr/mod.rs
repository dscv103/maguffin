//! Pull Request domain types and service.
//!
//! This module contains the core domain types for representing GitHub pull requests
//! and the service layer for PR operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a GitHub pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR number
    pub number: i64,

    /// PR title
    pub title: String,

    /// PR body/description (Markdown)
    pub body: Option<String>,

    /// Current state
    pub state: PrState,

    /// Whether this is a draft PR
    pub is_draft: bool,

    /// Author information
    pub author: Author,

    /// Head branch name (source)
    pub head_ref: String,

    /// Base branch name (target)
    pub base_ref: String,

    /// Labels attached to the PR
    pub labels: Vec<Label>,

    /// Review decision status
    pub review_decision: Option<ReviewDecision>,

    /// Whether the PR can be merged
    pub mergeable: Mergeable,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Number of commits in the PR
    pub commit_count: i32,

    /// Number of additions
    pub additions: i32,

    /// Number of deletions
    pub deletions: i32,

    /// Number of changed files
    pub changed_files: i32,
}

/// PR author information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// GitHub username
    pub login: String,

    /// Avatar URL
    pub avatar_url: String,
}

/// PR state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

/// PR label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Label name
    pub name: String,

    /// Label color (hex without #)
    pub color: String,
}

/// Review decision for a PR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

/// Whether a PR is mergeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Mergeable {
    Mergeable,
    Conflicting,
    Unknown,
}

/// Detailed PR information including commits and files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestDetails {
    /// Basic PR information
    #[serde(flatten)]
    pub pr: PullRequest,

    /// Commits in this PR
    pub commits: Vec<Commit>,

    /// Files changed in this PR
    pub files: Vec<ChangedFile>,

    /// Reviews on this PR
    pub reviews: Vec<Review>,

    /// Review requests (pending reviewers)
    pub review_requests: Vec<String>,
}

/// A commit in a PR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Commit SHA (object ID)
    pub oid: String,

    /// Commit message
    pub message: String,

    /// Commit author name
    pub author_name: String,

    /// Commit date
    pub authored_date: DateTime<Utc>,
}

/// A file changed in a PR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    /// File path
    pub path: String,

    /// Number of additions
    pub additions: i32,

    /// Number of deletions
    pub deletions: i32,

    /// Type of change
    pub change_type: ChangeType,
}

/// Type of file change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeType {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Changed,
}

/// A review on a PR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Reviewer username
    pub author: String,

    /// Review state
    pub state: ReviewState,

    /// When the review was submitted
    pub submitted_at: DateTime<Utc>,
}

/// State of a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewState {
    Pending,
    Commented,
    Approved,
    ChangesRequested,
    Dismissed,
}

/// Options for creating a new PR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrOptions {
    /// PR title
    pub title: String,

    /// PR body/description
    pub body: Option<String>,

    /// Head branch (source)
    pub head: String,

    /// Base branch (target)
    pub base: String,

    /// Whether to create as draft
    pub draft: bool,
}

/// Options for merging a PR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_state_serialization() {
        let state = PrState::Open;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"OPEN\"");
    }

    #[test]
    fn test_review_decision_serialization() {
        let decision = ReviewDecision::Approved;
        let json = serde_json::to_string(&decision).unwrap();
        assert_eq!(json, "\"APPROVED\"");
    }

    #[test]
    fn test_merge_method_serialization() {
        let method = MergeMethod::Squash;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"SQUASH\"");
    }
}
