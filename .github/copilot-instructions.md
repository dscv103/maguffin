# Copilot Agent Instructions

## Overview
These instructions are designed to guide GitHub Copilot agents and related orchestration tools for SDLC workflows in this repository.

## Agent Roles
- **Coordinator Agent:** Orchestrates SDLC workflow tasks and delegates subtasks to specialized agents.
- **Code Generation Agent:** Generates source code, unit tests, and documentation for features and bug fixes.
- **Review Agent:** Performs automated code review for style, correctness, and security issues.
- **Integration Agent:** Manages merge queues, PR status, and CI/CD validation steps.
- **Release Agent:** Drafts releases, modifies release notes, and coordinates tagging and publishing.

## Workflow Instructions
1. **Branching:**
   - Create a branch for each major feature, bugfix, or refactor.
   - Use naming conventions: `feature/<feature-name>`, `bugfix/<issue-id>`, `refactor/<module-name>`.
2. **Code Generation:**
   - Incorporate code using approved templates and language-specific best practices.
   - Generate docstrings and comments.
   - Ensure unit-test coverage for new and updated modules.
3. **Automated Review:**
   - Review all incoming PRs for style adherence, security warnings, and documentation completeness.
   - Request changes if issues are found.
4. **Integration & CI/CD:**
   - Trigger CI pipeline on new commits and PRs.
   - Ensure build passes and code is linted.
   - Coordinate with merge queues for multiple PRs.
5. **Release Preparation:**
   - Aggregate changelog items from merged PRs.
   - Prepare release notes in the designated format.
   - Draft release and tag with semver (`vX.Y.Z`).

## Best Practices
- Automate repetitive tasks wherever possible.
- Favor small, frequent PRs with clear descriptions.
- Ensure all code is reviewed by the Review Agent before merging.
- Maintain up-to-date documentation with every release.

## References
- See the repository's README for more details on architecture and agent setup.
