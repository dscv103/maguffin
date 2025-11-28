import { useState } from "react";
import { AuthView, PRDashboard, StackList } from "./components";
import { useAuth, useStacks } from "./hooks";
import type { PullRequest, Stack } from "./types";

type View = "auth" | "dashboard" | "stacks" | "settings";

function App() {
  const { authState } = useAuth();
  const { stacks, loading: stacksLoading, error: stacksError, restackStack } = useStacks();
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

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="sidebar-header">
          <h1 className="app-logo">Maguffin</h1>
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
      </main>

      {selectedPR && (
        <aside className="pr-detail-panel">
          <header>
            <h2>#{selectedPR.number} {selectedPR.title}</h2>
            <button onClick={() => setSelectedPR(null)}>×</button>
          </header>
          <div className="pr-detail-content">
            <p>{selectedPR.body || "No description provided."}</p>
            <div className="pr-detail-meta">
              <p>
                <strong>Author:</strong> {selectedPR.author.login}
              </p>
              <p>
                <strong>Branch:</strong> {selectedPR.head_ref} → {selectedPR.base_ref}
              </p>
              <p>
                <strong>Changes:</strong> +{selectedPR.additions} -{selectedPR.deletions}
              </p>
            </div>
          </div>
        </aside>
      )}
    </div>
  );
}

export default App;
