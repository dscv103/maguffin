import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ThemeToggle } from "./ThemeToggle";

// Mock useTheme hook
const mockToggleTheme = vi.fn();
vi.mock("../hooks/useTheme", () => ({
  useTheme: vi.fn(() => ({
    theme: "dark",
    toggleTheme: mockToggleTheme,
  })),
}));

import { useTheme } from "../hooks/useTheme";

describe("ThemeToggle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useTheme).mockReturnValue({
      theme: "dark",
      toggleTheme: mockToggleTheme,
      setTheme: vi.fn(),
      isDark: true,
      isLight: false,
    });
  });

  it("renders a button", () => {
    render(<ThemeToggle />);
    const button = screen.getByRole("button");
    expect(button).toBeInTheDocument();
  });

  it("shows sun emoji when theme is dark", () => {
    vi.mocked(useTheme).mockReturnValue({
      theme: "dark",
      toggleTheme: mockToggleTheme,
      setTheme: vi.fn(),
      isDark: true,
      isLight: false,
    });
    render(<ThemeToggle />);
    expect(screen.getByRole("button")).toHaveTextContent("☀️");
  });

  it("shows moon emoji when theme is light", () => {
    vi.mocked(useTheme).mockReturnValue({
      theme: "light",
      toggleTheme: mockToggleTheme,
      setTheme: vi.fn(),
      isDark: false,
      isLight: true,
    });
    render(<ThemeToggle />);
    expect(screen.getByRole("button")).toHaveTextContent("🌙");
  });

  it("has correct aria-label for dark theme", () => {
    vi.mocked(useTheme).mockReturnValue({
      theme: "dark",
      toggleTheme: mockToggleTheme,
      setTheme: vi.fn(),
      isDark: true,
      isLight: false,
    });
    render(<ThemeToggle />);
    expect(screen.getByRole("button")).toHaveAttribute(
      "aria-label",
      "Switch to light mode"
    );
  });

  it("has correct aria-label for light theme", () => {
    vi.mocked(useTheme).mockReturnValue({
      theme: "light",
      toggleTheme: mockToggleTheme,
      setTheme: vi.fn(),
      isDark: false,
      isLight: true,
    });
    render(<ThemeToggle />);
    expect(screen.getByRole("button")).toHaveAttribute(
      "aria-label",
      "Switch to dark mode"
    );
  });

  it("has correct title for dark theme", () => {
    vi.mocked(useTheme).mockReturnValue({
      theme: "dark",
      toggleTheme: mockToggleTheme,
      setTheme: vi.fn(),
      isDark: true,
      isLight: false,
    });
    render(<ThemeToggle />);
    expect(screen.getByRole("button")).toHaveAttribute(
      "title",
      "Switch to light mode"
    );
  });

  it("calls toggleTheme when clicked", () => {
    render(<ThemeToggle />);
    fireEvent.click(screen.getByRole("button"));
    expect(mockToggleTheme).toHaveBeenCalledOnce();
  });

  it("has the correct CSS class", () => {
    render(<ThemeToggle />);
    expect(screen.getByRole("button")).toHaveClass("theme-toggle-btn");
  });
});
