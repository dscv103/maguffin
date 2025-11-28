//! Tauri command handlers.
//!
//! This module contains the IPC command handlers that bridge the
//! frontend UI to the Rust backend.

use crate::cache::{Cache, RecentRepository};
use crate::domain::pr::PullRequestDetails;
use crate::domain::repo::GitHubRemote;
use crate::domain::stack::{RestackResult, Stack};
use crate::domain::{AuthState, PullRequest, Repository, SyncState};
use crate::error::AppError;
use crate::git::{Git2Backend, GitOperations};
use crate::github::{AuthService, GitHubClient, PrService, StackService};
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
/// - Cache for persistent storage
pub struct AppState {
    /// Authentication service
    auth_service: AuthService,

    /// GitHub API client
    github_client: Arc<GitHubClient>,

    /// Current repository context (owner, repo, path)
    current_repo: Arc<RwLock<Option<RepoContext>>>,

    /// Local cache for recent repositories and settings
    cache: Arc<Cache>,
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
    ///
    /// This method handles service initialization failures gracefully by using
    /// fallback/default services where possible, rather than panicking.
    pub fn new() -> Self {
        let github_client = GitHubClient::new("https://api.github.com/graphql".to_string())
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create GitHub client, using default: {}", e);
                GitHubClient::default()
            });

        let auth_service = AuthService::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create AuthService, using default: {}", e);
            AuthService::default()
        });

        // Create cache in the user's data directory
        let cache = Self::create_cache();

        Self {
            auth_service,
            github_client: Arc::new(github_client),
            current_repo: Arc::new(RwLock::new(None)),
            cache: Arc::new(cache),
        }
    }

    /// Create cache, falling back to in-memory if file-based fails.
    fn create_cache() -> Cache {
        // Try to get user's data directory
        if let Some(data_dir) = dirs::data_dir() {
            let cache_dir = data_dir.join("maguffin");
            if std::fs::create_dir_all(&cache_dir).is_ok() {
                let cache_path = cache_dir.join("cache.db");
                if let Ok(cache) = Cache::open(&cache_path) {
                    return cache;
                }
            }
        }
        // Fall back to in-memory cache
        Cache::in_memory().expect("Failed to create in-memory cache")
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
    state.auth_service.logout().await.map_err(|e| e.to_string())
}

