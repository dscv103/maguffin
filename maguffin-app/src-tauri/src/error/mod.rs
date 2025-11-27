//! Error types for the Maguffin application.
//!
//! This module provides a unified error handling system using `thiserror`
//! for defining error types and `anyhow` for error context propagation.

use thiserror::Error;

/// The main error type for the application.
#[derive(Error, Debug)]
pub enum AppError {
    /// Git operation failed
    #[error("Git operation failed: {0}")]
    Git(#[from] GitError),

    /// GitHub API error
    #[error("GitHub API error: {0}")]
    GitHub(#[from] GitHubError),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    /// Storage error (database or keyring)
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Errors related to Git operations.
#[derive(Error, Debug)]
pub enum GitError {
    /// Repository not found or not valid
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    /// Branch operation failed
    #[error("Branch error: {0}")]
    Branch(String),

    /// Rebase operation failed
    #[error("Rebase failed: {0}")]
    RebaseFailed(String),

    /// Merge conflict detected
    #[error("Merge conflict in: {files:?}")]
    Conflict { files: Vec<String> },

    /// Remote operation failed
    #[error("Remote operation failed: {0}")]
    Remote(String),

    /// Underlying git2 error
    #[error("git2 error: {0}")]
    Git2(#[from] git2::Error),
}

/// Errors related to GitHub API operations.
#[derive(Error, Debug)]
pub enum GitHubError {
    /// GraphQL query error
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Resets at: {reset_at}")]
    RateLimited { reset_at: String },

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Unauthorized (invalid token)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),
}

impl From<reqwest::Error> for GitHubError {
    fn from(err: reqwest::Error) -> Self {
        GitHubError::Http(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::GitHub(GitHubError::from(err))
    }
}

/// Errors related to authentication.
#[derive(Error, Debug)]
pub enum AuthError {
    /// OAuth flow failed
    #[error("OAuth flow failed: {0}")]
    OAuthFailed(String),

    /// Token expired or invalid
    #[error("Token expired or invalid")]
    TokenExpired,

    /// No credentials stored
    #[error("No credentials found")]
    NoCredentials,

    /// Keyring access failed
    #[error("Keyring error: {0}")]
    Keyring(String),
}

/// Errors related to storage operations.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLite error
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Result type alias using AppError.
pub type Result<T> = std::result::Result<T, AppError>;

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_error_display() {
        let err = GitError::RepositoryNotFound("/path/to/repo".to_string());
        assert!(err.to_string().contains("/path/to/repo"));
    }

    #[test]
    fn test_github_error_display() {
        let err = GitHubError::RateLimited {
            reset_at: "2025-11-27T12:00:00Z".to_string(),
        };
        assert!(err.to_string().contains("Rate limit"));
    }

    #[test]
    fn test_app_error_from_git_error() {
        let git_err = GitError::Branch("test branch".to_string());
        let app_err: AppError = git_err.into();
        assert!(matches!(app_err, AppError::Git(_)));
    }
}
