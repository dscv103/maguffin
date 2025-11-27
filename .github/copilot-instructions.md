# GitHub Copilot Instructions

## Overview
These instructions guide GitHub Copilot agents and related orchestration tools for SDLC workflows in this repository. They define agent roles, workflow processes, and code review standards to maintain high code quality while minimizing false positives.

## Agent Roles

- **Coordinator Agent:** Orchestrates SDLC workflow tasks and delegates subtasks to specialized agents
- **Code Generation Agent:** Generates source code, unit tests, and documentation for features and bug fixes
- **Review Agent:** Performs automated code review for style, correctness, and security issues
- **Integration Agent:** Manages merge queues, PR status, and CI/CD validation steps
- **Release Agent:** Drafts releases, modifies release notes, and coordinates tagging and publishing

## Workflow Instructions

### Branching
- Create a branch for each major feature, bugfix, or refactor
- Use naming conventions: `feature/<feature-name>`, `bugfix/<issue-id>`, `refactor/<module-name>`

### Code Generation
- Incorporate code using approved templates and language-specific best practices
- Generate docstrings and comments for complex logic
- Ensure unit-test coverage for new and updated modules
- Follow existing patterns in the codebase

### Automated Review
- Review all incoming PRs for style adherence, security warnings, and documentation completeness
- Request changes if issues are found with HIGH CONFIDENCE (>80%)
- Focus on actionable feedback, not observations

### Integration & CI/CD
- Trigger CI pipeline on new commits and PRs
- Ensure build passes and code is linted
- Coordinate with merge queues for multiple PRs
- Do not flag issues that CI will catch automatically

### Release Preparation
- Aggregate changelog items from merged PRs
- Prepare release notes in the designated format
- Draft release and tag with semver (`vX.Y.Z`)

## Code Review Philosophy

### Core Principles
- Only comment when you have HIGH CONFIDENCE (>80%) that an issue exists
- Be concise: one sentence per comment when possible
- Focus on actionable feedback, not observations
- When reviewing text, only comment on clarity issues if the text is genuinely confusing or could lead to errors
- "Could be clearer" is not the same as "is confusing" - stay silent unless HIGH confidence it will cause problems

### Priority Areas (Review These)

#### Security & Safety
- Unsafe code blocks without justification
- Command injection risks (shell commands, user input)
- Path traversal vulnerabilities
- Credential exposure or hardcoded secrets
- Missing input validation on external data
- Improper error handling that could leak sensitive info

#### Correctness Issues
- Logic errors that could cause panics or incorrect behavior
- Race conditions in async code
- Resource leaks (files, connections, memory)
- Off-by-one errors or boundary conditions
- Incorrect error propagation (using `unwrap()` inappropriately)
- Optional types that don't need to be optional
- Booleans that should default to false but are set as optional
- Error context that doesn't add useful information (e.g., `.context("Failed to do X")` when error already says it failed)
- Overly defensive code that adds unnecessary checks
- Unnecessary comments that just restate what the code already shows (remove them)

#### Architecture & Patterns
- Code that violates existing patterns in the codebase
- Missing error handling (use project's error handling conventions)
- Async/await misuse or blocking operations in async contexts
- Improper trait implementations or interface misuse

## CI Pipeline Context

**Important**: You review PRs immediately, before CI completes. Do not flag issues that CI will catch.

### Common CI Checks to Avoid Duplicating

Be aware of what your CI pipeline checks and avoid commenting on issues it will catch:

**Typical checks:**
- Code formatting (e.g., rustfmt, prettier, black)
- Linting (e.g., clippy, eslint, pylint)
- Test execution (unit tests, integration tests)
- Dependency installation and validation
- Schema validation
- Build verification

**Setup steps CI typically performs:**
- Installs system dependencies
- Activates project-specific environments
- Caches dependencies
- Runs fresh dependency installs

**Key insight**: Commands that rely on dependencies in CI will work if CI installs them properly. Don't flag these as broken unless you can explain why CI setup wouldn't handle it.

## Skip These (Low Value)

Do not comment on:
- **Style/formatting** - CI handles this
- **Linting warnings** - CI handles this
- **Test failures** - CI handles this
- **Missing dependencies** - CI handles this
- **Minor naming suggestions** - unless truly confusing
- **Suggestions to add comments** - for self-documenting code
- **Refactoring suggestions** - unless there's a clear bug or maintainability issue
- **Multiple issues in one comment** - choose the single most critical issue
- **Logging suggestions** - unless for errors or security events (codebases need less logging, not more)
- **Pedantic accuracy in text** - unless it would cause actual confusion or errors

## Response Format

When you identify an issue:
1. **State the problem** (1 sentence)
2. **Why it matters** (1 sentence, only if not obvious)
3. **Suggested fix** (code snippet or specific action)

Example:
```
This could panic if the vector is empty. Consider using `.get(0)` or add a length check.
```

## When to Stay Silent

If you're uncertain whether something is an issue, don't comment. False positives create noise and reduce trust in the review process.

## Best Practices

- Automate repetitive tasks wherever possible
- Favor small, frequent PRs with clear descriptions
- Ensure all code is reviewed by the Review Agent before merging
- Maintain up-to-date documentation with every release
- Respect existing code patterns and conventions in the repository
- Refer to project-specific documentation (e.g., HOWTOAI.md, README) for additional standards

## References

See the repository's README and contributing guidelines for more details on architecture, coding standards, and agent setup.