/// Open a local repository.
#[tauri::command]
pub async fn open_repository(
    state: State<'_, AppState>,
    path: String,
) -> Result<Repository, String> {
    let path = PathBuf::from(&path);

    // Wrap git operations in spawn_blocking to avoid blocking the async runtime
    let (current_branch, default_branch, remote_url, github_remote) =
        tokio::task::spawn_blocking({
            let path = path.clone();
            move || {
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

                Ok::<_, String>((current_branch, default_branch, remote_url, github_remote))
            }
        })
        .await
        .map_err(|e| format!("Git operation task failed: {:?}", e))??;

    // Store the repo context
    let context = RepoContext {
        path: path.clone(),
        owner: github_remote.owner.clone(),
        name: github_remote.name.clone(),
        current_branch: current_branch.clone(),
        default_branch: default_branch.clone(),
    };
    *state.current_repo.write().await = Some(context);

    // Save to recent repositories
    let path_str = path.to_string_lossy().to_string();
    let _ = state.cache.save_recent_repository(
        &path_str,
        &github_remote.owner,
        &github_remote.name,
    );

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

/// Get list of recent repositories.
#[tauri::command]
pub async fn get_recent_repositories(
    state: State<'_, AppState>,
) -> Result<Vec<RecentRepository>, String> {
    state
        .cache
        .get_recent_repositories(10)
        .map_err(|e| e.to_string())
}

/// Remove a repository from recent list.
#[tauri::command]
pub async fn remove_recent_repository(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    state
        .cache
        .remove_recent_repository(&path)
        .map_err(|e| e.to_string())
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

/// Get full details for a specific pull request including CI status.
#[tauri::command]
pub async fn get_pull_request_details(
    state: State<'_, AppState>,
    number: i64,
) -> Result<PullRequestDetails, String> {
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
        .get_pr_details(number)
        .await
        .map_err(|e| e.to_string())
}

/// Checkout a PR branch locally.
#[tauri::command]
pub async fn checkout_pull_request(state: State<'_, AppState>, number: i64) -> Result<(), String> {
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

    // Checkout the branch (wrap in spawn_blocking to avoid blocking async runtime)
    tokio::task::spawn_blocking({
        let repo_path = repo.path.clone();
        let branch_name = branch_name.clone();
        move || {
            let git = Git2Backend::open(&repo_path).map_err(|e| e.to_string())?;

            // First try to checkout existing branch
            if git.branch_exists(&branch_name).map_err(|e| e.to_string())? {
                git.checkout_branch(&branch_name)
                    .map_err(|e| e.to_string())?;
            } else {
                // Need to fetch the branch first
                git.fetch("origin").map_err(|e| e.to_string())?;

                // Create local tracking branch
                let git_repo = git2::Repository::open(&repo_path).map_err(|e| e.to_string())?;
                let remote_branch = format!("origin/{}", branch_name);
                let remote_ref = git_repo
                    .find_reference(&format!("refs/remotes/{}", remote_branch))
                    .map_err(|e| format!("Remote branch not found: {}", e))?;
                let commit = remote_ref.peel_to_commit().map_err(|e| e.to_string())?;
                git_repo
                    .branch(&branch_name, &commit, false)
                    .map_err(|e| e.to_string())?;

                git.checkout_branch(&branch_name)
                    .map_err(|e| e.to_string())?;
            }
            Ok::<_, String>(())
        }
    })
    .await
    .map_err(|e| format!("Git operation panicked: {}", e))??;

    // Update current branch in context
    let mut context = state.current_repo.write().await;
    if let Some(ref mut ctx) = *context {
        ctx.current_branch = branch_name;
    }

    Ok(())
}

/// Create a new pull request.
#[tauri::command]
pub async fn create_pull_request(
    state: State<'_, AppState>,
    title: String,
    body: Option<String>,
    head: String,
    base: String,
    draft: bool,
) -> Result<i64, String> {
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
        .create_pr(title, body, head, base, draft)
        .await
        .map_err(|e| e.to_string())
}

/// Merge a pull request.
#[tauri::command]
pub async fn merge_pull_request(
    state: State<'_, AppState>,
    pr_id: String,
    merge_method: String,
) -> Result<bool, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let method = match merge_method.to_uppercase().as_str() {
        "MERGE" => crate::domain::pr::MergeMethod::Merge,
        "SQUASH" => crate::domain::pr::MergeMethod::Squash,
        "REBASE" => crate::domain::pr::MergeMethod::Rebase,
        _ => return Err(format!("Invalid merge method: {}", merge_method)),
    };

    let pr_service = PrService::new(
        state.github_client.clone(),
        repo.owner.clone(),
        repo.name.clone(),
    );

    pr_service
        .merge_pr(pr_id, method)
        .await
        .map_err(|e| e.to_string())
}

/// Close a pull request without merging.
#[tauri::command]
pub async fn close_pull_request(
    state: State<'_, AppState>,
    pr_id: String,
) -> Result<bool, String> {
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

    pr_service.close_pr(pr_id).await.map_err(|e| e.to_string())
}

/// List all stacks in the current repository.
#[tauri::command]
pub async fn list_stacks(state: State<'_, AppState>) -> Result<Vec<Stack>, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let repo_path = repo.path;

    // Read stack metadata directly from file (no git operations needed)
    tokio::task::spawn_blocking(move || {
        let metadata_path = repo_path.join(".git").join("stack-metadata.json");
        if metadata_path.exists() {
            let content = std::fs::read_to_string(&metadata_path)
                .map_err(|e| format!("Failed to read stack metadata: {}", e))?;
            let metadata: crate::domain::stack::StackMetadata = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse stack metadata: {}", e))?;
            Ok(metadata.stacks)
        } else {
            Ok(Vec::new())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Create a new stack rooted at the given branch.
#[tauri::command]
pub async fn create_stack(
    state: State<'_, AppState>,
    root_branch: String,
) -> Result<Stack, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let repo_path = repo.path.clone();

    // Run entire operation in spawn_blocking since StackService contains non-Send types
    tokio::task::spawn_blocking(move || {
        let git = Git2Backend::open(&repo_path).map_err(|e| e.to_string())?;
        let stack_service = StackService::new(repo.path, git).map_err(|e| e.to_string())?;

        // Use block_on for the async method since we're in a blocking context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { stack_service.create_stack(root_branch).await })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {:?}", e))?
}

/// Create a new branch on an existing stack.
#[tauri::command]
pub async fn create_stack_branch(
    state: State<'_, AppState>,
    stack_id: String,
    branch_name: String,
    parent_name: String,
) -> Result<(), String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let repo_path = repo.path.clone();

    // Run entire operation in spawn_blocking since StackService contains non-Send types
    tokio::task::spawn_blocking(move || {
        let git = Git2Backend::open(&repo_path).map_err(|e| e.to_string())?;
        let stack_service = StackService::new(repo.path, git).map_err(|e| e.to_string())?;
        let stack_uuid = uuid::Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

        // Use block_on for the async method since we're in a blocking context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            stack_service
                .create_stack_branch(stack_uuid, branch_name, parent_name)
                .await
        })
        .map_err(|e| e.to_string())?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("Task failed: {:?}", e))?
}

/// Restack all branches in a stack.
#[tauri::command]
pub async fn restack(
    state: State<'_, AppState>,
    stack_id: String,
) -> Result<RestackResult, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let repo_path = repo.path.clone();

    // Run entire operation in spawn_blocking since StackService contains non-Send types
    tokio::task::spawn_blocking(move || {
        let git = Git2Backend::open(&repo_path).map_err(|e| e.to_string())?;
        let stack_service = StackService::new(repo.path, git).map_err(|e| e.to_string())?;
        let stack_uuid = uuid::Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

        // Use block_on for the async method since we're in a blocking context
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { stack_service.restack(stack_uuid).await })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {:?}", e))?
}

/// Create a PR for a branch in a stack with the correct base branch.
#[tauri::command]
pub async fn create_stack_pr(
    state: State<'_, AppState>,
    stack_id: String,
    branch_name: String,
    title: String,
    body: Option<String>,
    draft: bool,
) -> Result<i64, String> {
    let repo = state
        .current_repo
        .read()
        .await
        .clone()
        .ok_or("No repository opened")?;

    let repo_path = repo.path.clone();

    // First, find the parent branch from the stack metadata
    let parent_branch = tokio::task::spawn_blocking({
        let repo_path = repo_path.clone();
        let stack_id = stack_id.clone();
        let branch_name = branch_name.clone();
        move || {
            let git = Git2Backend::open(&repo_path).map_err(|e| e.to_string())?;
            let stack_service = StackService::new(repo_path, git).map_err(|e| e.to_string())?;
            let stack_uuid = uuid::Uuid::parse_str(&stack_id).map_err(|e| e.to_string())?;

            let rt = tokio::runtime::Handle::current();
            let stack = rt
                .block_on(async { stack_service.get_stack(stack_uuid).await })
                .ok_or("Stack not found")?;

            let branch = stack
                .branches
                .iter()
                .find(|b| b.name == branch_name)
                .ok_or("Branch not found in stack")?;

            Ok::<String, String>(branch.parent.clone())
        }
    })
    .await
    .map_err(|e| format!("Task failed: {:?}", e))??;

    // Create the PR with the parent branch as base
    let pr_service = PrService::new(
        state.github_client.clone(),
        repo.owner.clone(),
        repo.name.clone(),
    );

    let pr_number = pr_service
        .create_pr(title, body, branch_name.clone(), parent_branch, draft)
        .await
        .map_err(|e| e.to_string())?;

    // Update the stack metadata with the PR number
    tokio::task::spawn_blocking({
        let repo_path = repo.path;
        move || {
            let git = Git2Backend::open(&repo_path).map_err(|e| e.to_string())?;
            let stack_service = StackService::new(repo_path, git).map_err(|e| e.to_string())?;

            let rt = tokio::runtime::Handle::current();
            rt.block_on(async { stack_service.set_branch_pr(&branch_name, pr_number).await })
                .map_err(|e| e.to_string())
        }
    })
    .await
    .map_err(|e| format!("Task failed: {:?}", e))??;

    Ok(pr_number)
}

/// Generate all command handlers for registration.
pub fn generate_handlers() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        get_auth_state,
        start_device_flow,
        poll_device_flow,
        logout,
        open_repository,
        get_recent_repositories,
        remove_recent_repository,
        list_pull_requests,
        get_pull_request,
        get_pull_request_details,
        checkout_pull_request,
        create_pull_request,
        merge_pull_request,
        close_pull_request,
        list_stacks,
        create_stack,
        create_stack_branch,
        create_stack_pr,
        restack,
    ]
}

impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}
