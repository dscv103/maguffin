//! PR Description Template types.
//!
//! This module provides types for managing pull request description templates.
//! Templates support placeholder variables that are replaced with actual values
//! when creating a PR.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A pull request description template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrTemplate {
    /// Unique identifier for the template
    pub id: Uuid,

    /// User-friendly name for the template
    pub name: String,

    /// Template content with optional placeholders
    pub body: String,

    /// Whether this is the default template
    pub is_default: bool,

    /// When the template was created
    pub created_at: DateTime<Utc>,

    /// When the template was last updated
    pub updated_at: DateTime<Utc>,
}

impl PrTemplate {
    /// Create a new template with the given name and body.
    pub fn new(name: String, body: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            body,
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark this template as the default.
    pub fn set_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    /// Create a default template with common sections.
    pub fn default_template() -> Self {
        Self::new(
            "Default".to_string(),
            r#"## Summary

<!-- Describe your changes here -->

## Changes

<!-- List the key changes made -->

## Testing

<!-- How was this tested? -->

## Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Ready for review
"#
            .to_string(),
        )
        .set_default(true)
    }

    /// Render the template by replacing placeholders with values.
    ///
    /// Supported placeholders:
    /// - `{{branch}}` - Current branch name
    /// - `{{author}}` - Current user's login
    /// - `{{date}}` - Current date (YYYY-MM-DD)
    /// - `{{title}}` - Pull request title
    ///
    /// # Arguments
    ///
    /// * `context` - The template context with values to substitute
    ///
    /// # Returns
    ///
    /// The rendered template with placeholders replaced.
    pub fn render(&self, context: &TemplateContext) -> String {
        let mut result = self.body.clone();

        if let Some(branch) = &context.branch {
            result = result.replace("{{branch}}", branch);
        }

        if let Some(author) = &context.author {
            result = result.replace("{{author}}", author);
        }

        if let Some(date) = &context.date {
            result = result.replace("{{date}}", date);
        }

        if let Some(title) = &context.title {
            result = result.replace("{{title}}", title);
        }

        result
    }
}

/// Context for rendering a template.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Current branch name
    pub branch: Option<String>,

    /// Current user's login
    pub author: Option<String>,

    /// Current date (formatted as YYYY-MM-DD)
    pub date: Option<String>,

    /// PR title (if known)
    pub title: Option<String>,
}

impl TemplateContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the branch name.
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Set the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set the date.
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_new() {
        let template = PrTemplate::new("Test".to_string(), "Body content".to_string());
        assert_eq!(template.name, "Test");
        assert_eq!(template.body, "Body content");
        assert!(!template.is_default);
    }

    #[test]
    fn test_template_set_default() {
        let template = PrTemplate::new("Test".to_string(), "Body".to_string()).set_default(true);
        assert!(template.is_default);
    }

    #[test]
    fn test_default_template() {
        let template = PrTemplate::default_template();
        assert_eq!(template.name, "Default");
        assert!(template.is_default);
        assert!(template.body.contains("## Summary"));
    }

    #[test]
    fn test_template_render_with_placeholders() {
        let template = PrTemplate::new(
            "Test".to_string(),
            "Branch: {{branch}}\nAuthor: {{author}}\nDate: {{date}}".to_string(),
        );

        let context = TemplateContext::new()
            .with_branch("feature/test")
            .with_author("user123")
            .with_date("2025-01-15");

        let rendered = template.render(&context);
        assert_eq!(
            rendered,
            "Branch: feature/test\nAuthor: user123\nDate: 2025-01-15"
        );
    }

    #[test]
    fn test_template_render_partial_context() {
        let template = PrTemplate::new(
            "Test".to_string(),
            "Branch: {{branch}}\nAuthor: {{author}}".to_string(),
        );

        let context = TemplateContext::new().with_branch("main");

        let rendered = template.render(&context);
        assert_eq!(rendered, "Branch: main\nAuthor: {{author}}");
    }

    #[test]
    fn test_template_render_empty_context() {
        let template = PrTemplate::new("Test".to_string(), "No placeholders here".to_string());

        let context = TemplateContext::new();

        let rendered = template.render(&context);
        assert_eq!(rendered, "No placeholders here");
    }
}
