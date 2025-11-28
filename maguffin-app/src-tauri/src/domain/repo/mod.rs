//! Repository domain types.
//!
//! This module handles local Git repository detection and GitHub remote parsing.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;

/// Custom serializer for PathBuf that ensures consistent forward slashes across platforms.
/// This is important for frontend compatibility as Windows uses backslashes.
fn serialize_path<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert path to string with forward slashes for cross-platform consistency
    let path_str = path.to_string_lossy().replace('\\', "/");
    serializer.serialize_str(&path_str)
}

/// Custom deserializer for PathBuf that handles both forward and back slashes.
fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(PathBuf::from(s))
}

/// Represents a local Git repository with GitHub remote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Local path to the repository (serialized with forward slashes for cross-platform consistency)
    #[serde(serialize_with = "serialize_path", deserialize_with = "deserialize_path")]
    pub path: PathBuf,

    /// GitHub owner (user or organization)
    pub owner: String,

    /// GitHub repository name
    pub name: String,

    /// Current branch name
    pub current_branch: String,

    /// Default branch (usually main or master)
    pub default_branch: String,

    /// Remote URL
    pub remote_url: String,

    /// Current sync state
    pub sync_state: SyncState,
}

impl Repository {
    /// Get the full repository name (owner/name).
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

/// Sync state of the repository relative to remote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncState {
    /// Up to date with remote
    UpToDate,

    /// Local has commits not on remote
    Ahead { commits: u32 },

    /// Remote has commits not on local
    Behind { commits: u32 },

    /// Both local and remote have diverged
    Diverged { ahead: u32, behind: u32 },

    /// Unknown state (needs fetch)
    Unknown,
}

/// Parsed GitHub remote information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRemote {
    /// Owner (user or org)
    pub owner: String,

    /// Repository name
    pub name: String,

    /// Whether this is GitHub Enterprise
    pub is_enterprise: bool,

    /// Host (github.com or enterprise domain)
    pub host: String,
}

impl GitHubRemote {
    /// Parse a GitHub remote URL.
    ///
    /// Supports formats:
    /// - `https://github.com/owner/repo.git`
    /// - `https://github.com/owner/repo`
    /// - `git@github.com:owner/repo.git`
    /// - `git@github.com:owner/repo`
    /// - `ssh://git@github.com/owner/repo.git`
    pub fn parse(url: &str) -> Option<Self> {
        // Try HTTPS format
        if let Some(parsed) = Self::parse_https(url) {
            return Some(parsed);
        }

        // Try SSH format (git@...)
        if let Some(parsed) = Self::parse_ssh_short(url) {
            return Some(parsed);
        }

        // Try SSH URL format (ssh://...)
        if let Some(parsed) = Self::parse_ssh_url(url) {
            return Some(parsed);
        }

        None
    }

    fn parse_https(url: &str) -> Option<Self> {
        let url = url.strip_prefix("https://")?;
        let (host, path) = url.split_once('/')?;
        Self::parse_path(host, path)
    }

    fn parse_ssh_short(url: &str) -> Option<Self> {
        let url = url.strip_prefix("git@")?;
        let (host, path) = url.split_once(':')?;
        Self::parse_path(host, path)
    }

    fn parse_ssh_url(url: &str) -> Option<Self> {
        let url = url.strip_prefix("ssh://")?;
        let url = url.strip_prefix("git@")?;
        let (host, path) = url.split_once('/')?;
        Self::parse_path(host, path)
    }

    fn parse_path(host: &str, path: &str) -> Option<Self> {
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 2 {
            return None;
        }

        let owner = parts[0].to_string();
        let name = parts[1].to_string();

        if owner.is_empty() || name.is_empty() {
            return None;
        }

        let is_enterprise = host != "github.com";

        Some(Self {
            owner,
            name,
            is_enterprise,
            host: host.to_string(),
        })
    }

    /// Get the full repository name (owner/name).
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// Get the GitHub web URL for this repository.
    pub fn web_url(&self) -> String {
        format!("https://{}/{}/{}", self.host, self.owner, self.name)
    }
}

/// Recent repository entry for quick access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRepository {
    /// Local path (serialized with forward slashes for cross-platform consistency)
    #[serde(serialize_with = "serialize_path", deserialize_with = "deserialize_path")]
    pub path: PathBuf,

    /// Repository full name
    pub full_name: String,

    /// Last opened timestamp
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_url() {
        let remote = GitHubRemote::parse("https://github.com/owner/repo.git").unwrap();
        assert_eq!(remote.owner, "owner");
        assert_eq!(remote.name, "repo");
        assert!(!remote.is_enterprise);
    }

    #[test]
    fn test_parse_https_url_without_git() {
        let remote = GitHubRemote::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(remote.owner, "owner");
        assert_eq!(remote.name, "repo");
    }

    #[test]
    fn test_parse_ssh_short() {
        let remote = GitHubRemote::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(remote.owner, "owner");
        assert_eq!(remote.name, "repo");
    }

    #[test]
    fn test_parse_ssh_url() {
        let remote = GitHubRemote::parse("ssh://git@github.com/owner/repo.git").unwrap();
        assert_eq!(remote.owner, "owner");
        assert_eq!(remote.name, "repo");
    }

    #[test]
    fn test_parse_enterprise() {
        let remote = GitHubRemote::parse("https://github.enterprise.com/owner/repo").unwrap();
        assert_eq!(remote.owner, "owner");
        assert_eq!(remote.name, "repo");
        assert!(remote.is_enterprise);
        assert_eq!(remote.host, "github.enterprise.com");
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(GitHubRemote::parse("not-a-url").is_none());
        assert!(GitHubRemote::parse("https://github.com/").is_none());
        assert!(GitHubRemote::parse("https://github.com/owner").is_none());
    }

    #[test]
    fn test_web_url() {
        let remote = GitHubRemote::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(remote.web_url(), "https://github.com/owner/repo");
    }

    #[test]
    fn test_sync_state_serialization() {
        let state = SyncState::Ahead { commits: 5 };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("ahead"));
    }

    #[test]
    fn test_path_serialization_uses_forward_slashes() {
        let repo = Repository {
            path: PathBuf::from("/home/user/project"),
            owner: "owner".to_string(),
            name: "repo".to_string(),
            current_branch: "main".to_string(),
            default_branch: "main".to_string(),
            remote_url: "https://github.com/owner/repo.git".to_string(),
            sync_state: SyncState::Unknown,
        };
        let json = serde_json::to_string(&repo).unwrap();
        // Verify path is serialized with forward slashes (no backslashes)
        assert!(!json.contains("\\\\"));
        assert!(json.contains("/home/user/project"));
    }
}
