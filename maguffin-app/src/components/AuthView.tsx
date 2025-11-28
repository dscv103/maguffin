import { useEffect } from "react";
import { useAuth } from "../hooks";

interface AuthViewProps {
  onAuthenticated?: () => void;
}

export function AuthView({ onAuthenticated }: AuthViewProps) {
  const { authState, loading, error, startDeviceFlow, pollDeviceFlow, logout } = useAuth();

  useEffect(() => {
    if (authState.type === "authenticated" && onAuthenticated) {
      onAuthenticated();
    }
  }, [authState, onAuthenticated]);

  // Poll for auth completion when in pending state
  useEffect(() => {
    if (authState.type !== "pending") return;

    const interval = setInterval(async () => {
      const newState = await pollDeviceFlow();
      if (newState.type === "authenticated") {
        clearInterval(interval);
      }
    }, 5000);

    return () => clearInterval(interval);
  }, [authState.type, pollDeviceFlow]);

  if (loading) {
    return (
      <div className="auth-view loading">
        <div className="spinner" />
        <p>Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="auth-view error">
        <p className="error-message">Error: {error}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </div>
    );
  }

  if (authState.type === "authenticated") {
    return (
      <div className="auth-view authenticated">
        <div className="user-info">
          <img
            src={authState.data.avatar_url}
            alt={authState.data.username}
            className="avatar"
          />
          <span className="username">{authState.data.username}</span>
        </div>
        <button onClick={logout} className="logout-btn">
          Logout
        </button>
      </div>
    );
  }

  if (authState.type === "pending") {
    return (
      <div className="auth-view pending">
        <h2>GitHub Authentication</h2>
        <p>Enter this code at GitHub:</p>
        <code className="user-code">{authState.data.user_code}</code>
        <a
          href={authState.data.verification_uri}
          target="_blank"
          rel="noopener noreferrer"
          className="verify-link"
        >
          Open GitHub
        </a>
        <p className="waiting">Waiting for authorization...</p>
      </div>
    );
  }

  // Unauthenticated state
  return (
    <div className="auth-view unauthenticated">
      <h2>Welcome to Maguffin</h2>
      <p>Sign in with GitHub to manage your pull requests</p>
      <button onClick={startDeviceFlow} className="login-btn">
        Sign in with GitHub
      </button>
    </div>
  );
}
