import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { TemplateManager } from "./TemplateManager";

// Mock the Tauri invoke function
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("TemplateManager", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("shows loading state initially", () => {
    mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves
    render(<TemplateManager />);
    expect(screen.getByText("Loading templates...")).toBeInTheDocument();
  });

  it("shows empty state when no templates", async () => {
    mockInvoke.mockResolvedValue([]);
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("No templates yet. Create one to get started.")).toBeInTheDocument();
    });
  });

  it("shows templates when loaded", async () => {
    mockInvoke.mockResolvedValue([
      {
        id: "test-id-1",
        name: "Test Template",
        body: "Test body content",
        is_default: false,
        created_at: "2025-01-01T00:00:00Z",
        updated_at: "2025-01-01T00:00:00Z",
      },
    ]);
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("Test Template")).toBeInTheDocument();
    });
  });

  it("shows default badge for default template", async () => {
    mockInvoke.mockResolvedValue([
      {
        id: "test-id-1",
        name: "Default Template",
        body: "Default body",
        is_default: true,
        created_at: "2025-01-01T00:00:00Z",
        updated_at: "2025-01-01T00:00:00Z",
      },
    ]);
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("Default")).toBeInTheDocument();
    });
  });

  it("shows create form when clicking new template button", async () => {
    mockInvoke.mockResolvedValue([]);
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("+ New Template")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("+ New Template"));

    expect(screen.getByText("Create Template")).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toBeInTheDocument();
    expect(screen.getByLabelText("Body")).toBeInTheDocument();
  });

  it("shows edit form when clicking edit button", async () => {
    mockInvoke.mockResolvedValue([
      {
        id: "test-id-1",
        name: "Test Template",
        body: "Test body",
        is_default: false,
        created_at: "2025-01-01T00:00:00Z",
        updated_at: "2025-01-01T00:00:00Z",
      },
    ]);
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("Test Template")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Edit"));

    expect(screen.getByText("Edit Template")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Test Template")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Test body")).toBeInTheDocument();
  });

  it("calls onSelectTemplate when Use button is clicked", async () => {
    const onSelectTemplate = vi.fn();
    const template = {
      id: "test-id-1",
      name: "Test Template",
      body: "Test body",
      is_default: false,
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    };
    mockInvoke.mockResolvedValue([template]);

    render(<TemplateManager onSelectTemplate={onSelectTemplate} />);

    await waitFor(() => {
      expect(screen.getByText("Use")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Use"));

    expect(onSelectTemplate).toHaveBeenCalledWith(template);
  });

  it("cancels editing when cancel button is clicked", async () => {
    mockInvoke.mockResolvedValue([]);
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("+ New Template")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("+ New Template"));
    expect(screen.getByText("Create Template")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Cancel"));
    expect(screen.getByText("+ New Template")).toBeInTheDocument();
  });

  it("shows placeholder hint in form", async () => {
    mockInvoke.mockResolvedValue([]);
    render(<TemplateManager />);

    await waitFor(() => {
      fireEvent.click(screen.getByText("+ New Template"));
    });

    expect(screen.getByText(/Available placeholders:/)).toBeInTheDocument();
  });

  it("shows error state", async () => {
    mockInvoke.mockRejectedValue(new Error("Failed to load"));
    render(<TemplateManager />);

    await waitFor(() => {
      expect(screen.getByText("Error: Failed to load")).toBeInTheDocument();
    });
  });

  it("disables save button when name is empty", async () => {
    mockInvoke.mockResolvedValue([]);
    render(<TemplateManager />);

    await waitFor(() => {
      fireEvent.click(screen.getByText("+ New Template"));
    });

    const saveButton = screen.getByText("Save");
    expect(saveButton).toBeDisabled();
  });

  it("enables save button when name is filled", async () => {
    mockInvoke.mockResolvedValue([]);
    render(<TemplateManager />);

    await waitFor(() => {
      fireEvent.click(screen.getByText("+ New Template"));
    });

    const nameInput = screen.getByLabelText("Name");
    fireEvent.change(nameInput, { target: { value: "My Template" } });

    const saveButton = screen.getByText("Save");
    expect(saveButton).not.toBeDisabled();
  });
});
