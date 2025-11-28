//! GraphQL query definitions for GitHub API.
//!
//! This module contains the GraphQL queries and response types
//! for interacting with the GitHub GraphQL API.

use serde::{Deserialize, Serialize};

/// Query to list pull requests for a repository.
pub const LIST_PULL_REQUESTS: &str = r#"
query ListPullRequests($owner: String!, $repo: String!, $baseRefName: String, $first: Int!, $after: String) {
  repository(owner: $owner, name: $repo) {
    pullRequests(
      baseRefName: $baseRefName
      states: [OPEN]
      first: $first
      after: $after
      orderBy: { field: UPDATED_AT, direction: DESC }
    ) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        id
        number
        title
        body
        state
        isDraft
        createdAt
        updatedAt
        author {
          login
          avatarUrl
        }
        labels(first: 10) {
          nodes {
            name
            color
          }
        }
        reviewDecision
        headRefName
        baseRefName
        mergeable
        commits {
          totalCount
        }
        additions
        deletions
        changedFiles
      }
    }
  }
}
"#;

/// Query to get details of a specific pull request.
pub const GET_PULL_REQUEST_DETAILS: &str = r#"
query GetPullRequestDetails($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      id
      number
      title
      body
      state
      isDraft
      mergeable
      createdAt
      updatedAt
      author {
        login
        avatarUrl
      }
      headRefName
      baseRefName
      headRefOid
      baseRefOid
      additions
      deletions
      changedFiles
      commits(first: 100) {
        nodes {
          commit {
            oid
            message
            author {
              name
              date
            }
          }
        }
      }
      files(first: 100) {
        nodes {
          path
          additions
          deletions
          changeType
        }
      }
      reviews(first: 50) {
        nodes {
          author {
            login
          }
          state
          submittedAt
        }
      }
      reviewRequests(first: 10) {
        nodes {
          requestedReviewer {
            ... on User {
              login
            }
          }
        }
      }
    }
  }
}
"#;

/// Mutation to create a pull request.
pub const CREATE_PULL_REQUEST: &str = r#"
mutation CreatePullRequest($repositoryId: ID!, $baseRefName: String!, $headRefName: String!, $title: String!, $body: String, $draft: Boolean) {
  createPullRequest(input: {
    repositoryId: $repositoryId
    baseRefName: $baseRefName
    headRefName: $headRefName
    title: $title
    body: $body
    draft: $draft
  }) {
    pullRequest {
      id
      number
      url
    }
  }
}
"#;

/// Mutation to merge a pull request.
pub const MERGE_PULL_REQUEST: &str = r#"
mutation MergePullRequest($pullRequestId: ID!, $mergeMethod: PullRequestMergeMethod) {
  mergePullRequest(input: { 
    pullRequestId: $pullRequestId
    mergeMethod: $mergeMethod
  }) {
    pullRequest {
      number
      merged
    }
  }
}
"#;

/// Mutation to update pull request base branch.
pub const UPDATE_PULL_REQUEST_BRANCH: &str = r#"
mutation UpdatePullRequestBranch($pullRequestId: ID!, $expectedHeadOid: GitObjectID) {
  updatePullRequestBranch(input: { 
    pullRequestId: $pullRequestId
    expectedHeadOid: $expectedHeadOid
  }) {
    pullRequest {
      number
      headRefOid
    }
  }
}
"#;

/// Mutation to close a pull request.
pub const CLOSE_PULL_REQUEST: &str = r#"
mutation ClosePullRequest($pullRequestId: ID!) {
  closePullRequest(input: { pullRequestId: $pullRequestId }) {
    pullRequest {
      number
      state
    }
  }
}
"#;

/// Query to get repository ID.
pub const GET_REPOSITORY_ID: &str = r#"
query GetRepositoryId($owner: String!, $repo: String!) {
  repository(owner: $owner, name: $repo) {
    id
    defaultBranchRef {
      name
    }
  }
}
"#;

/// Variables for listing pull requests.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPullRequestsVariables {
    pub owner: String,
    pub repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_ref_name: Option<String>,
    pub first: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

/// Variables for getting PR details.
#[derive(Debug, Clone, Serialize)]
pub struct GetPullRequestDetailsVariables {
    pub owner: String,
    pub repo: String,
    pub number: i32,
}

/// Variables for creating a pull request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePullRequestVariables {
    pub repository_id: String,
    pub base_ref_name: String,
    pub head_ref_name: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

/// Variables for merging a pull request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePullRequestVariables {
    pub pull_request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_method: Option<String>,
}

/// Variables for closing a pull request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePullRequestVariables {
    pub pull_request_id: String,
}

/// Variables for getting repository ID.
#[derive(Debug, Clone, Serialize)]
pub struct GetRepositoryIdVariables {
    pub owner: String,
    pub repo: String,
}

// Response types

/// Page info for pagination.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

/// Author information from GraphQL.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlAuthor {
    pub login: String,
    pub avatar_url: String,
}

/// Label from GraphQL.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlLabel {
    pub name: String,
    pub color: String,
}

