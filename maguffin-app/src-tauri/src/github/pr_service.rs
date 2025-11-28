//! Pull Request Service.
//!
//! This module provides the service layer for pull request operations,
//! bridging the domain types with the GitHub GraphQL API.

use crate::domain::pr::{
    Author, ChangeType, ChangedFile, Commit, Label, MergeMethod, Mergeable, PrState, PullRequest,
    PullRequestDetails, Review, ReviewDecision, ReviewState,
};
use crate::error::{GitHubError, Result};
use crate::github::queries::{
    ClosePullRequestVariables, CreatePullRequestVariables, GetPullRequestDetailsResponse,
    GetPullRequestDetailsVariables, GetRepositoryIdResponse, GetRepositoryIdVariables,
    GqlPullRequestDetails, GqlPullRequestNode, ListPullRequestsResponse,
    ListPullRequestsVariables, MergePullRequestVariables, CLOSE_PULL_REQUEST, CREATE_PULL_REQUEST,
    GET_PULL_REQUEST_DETAILS, GET_REPOSITORY_ID, LIST_PULL_REQUESTS, MERGE_PULL_REQUEST,
};
use crate::github::GitHubClient;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Service for pull request operations.
pub struct PrService {
    client: Arc<GitHubClient>,
    owner: String,
    repo: String,
}

impl PrService {
    /// Create a new PR service.
    pub fn new(client: Arc<GitHubClient>, owner: String, repo: String) -> Self {
        Self {
            client,
            owner,
            repo,
        }
    }

    /// List open pull requests.
    pub async fn list_prs(&self, base_branch: Option<String>) -> Result<Vec<PullRequest>> {
        let mut all_prs = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = ListPullRequestsVariables {
                owner: self.owner.clone(),
                repo: self.repo.clone(),
                base_ref_name: base_branch.clone(),
                first: 50,
                after: cursor.clone(),
            };

            let response: ListPullRequestsResponse = self
                .client
                .query(LIST_PULL_REQUESTS, serde_json::to_value(variables)?)
                .await?;

            let repository = response
                .repository
                .ok_or_else(|| GitHubError::NotFound("Repository not found".to_string()))?;

            let connection = repository.pull_requests;

            if let Some(nodes) = connection.nodes {
                for node in nodes {
                    all_prs.push(Self::convert_pr_node(node));
                }
            }

            if connection.page_info.has_next_page {
                cursor = connection.page_info.end_cursor;
            } else {
                break;
            }
        }

