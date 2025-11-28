import { useState } from "react";
import { AuthView, PRDashboard, PRDetailPanel, StackList, RepoSelector, ThemeToggle } from "./components";
import { useAuth, useStacks, useRepository, usePullRequests, useTheme } from "./hooks";
import type { PullRequest, Stack } from "./types";

type View = "auth" | "dashboard" | "stacks" | "settings";

function App() {
  const { authState } = useAuth();
  const { repository, recentRepositories, loading: repoLoading, error: repoError, openRepository, removeRecentRepository, clearRepository, clearError: clearRepoError } = useRepository();
  const { stacks, loading: stacksLoading, error: stacksError, restackStack } = useStacks(repository);
  const { refresh: refreshPRs } = usePullRequests();
  const { theme, setTheme } = useTheme();
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [selectedPR, setSelectedPR] = useState<PullRequest | null>(null);

  const isAuthenticated = authState.type === "authenticated";

  if (!isAuthenticated) {
    return (
      <div className="app">
        <AuthView onAuthenticated={() => setCurrentView("dashboard")} />
      </div>
    );
  }

  const handleRestack = async (stack: Stack) => {
    await restackStack(stack.id);
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
              <PRDashboard onSelectPR={(pr) => setSelectedPR(pr)} />
            )}

            {currentView === "stacks" && (
              <div className="stacks-view">
                <h1>Stacks</h1>
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
                  <h2>About</h2>
                  <p className="about-text">Maguffin is a cross-platform Git client with a Tower-style PR dashboard and Graphite-style stacked PR workflow.</p>
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
    </div>
  );
}

export default App;
