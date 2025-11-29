import { describe, it, expect, vi, beforeAll, afterAll } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ErrorBoundary, ViewErrorFallback } from "./ErrorBoundary";
import type { ReactNode } from "react";

// Component that throws an error
function ThrowingComponent({ shouldThrow = true }: { shouldThrow?: boolean }): ReactNode {
  if (shouldThrow) {
    throw new Error("Test error message");
  }
  return <div>Normal render</div>;
}

// Suppress console.error for cleaner test output
const originalError = console.error;
beforeAll(() => {
  console.error = vi.fn();
});
afterAll(() => {
  console.error = originalError;
});

describe("ErrorBoundary", () => {
  it("renders children when no error occurs", () => {
    render(
      <ErrorBoundary>
        <div>Child content</div>
      </ErrorBoundary>
    );

    expect(screen.getByText("Child content")).toBeInTheDocument();
  });

  it("renders default fallback when error occurs", () => {
    render(
      <ErrorBoundary>
        <ThrowingComponent />
      </ErrorBoundary>
    );

    expect(screen.getByText("Something went wrong")).toBeInTheDocument();
    expect(screen.getByText("Test error message")).toBeInTheDocument();
    expect(screen.getByText("Try Again")).toBeInTheDocument();
  });

  it("renders custom fallback when provided", () => {
    render(
      <ErrorBoundary fallback={<div>Custom error UI</div>}>
        <ThrowingComponent />
      </ErrorBoundary>
    );

    expect(screen.getByText("Custom error UI")).toBeInTheDocument();
    expect(screen.queryByText("Something went wrong")).not.toBeInTheDocument();
  });

  it("calls onError callback when error occurs", () => {
    const onError = vi.fn();
    render(
      <ErrorBoundary onError={onError}>
        <ThrowingComponent />
      </ErrorBoundary>
    );

    expect(onError).toHaveBeenCalled();
    expect(onError.mock.calls[0][0]).toBeInstanceOf(Error);
    expect(onError.mock.calls[0][0].message).toBe("Test error message");
  });

  it("resets error state when Try Again is clicked", () => {
    // We need to mock the component to allow toggling
    let shouldThrow = true;
    function ToggleableThrowingComponent() {
      if (shouldThrow) {
        throw new Error("Error!");
      }
      return <div>Recovered!</div>;
    }

    const { rerender } = render(
      <ErrorBoundary>
        <ToggleableThrowingComponent />
      </ErrorBoundary>
    );

    // Error state should be shown
    expect(screen.getByText("Something went wrong")).toBeInTheDocument();

    // Stop throwing
    shouldThrow = false;

    // Click Try Again
    fireEvent.click(screen.getByText("Try Again"));

    // Rerender to trigger the component again
    rerender(
      <ErrorBoundary>
        <ToggleableThrowingComponent />
      </ErrorBoundary>
    );

    // Should show recovered content
    expect(screen.getByText("Recovered!")).toBeInTheDocument();
  });

  it("shows generic message when error has no message", () => {
    function ThrowNoMessage(): ReactNode {
      throw new Error();
    }

    render(
      <ErrorBoundary>
        <ThrowNoMessage />
      </ErrorBoundary>
    );

    expect(screen.getByText("An unexpected error occurred")).toBeInTheDocument();
  });
});

describe("ViewErrorFallback", () => {
  it("renders default message", () => {
    render(<ViewErrorFallback />);

    expect(screen.getByText("This view couldn't be loaded")).toBeInTheDocument();
    expect(screen.getByText("Oops!")).toBeInTheDocument();
    expect(screen.getByText("⚠️")).toBeInTheDocument();
  });

  it("renders custom message", () => {
    render(<ViewErrorFallback message="Failed to load PRs" />);

    expect(screen.getByText("Failed to load PRs")).toBeInTheDocument();
  });

  it("shows retry button when onRetry is provided", () => {
    const onRetry = vi.fn();
    render(<ViewErrorFallback onRetry={onRetry} />);

    const retryButton = screen.getByText("Retry");
    expect(retryButton).toBeInTheDocument();

    fireEvent.click(retryButton);
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("hides retry button when onRetry is not provided", () => {
    render(<ViewErrorFallback />);

    expect(screen.queryByText("Retry")).not.toBeInTheDocument();
  });
});
