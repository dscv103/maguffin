import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AuthView } from "./AuthView";
import type { AuthState } from "../types";

// Mock useAuth hook
const mockStartDeviceFlow = vi.fn();
const mockPollDeviceFlow = vi.fn();
const mockLogout = vi.fn();
const mockRefresh = vi.fn();

vi.mock("../hooks", () => ({
  useAuth: vi.fn(() => ({
    authState: { type: "unauthenticated" } as AuthState,
    loading: false,
    error: null,
    startDeviceFlow: mockStartDeviceFlow,
    pollDeviceFlow: mockPollDeviceFlow,
    logout: mockLogout,
    refresh: mockRefresh,
  })),
}));

import { useAuth } from "../hooks";

describe("AuthView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("loading state", () => {
    it("renders loading spinner when loading", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: true,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Loading...")).toBeInTheDocument();
    });

    it("applies loading class", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: true,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      const { container } = render(<AuthView />);
      expect(container.querySelector(".loading")).toBeInTheDocument();
    });
  });

  describe("error state", () => {
    it("renders error message", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: "Authentication failed",
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Error: Authentication failed")).toBeInTheDocument();
    });

    it("renders retry button on error", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: "Some error",
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Retry")).toBeInTheDocument();
    });
  });

  describe("unauthenticated state", () => {
    it("renders welcome message", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Welcome to Maguffin")).toBeInTheDocument();
    });

    it("renders sign in button", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Sign in with GitHub")).toBeInTheDocument();
    });

    it("calls startDeviceFlow when sign in button is clicked", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      fireEvent.click(screen.getByText("Sign in with GitHub"));
      expect(mockStartDeviceFlow).toHaveBeenCalledOnce();
    });
  });

  describe("pending state", () => {
    it("renders user code", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "pending",
          user_code: "ABCD-1234",
          verification_uri: "https://github.com/login/device",
          expires_at: new Date(Date.now() + 600000).toISOString(),
          interval: 5,
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("ABCD-1234")).toBeInTheDocument();
    });

    it("renders GitHub link", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "pending",
          user_code: "ABCD-1234",
          verification_uri: "https://github.com/login/device",
          expires_at: new Date(Date.now() + 600000).toISOString(),
          interval: 5,
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      const link = screen.getByText("Open GitHub");
      expect(link).toHaveAttribute("href", "https://github.com/login/device");
      expect(link).toHaveAttribute("target", "_blank");
    });

    it("shows waiting message", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "pending",
          user_code: "ABCD-1234",
          verification_uri: "https://github.com/login/device",
          expires_at: new Date(Date.now() + 600000).toISOString(),
          interval: 5,
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Waiting for authorization...")).toBeInTheDocument();
    });
  });

  describe("authenticated state", () => {
    it("renders user avatar", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "authenticated",
          login: "testuser",
          id: 123,
          name: "Test User",
          email: "test@example.com",
          avatar_url: "https://example.com/avatar.jpg",
          authenticated_at: new Date().toISOString(),
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      const avatar = screen.getByRole("img");
      expect(avatar).toHaveAttribute("src", "https://example.com/avatar.jpg");
      expect(avatar).toHaveAttribute("alt", "testuser");
    });

    it("renders username", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "authenticated",
          login: "testuser",
          id: 123,
          name: "Test User",
          email: "test@example.com",
          avatar_url: "https://example.com/avatar.jpg",
          authenticated_at: new Date().toISOString(),
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("testuser")).toBeInTheDocument();
    });

    it("renders logout button", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "authenticated",
          login: "testuser",
          id: 123,
          name: "Test User",
          email: "test@example.com",
          avatar_url: "https://example.com/avatar.jpg",
          authenticated_at: new Date().toISOString(),
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      expect(screen.getByText("Logout")).toBeInTheDocument();
    });

    it("calls logout when logout button is clicked", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "authenticated",
          login: "testuser",
          id: 123,
          name: "Test User",
          email: "test@example.com",
          avatar_url: "https://example.com/avatar.jpg",
          authenticated_at: new Date().toISOString(),
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView />);
      fireEvent.click(screen.getByText("Logout"));
      expect(mockLogout).toHaveBeenCalledOnce();
    });
  });

  describe("onAuthenticated callback", () => {
    it("calls onAuthenticated when user becomes authenticated", () => {
      const onAuthenticated = vi.fn();
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "authenticated",
          login: "testuser",
          id: 123,
          name: "Test User",
          email: "test@example.com",
          avatar_url: "https://example.com/avatar.jpg",
          authenticated_at: new Date().toISOString(),
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView onAuthenticated={onAuthenticated} />);
      expect(onAuthenticated).toHaveBeenCalledOnce();
    });

    it("does not call onAuthenticated when unauthenticated", () => {
      const onAuthenticated = vi.fn();
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      render(<AuthView onAuthenticated={onAuthenticated} />);
      expect(onAuthenticated).not.toHaveBeenCalled();
    });
  });

  describe("CSS classes", () => {
    it("applies authenticated class when authenticated", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "authenticated",
          login: "testuser",
          id: 123,
          name: "Test User",
          email: "test@example.com",
          avatar_url: "https://example.com/avatar.jpg",
          authenticated_at: new Date().toISOString(),
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      const { container } = render(<AuthView />);
      expect(container.querySelector(".authenticated")).toBeInTheDocument();
    });

    it("applies unauthenticated class when unauthenticated", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: { type: "unauthenticated" },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      const { container } = render(<AuthView />);
      expect(container.querySelector(".unauthenticated")).toBeInTheDocument();
    });

    it("applies pending class when pending", () => {
      vi.mocked(useAuth).mockReturnValue({
        authState: {
          type: "pending",
          user_code: "ABCD-1234",
          verification_uri: "https://github.com/login/device",
          expires_at: new Date(Date.now() + 600000).toISOString(),
          interval: 5,
        },
        loading: false,
        error: null,
        startDeviceFlow: mockStartDeviceFlow,
        pollDeviceFlow: mockPollDeviceFlow,
        logout: mockLogout,
        refresh: mockRefresh,
      });
      const { container } = render(<AuthView />);
      expect(container.querySelector(".pending")).toBeInTheDocument();
    });
  });
});
