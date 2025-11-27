# Agent Handoff and Ownership

This document defines how SDLC agents transfer ownership, what artifacts move at each handoff, and what validation each receiving agent must perform before accepting ownership.

## Core principles
- Never hand off without complete context: task state, decisions and rationale, and relevant metadata (acceptance criteria, dependencies, quality gates).
- Handoffs are one-way ownership transfers between SDLC phases; feedback loops are explicit and short.
- The SDLC Manager maintains meta-ownership via `PROJECTSTATE.md` and coordinates all phase transitions.

## Standard handoff points

1. **Planning → Requirements**  
   - From: Product/Project owner (human) or planning agent  
   - To: `requirements-analyst`  
   - Deliverables: project charter, scope, constraints, initial risks.  
   - Validation: `requirements-analyst` confirms constraints, success metrics, and stakeholders are clear.

2. **Requirements → Design**  
   - From: `requirements-analyst`  
   - To: `implementation-planner` and `code-architect`  
   - Deliverables: `REQUIREMENTS.md`, prioritized backlog, traceability to business goals.  
   - Validation: receiving agents verify all acceptance criteria are testable and no blocking ambiguities remain.

3. **Design → Implementation**  
   - From: `implementation-planner` and `code-architect`  
   - To: `feature-developer` (and `bug-fix-agent` for maintenance work)  
   - Deliverables: `SOLUTIONPLAN.md`, ADRs, architecture diagrams and interface contracts.  
   - Validation: developers confirm module boundaries, APIs, and data models are sufficiently specified.

4. **Implementation → Testing**  
   - From: `feature-developer` / `bug-fix-agent`  
   - To: `test-specialist`, then `qa-engineer`  
   - Deliverables: merged code, basic unit tests, PR links, manual notes.  
   - Validation: testing agents confirm coverage is above minimum threshold and acceptance criteria are addressable with tests.

5. **Testing → Deployment**  
   - From: `qa-engineer`  
   - To: `devops-engineer` and `release-manager`  
   - Deliverables: passing test reports, bug list resolved, performance/security reports, QA sign-off.  
   - Validation: DevOps and Release confirm gates (tests, security, compliance) are green before planning releases.

6. **Deployment → Maintenance**  
   - From: `release-manager`  
   - To: monitoring/maintenance loop (future agents) and back to `bug-fix-agent` / `feature-developer` as needed  
   - Deliverables: release notes, deployment logs, health dashboards, rollback plan.  
   - Validation: post-deploy checks pass and monitoring is active.

## Handoff record format

Each handoff appends a JSON block to `PROJECTSTATE.md` under a `## Handoffs` section:

```json
{
  "from_agent": "implementation-planner",
  "to_agent": "feature-developer",
  "phase": "designtoimplementation",
  "artefacts": ["SOLUTIONPLAN.md", "docs/adr/ADR-001.md"],
  "trigger": "Architecture review approved; non-functional requirements addressed",
  "timestamp": "2025-11-22T00:00:00Z",
  "validation": "feature-developer confirmed architecture is sufficiently detailed"
}
```

Agents must update this log when they relinquish ownership and verify the previous entry when they assume ownership.
