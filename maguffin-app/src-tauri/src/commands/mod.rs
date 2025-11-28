//! Tauri command handlers.
//!
//! This module contains the IPC command handlers that bridge the
//! frontend UI to the Rust backend.

use crate::domain::repo::GitHubRemote;
use crate::domain::{AuthState, PullRequest, Repository, SyncState};
use crate::error::AppError;
use crate::git::{Git2Backend, GitOperations};
use crate::github::{AuthService, GitHubClient, PrService};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state managed by Tauri.
///
/// Contains all services needed for the application:
/// - AuthService for authentication
/// - GitHubClient for API calls
/// - Current repository context
pub struct AppState {
    /// Authentication service
    auth_service: AuthService,

    /// GitHub API client
    github_client: Arc<GitHubClient>,

    /// Current repository context (owner, repo, path)
    current_repo: Arc<RwLock<Option<RepoContext>>>,
}

/// Context for the currently opened repository.
#[derive(Debug, Clone)]
pub struct RepoContext {
    /// Local path to the repository
    pub path: PathBuf,
    /// GitHub owner
    pub owner: String,
    /// GitHub repo name
    pub name: String,
    /// Current branch
    pub current_branch: String,
    /// Default branch
    pub default_branch: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Create a new application state.
    pub fn new() -> Self {
        let github_client = GitHubClient::new("https://api.github.com/graphql".to_string())
            .expect("Failed to create GitHub client");

        Self {
            auth_service: AuthService::new().expect("Failed to create AuthService"),
            github_client: Arc::new(github_client),
            current_repo: Arc::new(RwLock::new(None)),
        }
    }
}

/// Get the current authentication state.
#[tauri::command]
pub async fn get_auth_state(state: State<'_, AppState>) -> Result<AuthState, String> {
    // Try to restore from keyring first
    match state.auth_service.try_restore().await {
        Ok(auth_state) => {
            // If authenticated, set the token on the GitHub client
            if let AuthState::Authenticated(_) = &auth_state {
                if let Ok(Some(token)) = state.auth_service.get_token() {
                    state.github_client.set_token(token).await;
                }
            }
            Ok(auth_state)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Start the GitHub OAuth device flow.
#[tauri::command]
pub async fn start_device_flow(state: State<'_, AppState>) -> Result<AuthState, String> {
    state
        .auth_service
        .start_device_flow()
        .await
        .map_err(|e| e.to_string())
}

/// Poll the device flow for completion.
#[tauri::command]
pub async fn poll_device_flow(state: State<'_, AppState>) -> Result<AuthState, String> {
    match state.auth_service.poll_device_flow().await {
        Ok(auth_state) => {
            // If authenticated, set the token on the GitHub client
            if let AuthState::Authenticated(_) = &auth_state {
                if let Ok(Some(token)) = state.auth_service.get_token() {
                    state.github_client.set_token(token).await;
                }
            }
            Ok(auth_state)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Log out and clear credentials.
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    state.github_client.clear_token().await;
    state
        .auth_service
        .logout()
        .await
        .map_err(|e| e.to_string())
}

/// Open a local repository.
#[tauri::command]
pub async fn open_repository(
    state: State<'_, AppState>,
    path: String,
) -> Result<Repository, String> {
    let path = PathBuf::from(&path);

    // All git operations in a block to ensure they're dropped before await
    let (current_branch, default_branch, remote_url, github_remote) = {
        // Open the git repository
        let git = Git2Backend::discover(&path).map_err(|e| e.to_string())?;

        // Get current branch
        let current_branch = git.current_branch().map_err(|e| e.to_string())?;

        // Get default branch
        let default_branch = git.default_branch().unwrap_or_else(|_| "main".to_string());

        // Get remote URL and parse GitHub info
        let repo = git2::Repository::open(&path).map_err(|e| e.to_string())?;
        let remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        let remote_url = remote.url().ok_or("No remote URL found")?.to_string();

        let github_remote =
            GitHubRemote::parse(&remote_url).ok_or("Could not parse GitHub remote URL")?;

        (current_branch, default_branch, remote_url, github_remote)
    };

    // Store the repo context
    let context = RepoContext {
        path: path.clone(),
        owner: github_remote.owner.clone(),
        name: github_remote.name.clone(),
        current_branch: current_branch.clone(),
        default_branch: default_branch.clone(),
    };
    *state.current_repo.write().await = Some(context);

    Ok(Repository {
        path,
        owner: github_remote.owner,
        name: github_remote.name,
        current_branch,
        default_branch,
        remote_url,
        sync_state: SyncState::Unknown,
    })
}

/// List pull requests for the current repository.
#[tauri::command]
pub async fn list_pull_requests(
    state: State<'_, AppState>,
    base_branch: Option<String>,
) -> Result<Vec<PullRequest>, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let pr_service = PrService::new(
        state.github_client.clone(),
        repo.owner.clone(),
        repo.name.clone(),
    );

    pr_service
        .list_prs(base_branch)
        .await
        .map_err(|e| e.to_string())
}

/// Get details for a specific pull request.
#[tauri::command]
pub async fn get_pull_request(
    state: State<'_, AppState>,
    number: i64,
) -> Result<PullRequest, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let pr_service = PrService::new(
        state.github_client.clone(),
        repo.owner.clone(),
        repo.name.clone(),
    );

    let details = pr_service
        .get_pr_details(number)
        .await
        .map_err(|e| e.to_string())?;

    Ok(details.pr)
}

/// Checkout a PR branch locally.
#[tauri::command]
pub async fn checkout_pull_request(
    state: State<'_, AppState>,
    number: i64,
) -> Result<(), String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    // Get PR details to find the branch name
    let pr_service = PrService::new(
        state.github_client.clone(),
        repo.owner.clone(),
        repo.name.clone(),
    );

    let details = pr_service
        .get_pr_details(number)
        .await
        .map_err(|e| e.to_string())?;

    let branch_name = details.pr.head_ref;

    // Checkout the branch
    let git = Git2Backend::open(&repo.path).map_err(|e| e.to_string())?;

    // First try to checkout existing branch
    if git.branch_exists(&branch_name).map_err(|e| e.to_string())? {
        git.checkout_branch(&branch_name)
            .map_err(|e| e.to_string())?;
    } else {
        // Need to fetch the branch first
        git.fetch("origin").map_err(|e| e.to_string())?;

        // Create local tracking branch
        let git_repo = git2::Repository::open(&repo.path).map_err(|e| e.to_string())?;
        let remote_branch = format!("origin/{}", branch_name);
        let remote_ref = git_repo
            .find_reference(&format!("refs/remotes/{}", remote_branch))
            .map_err(|e| format!("Remote branch not found: {}", e))?;
        let commit = remote_ref
            .peel_to_commit()
            .map_err(|e| e.to_string())?;
        git_repo
            .branch(&branch_name, &commit, false)
            .map_err(|e| e.to_string())?;

        git.checkout_branch(&branch_name)
            .map_err(|e| e.to_string())?;
    }

    // Update current branch in context
    let mut context = state.current_repo.write().await;
    if let Some(ref mut ctx) = *context {
        ctx.current_branch = branch_name;
    }

    Ok(())
}

/// Generate all command handlers for registration.
pub fn generate_handlers() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        get_auth_state,
        start_device_flow,
        poll_device_flow,
        logout,
        open_repository,
        list_pull_requests,
        get_pull_request,
        checkout_pull_request,
    ]
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
