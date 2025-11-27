# Project State and Handoff Ledger

This file persists SDLC phase, ownership, and handoff history so agents can coordinate over time.

## Current project state

- `current_phase`: start
- `active_agents`: []
- `blocked_tasks`: []

Agents must update this section when they change phase, become active, or encounter blockers.

## Handoffs

A chronological list of JSON objects describing ownership transfers. Append new entries; never edit history.

```json
[]
```

## Conventions

- All times use ISO 8601 UTC.
- `from_agent` / `to_agent` must match actual agent `name` in `.copilot/agents`.
- `phase` uses `planningtorequirements`, `requirementstodesign`, `designtoimplementation`, `implementationtotesting`, `testingtodeployment`, `deploymenttomaintenance`, or a clearly named custom phase.
- `artefacts` are repository-relative paths.
- `validation` describes what the receiving agent checked before accepting ownership.