        Ok(all_prs)
    }

    /// Get details for a specific pull request.
    pub async fn get_pr_details(&self, number: i64) -> Result<PullRequestDetails> {
        let variables = GetPullRequestDetailsVariables {
            owner: self.owner.clone(),
            repo: self.repo.clone(),
            number: number as i32,
        };

        let response: GetPullRequestDetailsResponse = self
            .client
            .query(GET_PULL_REQUEST_DETAILS, serde_json::to_value(variables)?)
            .await?;

        let repository = response
            .repository
            .ok_or_else(|| GitHubError::NotFound("Repository not found".to_string()))?;

        let pr = repository
            .pull_request
            .ok_or_else(|| GitHubError::NotFound(format!("PR #{} not found", number)))?;

        Ok(Self::convert_pr_details(pr))
    }

    /// Create a new pull request.
    pub async fn create_pr(
        &self,
        title: String,
        body: Option<String>,
        head: String,
        base: String,
        draft: bool,
    ) -> Result<i64> {
        // First, get the repository ID
        let repo_id = self.get_repository_id().await?;

        let variables = CreatePullRequestVariables {
            repository_id: repo_id,
            base_ref_name: base,
            head_ref_name: head,
            title,
            body,
            draft: Some(draft),
        };

        let response: serde_json::Value = self
            .client
            .query(CREATE_PULL_REQUEST, serde_json::to_value(variables)?)
            .await?;

        let number = response["createPullRequest"]["pullRequest"]["number"]
            .as_i64()
            .ok_or_else(|| GitHubError::GraphQL("Failed to get PR number".to_string()))?;

        Ok(number)
    }

    /// Merge a pull request.
    pub async fn merge_pr(&self, pr_id: String, method: MergeMethod) -> Result<bool> {
        let merge_method = match method {
            MergeMethod::Merge => "MERGE",
            MergeMethod::Squash => "SQUASH",
            MergeMethod::Rebase => "REBASE",
        };

        let variables = MergePullRequestVariables {
            pull_request_id: pr_id,
            merge_method: Some(merge_method.to_string()),
        };

        let response: serde_json::Value = self
            .client
            .query(MERGE_PULL_REQUEST, serde_json::to_value(variables)?)
            .await?;

        let merged = response["mergePullRequest"]["pullRequest"]["merged"]
            .as_bool()
            .unwrap_or(false);

        Ok(merged)
    }

    /// Close a pull request without merging.
    pub async fn close_pr(&self, pr_id: String) -> Result<bool> {
        let variables = ClosePullRequestVariables {
            pull_request_id: pr_id,
        };

        let response: serde_json::Value = self
            .client
            .query(CLOSE_PULL_REQUEST, serde_json::to_value(variables)?)
            .await?;

        let state = response["closePullRequest"]["pullRequest"]["state"]
            .as_str()
            .unwrap_or("");

        Ok(state == "CLOSED")
    }

    /// Get the repository ID.
    async fn get_repository_id(&self) -> Result<String> {
        let variables = GetRepositoryIdVariables {
            owner: self.owner.clone(),
            repo: self.repo.clone(),
        };

        let response: GetRepositoryIdResponse = self
            .client
            .query(GET_REPOSITORY_ID, serde_json::to_value(variables)?)
            .await?;

        let repository = response
            .repository
            .ok_or_else(|| GitHubError::NotFound("Repository not found".to_string()))?;

        Ok(repository.id)
    }

    /// Convert a GraphQL PR node to domain type.
    fn convert_pr_node(node: GqlPullRequestNode) -> PullRequest {
        let author = node
            .author
            .map(|a| Author {
                login: a.login,
                avatar_url: a.avatar_url,
            })
            .unwrap_or(Author {
                login: "ghost".to_string(),
                avatar_url: String::new(),
            });

        let labels: Vec<Label> = node
            .labels
            .nodes
            .unwrap_or_default()
            .into_iter()
            .map(|l| Label {
                name: l.name,
                color: l.color,
            })
            .collect();

        let state = match node.state.as_str() {
            "OPEN" => PrState::Open,
            "CLOSED" => PrState::Closed,
            "MERGED" => PrState::Merged,
            _ => PrState::Open,
        };

        let review_decision = node.review_decision.as_deref().map(|d| match d {
            "APPROVED" => ReviewDecision::Approved,
            "CHANGES_REQUESTED" => ReviewDecision::ChangesRequested,
            "REVIEW_REQUIRED" => ReviewDecision::ReviewRequired,
            _ => ReviewDecision::ReviewRequired,
        });

        let mergeable = match node.mergeable.as_str() {
            "MERGEABLE" => Mergeable::Mergeable,
            "CONFLICTING" => Mergeable::Conflicting,
            _ => Mergeable::Unknown,
        };

        let created_at = DateTime::parse_from_rfc3339(&node.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&node.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        PullRequest {
            id: node.id,
            number: node.number,
            title: node.title,
            body: node.body,
            state,
            is_draft: node.is_draft,
            author,
            head_ref: node.head_ref_name,
            base_ref: node.base_ref_name,
            labels,
            review_decision,
            mergeable,
            created_at,
            updated_at,
            commit_count: node.commits.total_count,
            additions: node.additions,
            deletions: node.deletions,
            changed_files: node.changed_files,
        }
    }

    /// Convert detailed PR from GraphQL to domain type.
    fn convert_pr_details(pr: GqlPullRequestDetails) -> PullRequestDetails {
        let author = pr
            .author
            .map(|a| Author {
                login: a.login,
                avatar_url: a.avatar_url,
            })
            .unwrap_or(Author {
                login: "ghost".to_string(),
                avatar_url: String::new(),
            });

        let state = match pr.state.as_str() {
            "OPEN" => PrState::Open,
            "CLOSED" => PrState::Closed,
            "MERGED" => PrState::Merged,
            _ => PrState::Open,
        };

        let mergeable = match pr.mergeable.as_str() {
            "MERGEABLE" => Mergeable::Mergeable,
            "CONFLICTING" => Mergeable::Conflicting,
            _ => Mergeable::Unknown,
        };

        let created_at = DateTime::parse_from_rfc3339(&pr.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let updated_at = DateTime::parse_from_rfc3339(&pr.updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let commits: Vec<Commit> = pr
            .commits
            .nodes
            .unwrap_or_default()
            .into_iter()
            .map(|c| {
                let authored_date = c
                    .commit
                    .author
                    .as_ref()
                    .and_then(|a| a.date.as_ref())
                    .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                Commit {
                    oid: c.commit.oid,
                    message: c.commit.message,
                    author_name: c
                        .commit
                        .author
                        .and_then(|a| a.name)
                        .unwrap_or_else(|| "Unknown".to_string()),
                    authored_date,
                }
            })
            .collect();

        let files: Vec<ChangedFile> = pr
            .files
            .nodes
            .unwrap_or_default()
            .into_iter()
            .map(|f| {
                let change_type = match f.change_type.as_str() {
                    "ADDED" => ChangeType::Added,
                    "DELETED" => ChangeType::Deleted,
                    "MODIFIED" => ChangeType::Modified,
                    "RENAMED" => ChangeType::Renamed,
                    "COPIED" => ChangeType::Copied,
                    _ => ChangeType::Changed,
                };

                ChangedFile {
                    path: f.path,
                    additions: f.additions,
                    deletions: f.deletions,
                    change_type,
                }
            })
            .collect();

        let reviews: Vec<Review> = pr
            .reviews
            .nodes
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| {
                let author = r.author?.login;
                let state = match r.state.as_str() {
                    "PENDING" => ReviewState::Pending,
                    "COMMENTED" => ReviewState::Commented,
                    "APPROVED" => ReviewState::Approved,
                    "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                    "DISMISSED" => ReviewState::Dismissed,
                    _ => ReviewState::Pending,
                };
                let submitted_at = r
                    .submitted_at
                    .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                Some(Review {
                    author,
                    state,
                    submitted_at,
                })
            })
            .collect();

        let review_requests: Vec<String> = pr
            .review_requests
            .nodes
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| r.requested_reviewer?.login)
            .collect();

        let commit_count = commits.len() as i32;

        let base_pr = PullRequest {
            id: pr.id,
            number: pr.number,
            title: pr.title,
            body: pr.body,
            state,
            is_draft: pr.is_draft,
            author,
            head_ref: pr.head_ref_name,
            base_ref: pr.base_ref_name,
            labels: Vec::new(),    // Not included in details query
            review_decision: None, // Could add to query if needed
            mergeable,
            created_at,
            updated_at,
            commit_count,
            additions: pr.additions,
            deletions: pr.deletions,
            changed_files: pr.changed_files,
        };

        PullRequestDetails {
            pr: base_pr,
            commits,
            files,
            reviews,
            review_requests,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_method_conversion() {
        assert_eq!(
            match MergeMethod::Squash {
                MergeMethod::Merge => "MERGE",
                MergeMethod::Squash => "SQUASH",
                MergeMethod::Rebase => "REBASE",
            },
            "SQUASH"
        );
    }
}
