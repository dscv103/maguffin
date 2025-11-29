import { useState, useCallback, useEffect, useRef } from "react";
import { AuthView, PRDashboard, PRDetailPanel, StackList, RepoSelector, ThemeToggle, KeyboardShortcutsHelp, SyncStatusIndicator, ErrorBoundary, ViewErrorFallback, ConflictResolutionDialog, OnboardingFlow, useOnboarding } from "./components";
import { useAuth, useStacks, useRepository, usePullRequests, useTheme, useAppKeyboardShortcuts, AVAILABLE_SHORTCUTS, useSync } from "./hooks";
import type { PullRequest, Stack, RestackResult, ReconcileReport } from "./types";

type View = "auth" | "dashboard" | "stacks" | "settings";

function App() {
  const { authState } = useAuth();
  const { repository, recentRepositories, loading: repoLoading, error: repoError, openRepository, removeRecentRepository, clearRepository, clearError: clearRepoError } = useRepository();
  const { stacks, loading: stacksLoading, error: stacksError, restackStack, reconcileStacks, refresh: refreshStacks } = useStacks(repository);
  const { refresh: refreshPRs } = usePullRequests();
  const { theme, setTheme, toggleTheme } = useTheme();
  const { status: syncStatus, config: syncConfig, syncNow, loading: syncLoading, startSync, updateConfig, error: syncError, clearError: clearSyncError } = useSync();
  const { showOnboarding, completeOnboarding, resetOnboarding } = useOnboarding();
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [selectedPR, setSelectedPR] = useState<PullRequest | null>(null);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const [restackResult, setRestackResult] = useState<RestackResult | null>(null);
  const [currentRestackStackId, setCurrentRestackStackId] = useState<string | null>(null);
  const [reconcileReport, setReconcileReport] = useState<ReconcileReport | null>(null);
  const [reconciling, setReconciling] = useState(false);

  const isAuthenticated = authState.type === "authenticated";
  
  // Track if sync has been initialized for this session
  const syncInitializedRef = useRef(false);

  // Start sync when authenticated and repository is open (only once)
  useEffect(() => {
    if (isAuthenticated && repository && !syncInitializedRef.current) {
      syncInitializedRef.current = true;
      startSync();
    }
  }, [isAuthenticated, repository, startSync]);

  // Define keyboard shortcut callbacks
  const onNavigateDashboard = useCallback(() => setCurrentView("dashboard"), []);
  const onNavigateStacks = useCallback(() => setCurrentView("stacks"), []);
  const onNavigateSettings = useCallback(() => setCurrentView("settings"), []);
  const onRefresh = useCallback(() => refreshPRs(), [refreshPRs]);
  const onToggleTheme = useCallback(() => toggleTheme(), [toggleTheme]);
  const onEscape = useCallback(() => {
    if (showShortcuts) {
      setShowShortcuts(false);
    } else if (selectedPR) {
      setSelectedPR(null);
    }
  }, [showShortcuts, selectedPR]);

  // Keyboard shortcuts
  useAppKeyboardShortcuts(
    {
      onNavigateDashboard,
      onNavigateStacks,
      onNavigateSettings,
      onRefresh,
      onToggleTheme,
      onEscape,
    },
    isAuthenticated
  );

  // Add ? shortcut for showing keyboard shortcuts help
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === "?" && !e.ctrlKey && !e.altKey && !e.metaKey) {
      const target = e.target as HTMLElement;
      if (target.tagName !== "INPUT" && target.tagName !== "TEXTAREA" && !target.isContentEditable) {
        e.preventDefault();
        setShowShortcuts(true);
      }
    }
  }, []);

  // Register ? key handler
  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Show onboarding for first-time users (before auth)
  if (showOnboarding) {
    return (
      <div className="app">
        <OnboardingFlow 
          onComplete={completeOnboarding} 
          onSkip={completeOnboarding} 
        />
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="app">
        <AuthView onAuthenticated={() => setCurrentView("dashboard")} />
      </div>
    );
  }

  const handleRestack = async (stack: Stack) => {
    setCurrentRestackStackId(stack.id);
    const result = await restackStack(stack.id);
    if (result) {
      setRestackResult(result);
    }
  };

  const handleRetryRestack = async () => {
    if (currentRestackStackId) {
      const result = await restackStack(currentRestackStackId);
      if (result) {
        setRestackResult(result);
      }
    }
  };

  const handleCloseRestackDialog = () => {
    setRestackResult(null);
    setCurrentRestackStackId(null);
  };

  const handleReconcile = async () => {
    setReconciling(true);
    setReconcileReport(null);
    const report = await reconcileStacks();
    setReconciling(false);
    if (report && (report.orphaned.length > 0 || report.warnings.length > 0)) {
      setReconcileReport(report);
    }
  };

  const handlePRActionComplete = () => {
    // Refresh the PR list after an action is completed
    refreshPRs();
  };

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="sidebar-header">
          <h1 className="app-logo">Maguffin</h1>
          <RepoSelector
            repository={repository}
            recentRepositories={recentRepositories}
            loading={repoLoading}
            error={repoError}
            onOpenRepository={openRepository}
            onClearRepository={clearRepository}
            onRemoveRecentRepository={removeRecentRepository}
            onClearError={clearRepoError}
          />
        </div>

        <ul className="nav-menu">
          <li>
            <button
              className={currentView === "dashboard" ? "active" : ""}
              onClick={() => setCurrentView("dashboard")}
            >
              <span className="nav-icon">📋</span>
              <span className="nav-label">Pull Requests</span>
            </button>
          </li>
          <li>
            <button
              className={currentView === "stacks" ? "active" : ""}
              onClick={() => setCurrentView("stacks")}
            >
              <span className="nav-icon">📚</span>
              <span className="nav-label">Stacks</span>
            </button>
          </li>
          <li>
            <button
              className={currentView === "settings" ? "active" : ""}
              onClick={() => setCurrentView("settings")}
            >
              <span className="nav-icon">⚙️</span>
              <span className="nav-label">Settings</span>
            </button>
          </li>
        </ul>

        <div className="sidebar-footer">
          {repository && (
            <SyncStatusIndicator
              status={syncStatus}
              onSyncNow={syncNow}
              loading={syncLoading}
            />
          )}
          <ThemeToggle />
          <AuthView />
        </div>
      </nav>

      <main className="main-content">
        {!repository ? (
          <div className="no-repo-view">
            <h1>Welcome to Maguffin</h1>
            <p>Open a Git repository to get started</p>
            <p className="hint">Enter a path to a local Git repository in the sidebar</p>
          </div>
        ) : (
          <>
            {currentView === "dashboard" && (
              <ErrorBoundary fallback={<ViewErrorFallback message="Failed to load pull requests dashboard" />}>
                <PRDashboard onSelectPR={(pr) => setSelectedPR(pr)} />
              </ErrorBoundary>
            )}

            {currentView === "stacks" && (
              <ErrorBoundary fallback={<ViewErrorFallback message="Failed to load stacks view" />}>
                <div className="stacks-view">
                  <div className="stacks-header">
                    <h1>Stacks</h1>
                    <div className="stacks-actions">
                      <button 
                        className="reconcile-btn" 
                        onClick={handleReconcile}
                        disabled={reconciling || stacks.length === 0}
                      >
                        {reconciling ? "Checking..." : "🔄 Reconcile"}
                      </button>
                      <button 
                        className="refresh-btn" 
                        onClick={() => refreshStacks()}
                        disabled={stacksLoading}
                      >
                        ↻ Refresh
                      </button>
                    </div>
                  </div>

                  {reconcileReport && (
                    <div className="reconcile-report">
                      <h3>Reconciliation Report</h3>
                      {reconcileReport.orphaned.length > 0 && (
                        <div className="orphaned-branches">
                          <h4>⚠️ Orphaned Branches</h4>
                          <p>These branches were deleted externally:</p>
                          <ul>
                            {reconcileReport.orphaned.map((branch) => (
                              <li key={branch}><code>{branch}</code></li>
                            ))}
                          </ul>
                        </div>
                      )}
                      {reconcileReport.warnings.length > 0 && (
                        <div className="reconcile-warnings">
                          <h4>⚠️ Warnings</h4>
                          <ul>
                            {reconcileReport.warnings.map((warning, idx) => (
                              <li key={idx}>
                                <code>{warning.branch}</code>: {
                                  warning.warning === "parent_not_ancestor" ? "Parent is not an ancestor (branch was rebased externally)" :
                                  warning.warning === "externally_modified" ? "Branch was modified externally" :
                                  warning.warning === "parent_deleted" ? "Parent branch was deleted" :
                                  warning.warning
                                }
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}
                      <button className="dismiss-btn" onClick={() => setReconcileReport(null)}>
                        Dismiss
                      </button>
                    </div>
                  )}

                  {stacksLoading ? (
                    <div className="loading">
                      <div className="spinner" />
                      <p>Loading stacks...</p>
                    </div>
                  ) : stacksError ? (
                    <div className="error">
                      <p className="error-message">{stacksError}</p>
                    </div>
                  ) : stacks.length === 0 ? (
                    <div className="empty-state">
                      <p>No stacks found</p>
                      <p className="hint">Create a stack to organize your branches</p>
                    </div>
                  ) : (
                    <StackList stacks={stacks} onRestack={handleRestack} />
                  )}
                </div>
              </ErrorBoundary>
            )}

            {currentView === "settings" && (
              <div className="settings-view">
                <h1>Settings</h1>
                
                <section className="settings-section">
                  <h2>Appearance</h2>
                  <div className="setting-item">
                    <label className="setting-label">Theme</label>
                    <div className="theme-options">
                      <button 
                        className={`theme-option ${theme === "dark" ? "active" : ""}`}
                        onClick={() => setTheme("dark")}
                      >
                        🌙 Dark
                      </button>
                      <button 
                        className={`theme-option ${theme === "light" ? "active" : ""}`}
                        onClick={() => setTheme("light")}
                      >
                        ☀️ Light
                      </button>
                    </div>
                  </div>
                </section>

                <section className="settings-section">
                  <h2>Synchronization</h2>
                  <div className="setting-item">
                    <label className="setting-label">Enable Sync</label>
                    <label className="toggle-switch">
                      <input 
                        type="checkbox" 
                        checked={syncConfig.enabled}
                        onChange={(e) => updateConfig(syncConfig.interval_secs, e.target.checked)}
                      />
                      <span className="toggle-slider"></span>
                    </label>
                  </div>
                  <div className="setting-item">
                    <label className="setting-label">Sync Interval</label>
                    <select
                      className="setting-select"
                      value={syncConfig.interval_secs}
                      onChange={(e) => updateConfig(parseInt(e.target.value, 10), syncConfig.enabled)}
                    >
                      <option value="30">30 seconds</option>
                      <option value="60">1 minute</option>
                      <option value="120">2 minutes</option>
                      <option value="300">5 minutes</option>
                    </select>
                  </div>
                  <div className="setting-item">
                    <p className="setting-description">
                      Current status: {
                        (() => {
                          switch (syncStatus.status) {
                            case "idle":
                              return `Idle${syncStatus.last_sync ? ` (last sync: ${new Date(syncStatus.last_sync).toLocaleTimeString()})` : ""}`;
                            case "in_progress":
                              return "Sync in progress...";
                            case "failed":
                              return `Sync failed${syncStatus.error ? `: ${syncStatus.error}` : ""}`;
                            case "rate_limited":
                              return `Rate limited until ${new Date(syncStatus.resets_at).toLocaleTimeString()}`;
                            default:
                              return "Unknown";
                          }
                        })()
                      }
                    </p>
                  </div>
                  {syncError && (
                    <div className="setting-item">
                      <div className="sync-error">
                        <p className="error-message">{syncError}</p>
                        <button onClick={clearSyncError} className="dismiss-btn">Dismiss</button>
                      </div>
                    </div>
                  )}
                </section>

                <section className="settings-section">
                  <h2>Keyboard Shortcuts</h2>
                  <div className="shortcuts-preview">
                    {AVAILABLE_SHORTCUTS.map((shortcut) => (
                      <div key={shortcut.key} className="shortcut-item">
                        <kbd className="shortcut-key">{shortcut.key}</kbd>
                        <span className="shortcut-description">{shortcut.description}</span>
                      </div>
                    ))}
                  </div>
                </section>

                <section className="settings-section">
                  <h2>About</h2>
                  <p className="about-text">Maguffin is a cross-platform Git client with a Tower-style PR dashboard and Graphite-style stacked PR workflow.</p>
                  <button className="replay-onboarding-btn" onClick={resetOnboarding}>
                    🎓 Replay onboarding tour
                  </button>
                </section>
              </div>
            )}
          </>
        )}
      </main>

      {selectedPR && repository && (
        <PRDetailPanel
          pr={selectedPR}
          repository={repository}
          onClose={() => setSelectedPR(null)}
          onActionComplete={handlePRActionComplete}
        />
      )}

      {showShortcuts && (
        <KeyboardShortcutsHelp onClose={() => setShowShortcuts(false)} />
      )}

      {restackResult && (
        <ConflictResolutionDialog
          result={restackResult}
          onClose={handleCloseRestackDialog}
          onRetry={restackResult.status !== "success" ? handleRetryRestack : undefined}
        />
      )}
    </div>
  );
}

export default App;
