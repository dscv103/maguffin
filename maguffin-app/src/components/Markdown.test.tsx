import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Markdown } from "./Markdown";

describe("Markdown", () => {
  it("renders empty content", () => {
    render(<Markdown content="" />);
    expect(screen.queryByText(/\S/)).toBeNull();
  });

  it("renders plain text as paragraph", () => {
    render(<Markdown content="Hello world" />);
    expect(screen.getByText("Hello world")).toBeInTheDocument();
  });

  it("renders headings correctly", () => {
    render(<Markdown content="# Heading 1" />);
    const heading = document.querySelector("h1");
    expect(heading).toBeInTheDocument();
    expect(heading?.textContent).toBe("Heading 1");
  });

  it("renders h2 headings", () => {
    render(<Markdown content="## Heading 2" />);
    const heading = document.querySelector("h2");
    expect(heading).toBeInTheDocument();
  });

  it("renders bold text", () => {
    render(<Markdown content="This is **bold** text" />);
    const strong = document.querySelector("strong");
    expect(strong).toBeInTheDocument();
    expect(strong?.textContent).toBe("bold");
  });

  it("renders italic text", () => {
    render(<Markdown content="This is *italic* text" />);
    const em = document.querySelector("em");
    expect(em).toBeInTheDocument();
    expect(em?.textContent).toBe("italic");
  });

  it("renders inline code", () => {
    render(<Markdown content="Use `const x = 1` here" />);
    const code = document.querySelector("code");
    expect(code).toBeInTheDocument();
    expect(code?.textContent).toBe("const x = 1");
  });

  it("renders code blocks", () => {
    const content = "```javascript\nconst x = 1;\n```";
    render(<Markdown content={content} />);
    const pre = document.querySelector("pre");
    const code = document.querySelector("code");
    expect(pre).toBeInTheDocument();
    expect(code).toBeInTheDocument();
    expect(code?.textContent).toContain("const x = 1");
  });

  it("renders links with safe URLs", () => {
    render(<Markdown content="[Click here](https://example.com)" />);
    const link = document.querySelector("a");
    expect(link).toBeInTheDocument();
    expect(link?.getAttribute("href")).toBe("https://example.com");
    expect(link?.textContent).toBe("Click here");
    expect(link?.getAttribute("target")).toBe("_blank");
    expect(link?.getAttribute("rel")).toBe("noopener noreferrer");
  });

  it("blocks javascript: URLs", () => {
    render(<Markdown content="[Click](javascript:alert(1))" />);
    const link = document.querySelector("a");
    expect(link).toBeNull();
  });

  it("blocks data: URLs", () => {
    render(<Markdown content="[Click](data:text/html,<script>alert(1)</script>)" />);
    const link = document.querySelector("a");
    expect(link).toBeNull();
  });

  it("renders unordered lists", () => {
    const content = `- Item 1
- Item 2`;
    render(<Markdown content={content} />);
    const ul = document.querySelector("ul");
    const items = document.querySelectorAll("li");
    expect(ul).toBeInTheDocument();
    expect(items.length).toBe(2);
  });

  it("renders ordered lists", () => {
    const content = `1. First
2. Second`;
    render(<Markdown content={content} />);
    const ol = document.querySelector("ol");
    const items = document.querySelectorAll("li");
    expect(ol).toBeInTheDocument();
    expect(items.length).toBe(2);
  });

  it("renders task lists", () => {
    const content = `- [ ] Unchecked
- [x] Checked`;
    render(<Markdown content={content} />);
    const checkboxes = document.querySelectorAll('input[type="checkbox"]');
    expect(checkboxes.length).toBe(2);
    expect(checkboxes[0]).not.toBeChecked();
    expect(checkboxes[1]).toBeChecked();
  });

  it("renders blockquotes", () => {
    render(<Markdown content="> This is a quote" />);
    const blockquote = document.querySelector("blockquote");
    expect(blockquote).toBeInTheDocument();
    expect(blockquote?.textContent).toContain("This is a quote");
  });

  it("renders horizontal rules", () => {
    const content = `Above

---

Below`;
    render(<Markdown content={content} />);
    const hr = document.querySelector("hr");
    expect(hr).toBeInTheDocument();
  });

  it("renders strikethrough text", () => {
    render(<Markdown content="This is ~~deleted~~ text" />);
    const del = document.querySelector("del");
    expect(del).toBeInTheDocument();
    expect(del?.textContent).toBe("deleted");
  });

  it("escapes HTML entities", () => {
    render(<Markdown content="<script>alert('xss')</script>" />);
    const script = document.querySelector("script");
    expect(script).toBeNull();
    expect(screen.getByText(/script/)).toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(
      <Markdown content="Test" className="custom-class" />
    );
    expect(container.firstChild).toHaveClass("custom-class");
    expect(container.firstChild).toHaveClass("markdown-content");
  });
});
