import { useState } from "react";
import { AuthView, PRDashboard, PRDetailPanel, StackList, RepoSelector } from "./components";
import { useAuth, useStacks, useRepository, usePullRequests } from "./hooks";
import type { PullRequest, Stack } from "./types";

type View = "auth" | "dashboard" | "stacks" | "settings";

function App() {
  const { authState } = useAuth();
  const { repository, loading: repoLoading, error: repoError, openRepository, clearRepository, clearError: clearRepoError } = useRepository();
  const { stacks, loading: stacksLoading, error: stacksError, restackStack } = useStacks(repository);
  const { refresh: refreshPRs } = usePullRequests();
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
            loading={repoLoading}
            error={repoError}
            onOpenRepository={openRepository}
            onClearRepository={clearRepository}
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
                <p className="coming-soon">Settings coming soon...</p>
              </div>
            )}
          </>
        )}
      </main>

      {selectedPR && (
        <PRDetailPanel
          pr={selectedPR}
          prId={selectedPR.id}
          onClose={() => setSelectedPR(null)}
          onActionComplete={handlePRActionComplete}
        />
      )}
    </div>
  );
}

export default App;
