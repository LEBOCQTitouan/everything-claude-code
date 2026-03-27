---
id: BL-064
title: Full app cartography — user journeys, data flows, and element registry across all sessions
status: open
created: 2026-03-26
promoted_to: ""
tags: [documentation, cartography, user-journeys, data-flows, diagrams, all-commands]
scope: EPIC
target_command: /spec dev
---

## Optimized Prompt

Extend **BL-056** (context-aware doc generation at end of `/implement`) into a **universal cartography system** that runs at the end of **every session and every command** — not just `/implement`. The system automatically documents all user journeys, data flows, and app elements so the entire application is mapped.

**Project**: everything-claude-code (Rust, hexagonal architecture, 7 crates)
**Extends**: BL-056

### What "cartography" means

Three documentation layers, each produced as both **Mermaid diagrams** and **structured Markdown**:

#### 1. User Journey Registry (`docs/cartography/journeys/`)

- One file per user journey (e.g., `developer-adds-feature.md`, `admin-configures-hooks.md`)
- **User categories**: all actor types (developers, end-users, admins, CI bots, AI agents in future sessions)
- **Actions**: every action each category can perform — CLI commands, API calls, UI interactions, config changes
- Each journey includes: actor, trigger, step sequence, decision points, outcomes, related data flows
- High-level overview diagram (sequence/flowchart) + detailed step-by-step breakdown
- User categories should be referenced and reused when creating user stories in `/spec` and other commands

#### 2. Data Flow Registry (`docs/cartography/flows/`)

- One file per data flow (e.g., `spec-artifact-persistence.md`, `hook-execution-pipeline.md`)
- **Every granularity**: function calls between modules, event propagation, state changes (internal) AND API calls, file I/O, database queries, webhook triggers (external)
- Each flow includes: source, destination, data shape, transformation steps, error paths
- High-level overview (C4/flowchart) + detailed per-step sequence diagram

#### 3. Element Registry (`docs/cartography/elements/`)

- Master index of all app elements: commands, agents, skills, hooks, rules, crates, ports, adapters, domain entities
- Each element entry: purpose, relationships (uses/used-by), data flows it participates in, user journeys it appears in
- Cross-reference matrix linking elements ↔ journeys ↔ flows

### Trigger model

- Runs at the **end of every session** that produces meaningful changes, across **all commands** (`/spec`, `/design`, `/implement`, `/audit-*`, `/verify`, `/backlog`, direct edits)
- Implemented as a **Stop hook** or **post-session phase** that detects what changed and updates only the affected cartography files
- **Merge strategy**: delta merge — append new steps, update changed ones, never overwrite prior content
- **Interrupted sessions**: discard partial cartography updates — only commit complete, consistent docs

### Multi-level documentation

Every cartography artifact must have:
- **High-level overview**: 1-page summary with Mermaid diagram, readable in 2 minutes
- **Deep detail**: full step-by-step breakdown, every function call, every data transformation
- Both levels cross-linked

### Audience

All docs must serve four audiences simultaneously:
1. **Developer who wrote the code** — memory aid, design rationale
2. **New contributor onboarding** — understand system behavior without reading all source
3. **Auditor** — verify completeness, trace data flows end-to-end
4. **AI agent in future session** — structured context for informed decision-making

### Integration with existing commands

- `/spec` reads the user category registry when creating user stories — ensures new features reference known actors
- `/design` reads the data flow registry to identify integration points
- `/implement` Phase 7.5 (BL-056) becomes part of this broader cartography system
- `/audit-doc` gains a cartography completeness check

### Acceptance criteria

- [ ] Cartography directory structure created (`docs/cartography/journeys/`, `flows/`, `elements/`)
- [ ] Stop hook or post-session mechanism detects changes and triggers cartography updates
- [ ] User journey docs generated with both Mermaid diagrams and structured Markdown
- [ ] Data flow docs generated at every granularity (internal + external)
- [ ] Element registry with cross-reference matrix maintained
- [ ] High-level overview + deep detail for every artifact
- [ ] Delta merge strategy implemented (no overwrites, no duplicates)
- [ ] Partial updates discarded on interrupted sessions
- [ ] `/spec` reads user categories from cartography when building user stories
- [ ] Works on any project in the current working directory (not ECC-specific)
- [ ] BL-056 Phase 7.5 integrated as one trigger point within this system
- [ ] CHANGELOG.md updated
- [ ] ADR created for cartography architecture decision

### Scope boundaries — do NOT

- Do not replace existing `/doc-suite` pipeline — this is complementary
- Do not generate cartography for unchanged areas — delta only
- Do not block session completion on cartography generation failure
- Do not hardcode ECC-specific knowledge — must work on any project

### Verification steps

1. Run a session that adds a new CLI command — verify journey + flow + element docs created
2. Run a second session modifying the same command — verify delta merge (no duplicates)
3. Interrupt a session mid-cartography — verify no partial docs committed
4. Run `/spec` on a new feature — verify user categories from cartography are referenced
5. Check all docs have both high-level and detailed views
6. Check Mermaid diagrams render correctly

## Original Input

"I want in every session to have doc automatically made on all user journeys creation and update, data flows inside/outside the app for any uses so that all elements in the created app are cartographied. If any addition could be made give upgrades to this proposition."

## Challenge Log

**Q1**: What exactly counts as a "user journey"?
**A1**: All actions made by devs or users. All categories of users should be cartographied and used when doing US in other commands.

**Q2**: What counts as a "data flow inside/outside the app"?
**A2**: Literally every data movement at any granularity.

**Q3**: Should this fire on every session or only meaningful ones?
**A3**: Every session and every command.

**Q4**: Is this for ECC itself or the apps ECC builds?
**A4**: Whatever project is in the current working directory.

**Q5**: Merge, overwrite, or version existing docs?
**A5**: Merge (delta merge).

**Q6**: What happens on interrupted sessions?
**A6**: Discard partial updates.

**Q7**: Extend BL-056 or separate mechanism?
**A7**: Extend BL-056.

**Q8**: Output format?
**A8**: Both Mermaid diagrams and structured Markdown.

**Q9**: How deep should cartography go?
**A9**: Deep detail + high-level overview.

**Q10**: Who reads these docs?
**A10**: Developers, new contributors, auditors, and AI agents.

## Related Backlog Items

- **BL-056**: Context-aware doc generation at end of /implement — this entry extends BL-056 from /implement-only to all commands
- BL-038: TaskCreate in audit-full and doc-orchestrator — parallel doc generation concern
- BL-029: Persist specs as file artifacts — cartography consumes spec artifacts
- BL-030: Persist tasks.md — cartography consumes task artifacts

## Suggested Enhancements

These emerged during the grill-me interview as potential upgrades:

1. **Change impact analysis**: Before each session starts, show which existing journeys/flows will be affected by the planned changes
2. **Staleness detection**: Flag cartography entries that haven't been updated in N sessions despite their source code changing
3. **Coverage dashboard**: A summary showing % of app elements covered by cartography, highlighting gaps
4. **Interactive navigation**: Generate an HTML page with clickable journey→flow→element cross-references
5. **Diff view**: Show what cartography changed in each session as part of the commit message
