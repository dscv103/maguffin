import React from "react";

interface MarkdownProps {
  content: string;
  className?: string;
}

/**
 * Simple Markdown renderer for PR descriptions.
 * Supports: headings, bold, italic, code, links, lists, blockquotes, and horizontal rules.
 */
export function Markdown({ content, className }: MarkdownProps) {
  const html = React.useMemo(() => parseMarkdown(content), [content]);

  return (
    <div
      className={`markdown-content ${className || ""}`}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function parseMarkdown(text: string): string {
  if (!text) return "";

  let html = escapeHtml(text);

  // Process code blocks first (```...```)
  html = html.replace(/```(\w*)\n([\s\S]*?)```/g, (_match, _lang, code) => {
    return `<pre><code>${code.trim()}</code></pre>`;
  });

  // Inline code (`...`)
  html = html.replace(/`([^`]+)`/g, "<code>$1</code>");

  // Headings (# through ######)
  html = html.replace(/^###### (.+)$/gm, "<h6>$1</h6>");
  html = html.replace(/^##### (.+)$/gm, "<h5>$1</h5>");
  html = html.replace(/^#### (.+)$/gm, "<h4>$1</h4>");
  html = html.replace(/^### (.+)$/gm, "<h3>$1</h3>");
  html = html.replace(/^## (.+)$/gm, "<h2>$1</h2>");
  html = html.replace(/^# (.+)$/gm, "<h1>$1</h1>");

  // Bold (**text** or __text__)
  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/__(.+?)__/g, "<strong>$1</strong>");

  // Italic (*text* or _text_)
  html = html.replace(/\*(.+?)\*/g, "<em>$1</em>");
  html = html.replace(/(?<![a-zA-Z0-9])_([^_]+)_(?![a-zA-Z0-9])/g, "<em>$1</em>");

  // Strikethrough (~~text~~)
  html = html.replace(/~~(.+?)~~/g, "<del>$1</del>");

  // Blockquotes (> text)
  html = html.replace(/^&gt; (.+)$/gm, "<blockquote>$1</blockquote>");

  // Horizontal rules (--- or ***)
  html = html.replace(/^(---|\*\*\*)$/gm, "<hr />");

  // Links [text](url)
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>');

  // Images ![alt](url)
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" />');

  // Process lists with proper handling for ordered vs unordered
  html = processLists(html);

  // Paragraphs - wrap lines that don't start with HTML tags
  const lines = html.split("\n");
  const processedLines = lines.map((line) => {
    const trimmed = line.trim();
    if (!trimmed) return "";
    if (
      trimmed.startsWith("<h") ||
      trimmed.startsWith("<ul") ||
      trimmed.startsWith("<ol") ||
      trimmed.startsWith("<li") ||
      trimmed.startsWith("<pre") ||
      trimmed.startsWith("<blockquote") ||
      trimmed.startsWith("<hr") ||
      trimmed.startsWith("</")
    ) {
      return line;
    }
    return `<p>${line}</p>`;
  });

  html = processedLines.join("\n");

  // Clean up empty paragraphs
  html = html.replace(/<p><\/p>/g, "");
  html = html.replace(/<p>\s*<\/p>/g, "");

  return html;
}

/**
 * Process list items with proper handling for:
 * - Task lists (- [ ] or - [x])
 * - Unordered lists (- or *)
 * - Ordered lists (1. 2. etc.)
 */
function processLists(html: string): string {
  const lines = html.split("\n");
  const result: string[] = [];
  let inList = false;
  let listType: "ul" | "ol" | null = null;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    
    // Task list (- [ ] or - [x])
    const taskUnchecked = line.match(/^[\-\*] \[ \] (.+)$/);
    const taskChecked = line.match(/^[\-\*] \[x\] (.+)$/i);
    // Unordered list (- or *)
    const unordered = line.match(/^[\-\*] (.+)$/);
    // Ordered list (1. 2. etc.)
    const ordered = line.match(/^(\d+)\. (.+)$/);

    if (taskUnchecked) {
      if (!inList || listType !== "ul") {
        if (inList) result.push(listType === "ol" ? "</ol>" : "</ul>");
        result.push("<ul>");
        inList = true;
        listType = "ul";
      }
      result.push(`<li class="task-item"><input type="checkbox" disabled /> ${taskUnchecked[1]}</li>`);
    } else if (taskChecked) {
      if (!inList || listType !== "ul") {
        if (inList) result.push(listType === "ol" ? "</ol>" : "</ul>");
        result.push("<ul>");
        inList = true;
        listType = "ul";
      }
      result.push(`<li class="task-item"><input type="checkbox" disabled checked /> ${taskChecked[1]}</li>`);
    } else if (ordered) {
      if (!inList || listType !== "ol") {
        if (inList) result.push(listType === "ol" ? "</ol>" : "</ul>");
        result.push("<ol>");
        inList = true;
        listType = "ol";
      }
      result.push(`<li>${ordered[2]}</li>`);
    } else if (unordered) {
      if (!inList || listType !== "ul") {
        if (inList) result.push(listType === "ol" ? "</ol>" : "</ul>");
        result.push("<ul>");
        inList = true;
        listType = "ul";
      }
      result.push(`<li>${unordered[1]}</li>`);
    } else {
      // Close any open list
      if (inList) {
        result.push(listType === "ol" ? "</ol>" : "</ul>");
        inList = false;
        listType = null;
      }
      result.push(line);
    }
  }

  // Close any remaining open list
  if (inList) {
    result.push(listType === "ol" ? "</ol>" : "</ul>");
  }

  return result.join("\n");
}

function escapeHtml(text: string): string {
  const escapeMap: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
  };
  return text.replace(/[&<>]/g, (char) => escapeMap[char] || char);
}
