# Project State and Handoff Ledger

This file persists SDLC phase, ownership, and handoff history so agents can coordinate over time.

## Current project state

- `current_phase`: design
- `active_agents`: ["sdlc-manager", "code-architect"]
- `blocked_tasks`: []

Agents must update this section when they change phase, become active, or encounter blockers.

## Handoffs

A chronological list of JSON objects describing ownership transfers. Append new entries; never edit history.

```json
[
  {
    "from_agent": "sdlc-manager",
    "to_agent": "requirements-analyst",
    "phase": "planningtorequirements",
    "artefacts": [],
    "trigger": "Initial project kickoff for Rust Git client with Tower-style PR dashboard and Graphite-style stacked PRs",
    "timestamp": "2025-11-27T05:13:35Z",
    "validation": "requirements-analyst confirmed scope and constraints are clear"
  },
  {
    "from_agent": "requirements-analyst",
    "to_agent": "code-architect",
    "phase": "requirementstodesign",
    "artefacts": ["REQUIREMENTS.md"],
    "trigger": "Functional and non-functional requirements documented and prioritized",
    "timestamp": "2025-11-27T05:20:00Z",
    "validation": "code-architect verified all acceptance criteria are testable and no blocking ambiguities remain"
  },
  {
    "from_agent": "code-architect",
    "to_agent": "feature-developer",
    "phase": "designtoimplementation",
    "artefacts": ["SOLUTIONPLAN.md", "docs/adr/ADR-001-ui-framework-selection.md", "docs/adr/ADR-002-git-integration-approach.md"],
    "trigger": "Architecture and implementation plan complete with ADRs for key decisions",
    "timestamp": "2025-11-27T05:30:00Z",
    "validation": "feature-developer confirmed module boundaries, APIs, and data models are sufficiently specified"
  }
]
```

## Conventions

- All times use ISO 8601 UTC.
- `from_agent` / `to_agent` must match actual agent `name` in `.github/agents`.
- `phase` uses `planningtorequirements`, `requirementstodesign`, `designtoimplementation`, `implementationtotesting`, `testingtodeployment`, `deploymenttomaintenance`, or a clearly named custom phase.
- `artefacts` are repository-relative paths.
- `validation` describes what the receiving agent checked before accepting ownership.