/// Labels wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlLabels {
    pub nodes: Option<Vec<GqlLabel>>,
}

/// Commits wrapper.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlCommits {
    pub total_count: i32,
}

/// Pull request node from list query.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlPullRequestNode {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub is_draft: bool,
    pub created_at: String,
    pub updated_at: String,
    pub author: Option<GqlAuthor>,
    pub labels: GqlLabels,
    pub review_decision: Option<String>,
    pub head_ref_name: String,
    pub base_ref_name: String,
    pub mergeable: String,
    pub commits: GqlCommits,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
}

/// Pull requests connection.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlPullRequestsConnection {
    pub page_info: PageInfo,
    pub nodes: Option<Vec<GqlPullRequestNode>>,
}

/// Repository from list PRs query.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlRepositoryPullRequests {
    pub pull_requests: GqlPullRequestsConnection,
}

/// Response for list pull requests query.
#[derive(Debug, Clone, Deserialize)]
pub struct ListPullRequestsResponse {
    pub repository: Option<GqlRepositoryPullRequests>,
}

/// Commit author for details query.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlCommitAuthor {
    pub name: Option<String>,
    pub date: Option<String>,
}

/// Commit details.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlCommitDetails {
    pub oid: String,
    pub message: String,
    pub author: Option<GqlCommitAuthor>,
}

/// Commit node wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlCommitNode {
    pub commit: GqlCommitDetails,
}

/// Commits connection.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlCommitsConnection {
    pub nodes: Option<Vec<GqlCommitNode>>,
}

/// File change details.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlFileChange {
    pub path: String,
    pub additions: i32,
    pub deletions: i32,
    pub change_type: String,
}

/// Files connection.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlFilesConnection {
    pub nodes: Option<Vec<GqlFileChange>>,
}

/// Review node.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlReviewNode {
    pub author: Option<GqlAuthor>,
    pub state: String,
    pub submitted_at: Option<String>,
}

/// Reviews connection.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlReviewsConnection {
    pub nodes: Option<Vec<GqlReviewNode>>,
}

/// Review requester.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlRequestedReviewer {
    pub login: Option<String>,
}

/// Review request node.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlReviewRequestNode {
    pub requested_reviewer: Option<GqlRequestedReviewer>,
}

/// Review requests connection.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlReviewRequestsConnection {
    pub nodes: Option<Vec<GqlReviewRequestNode>>,
}

/// Detailed pull request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlPullRequestDetails {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub is_draft: bool,
    pub mergeable: String,
    pub created_at: String,
    pub updated_at: String,
    pub author: Option<GqlAuthor>,
    pub head_ref_name: String,
    pub base_ref_name: String,
    pub head_ref_oid: String,
    pub base_ref_oid: String,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub commits: GqlCommitsConnection,
    pub files: GqlFilesConnection,
    pub reviews: GqlReviewsConnection,
    pub review_requests: GqlReviewRequestsConnection,
}

/// Repository from get PR details query.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlRepositoryPullRequest {
    pub pull_request: Option<GqlPullRequestDetails>,
}

/// Response for get PR details query.
#[derive(Debug, Clone, Deserialize)]
pub struct GetPullRequestDetailsResponse {
    pub repository: Option<GqlRepositoryPullRequest>,
}

/// Created pull request result.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlCreatedPullRequest {
    pub id: String,
    pub number: i64,
    pub url: String,
}

/// Create PR mutation result.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlCreatePullRequestResult {
    pub pull_request: GqlCreatedPullRequest,
}

/// Response for create PR mutation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePullRequestResponse {
    pub create_pull_request: GqlCreatePullRequestResult,
}

/// Merged pull request result.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlMergedPullRequest {
    pub number: i64,
    pub merged: bool,
}

/// Merge PR mutation result.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlMergePullRequestResult {
    pub pull_request: GqlMergedPullRequest,
}

/// Response for merge PR mutation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePullRequestResponse {
    pub merge_pull_request: GqlMergePullRequestResult,
}

/// Default branch ref.
#[derive(Debug, Clone, Deserialize)]
pub struct GqlDefaultBranchRef {
    pub name: String,
}

/// Repository ID result.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GqlRepositoryId {
    pub id: String,
    pub default_branch_ref: Option<GqlDefaultBranchRef>,
}

/// Response for get repository ID query.
#[derive(Debug, Clone, Deserialize)]
pub struct GetRepositoryIdResponse {
    pub repository: Option<GqlRepositoryId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_pr_variables_serialization() {
        let vars = ListPullRequestsVariables {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            base_ref_name: Some("main".to_string()),
            first: 20,
            after: None,
        };
        let json = serde_json::to_string(&vars).unwrap();
        assert!(json.contains("\"owner\":\"owner\""));
        assert!(json.contains("\"baseRefName\":\"main\""));
    }

    #[test]
    fn test_list_pr_variables_without_base() {
        let vars = ListPullRequestsVariables {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            base_ref_name: None,
            first: 20,
            after: None,
        };
        let json = serde_json::to_string(&vars).unwrap();
        assert!(!json.contains("baseRefName"));
    }
}
