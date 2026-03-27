---
id: BL-056
title: Context-aware doc generation step at end of /implement
status: implemented
created: 2026-03-22
promoted_to: ""
tags: [implement, doc-generation, MODULE-SUMMARIES, diagrams, mermaid, architecture]
scope: HIGH
target_command: /spec dev
---

## Optimized Prompt

Add a standalone **Phase 7.5** to `/implement` — positioned after doc updates (Phase 6) and before writing implement-done.md (Phase 7) — that generates supplemental architecture and design-rationale documentation using the full session context available at that point.

**Project**: everything-claude-code (Rust, hexagonal architecture, 7 crates)
**Target file**: `commands/implement.md`

### Context available at Phase 7.5

By the time Phase 7.5 runs, the session holds:
- Full spec (acceptance criteria, decisions, grill-me rationale)
- Full design (bounded contexts, file changes table, pass conditions)
- TDD results (wave plan, PC outcomes, files actually changed)
- ADRs created (Phase 6)
- Code review findings (Phase 5)

This context would be lost after the session ends. Phase 7.5 captures it as persistent docs before the session closes.

### What Phase 7.5 must produce

Phase 7.5 launches **two independent Task subagents in parallel**:

#### Subagent A — Module Summary Updater

Updates `docs/MODULE-SUMMARIES.md` with entries for every file or module changed during the TDD loop.

Each updated entry must include:
- Module purpose (1-2 sentences, derived from spec and implementation)
- Key functions/types introduced or modified (not exhaustive — signal-bearing only)
- Cross-links to the relevant spec artifact (`docs/specs/YYYY-MM-DD-<slug>/`) and any new ADRs
- Design rationale note: *why* the module was shaped this way (captured from spec decisions/grill-me answers)

Rules:
- Do NOT regenerate the full file — only update stale or add missing entries
- Preserve the existing header comment and table structure
- Allowed tools: Read, Write, Edit, Grep, Glob

#### Subagent B — Diagram Generator

Creates or updates Mermaid diagrams in `docs/diagrams/` for the implemented feature.

Diagrams to generate (select based on what changed):
- **Sequence diagram** if new cross-module flows were introduced (e.g., new command → agent interaction)
- **Flowchart** if a new decision path or state machine was added
- **Module dependency graph update** (`docs/diagrams/module-dependency-graph.md`) if new crate-level dependencies were introduced
- **C4 component diagram** if a new bounded context or port/adapter pair was added

Each diagram file must:
- Use the existing format in `docs/diagrams/` (Mermaid fenced code block, generator comment header)
- Be registered in `docs/diagrams/INDEX.md`
- Be cross-linked from the corresponding MODULE-SUMMARIES entry

Rules:
- Derive diagram content from the design's bounded contexts, file changes table, and PC results — not from re-reading source code
- If no diagram is warranted (e.g., trivial single-file change), skip and record "No diagram generated" in implement-done.md
- Allowed tools: Read, Write, Edit, Grep, Glob

### Integration with /implement

1. Phase 7.5 runs AFTER Phase 6 (doc updates) and BEFORE Phase 7 (implement-done.md)
2. Both subagents run in parallel (independent output targets)
3. Each subagent's output is committed before Phase 7 proceeds:
   - `docs: update MODULE-SUMMARIES for <feature>`
   - `docs(diagrams): add <feature> diagrams`
4. Phase 7 (implement-done.md) gains a new `## Supplemental Docs` section listing what Phase 7.5 produced
5. Phase 8 stop-gate gains a new check: `## Supplemental Docs` section is present in implement-done.md

### /doc-suite background trigger

After Phase 8 (Final Verification), trigger `/doc-suite` as a background process:
- Use `!bash -c "claude /doc-suite &"` or equivalent background invocation
- The full /doc-suite run is independent — it picks up the supplemental docs just written
- Do NOT block on /doc-suite completion

### Acceptance criteria

- [ ] `commands/implement.md` has a Phase 7.5 section with the two-subagent structure
- [ ] Module Summary Updater agent spec is fully defined (prompt, tools, output format)
- [ ] Diagram Generator agent spec is fully defined (prompt, tools, diagram selection logic)
- [ ] Both subagents run in parallel using Task dispatch
- [ ] Each subagent commits its output atomically before Phase 7 proceeds
- [ ] implement-done.md schema gains `## Supplemental Docs` section
- [ ] Phase 8 stop-gate checks for `## Supplemental Docs`
- [ ] Background /doc-suite trigger is added after Phase 8
- [ ] CHANGELOG.md updated
- [ ] ADR created if significant architectural decision made

### Scope boundaries — do NOT

- Do not modify the existing /doc-suite pipeline or its agents
- Do not replace Phase 6 doc updates — Phase 7.5 is supplemental, not a replacement
- Do not create new dedicated agents in `agents/` — the subagent logic lives inline in the Phase 7.5 prompt unless the spec author decides otherwise
- Do not re-read source files from scratch — derive doc content from context already in session
- Do not block /implement completion on /doc-suite finishing

### Verification steps

1. Run `/implement` on a medium-scope feature and confirm Phase 7.5 fires
2. Verify `docs/MODULE-SUMMARIES.md` has updated entries for files changed in the TDD loop
3. Verify at least one diagram was added or updated in `docs/diagrams/`
4. Verify `docs/diagrams/INDEX.md` is updated
5. Verify implement-done.md contains `## Supplemental Docs` section
6. Verify two commits from Phase 7.5 appear in `git log`
7. Verify /doc-suite is triggered in background after Phase 8

## Original Input

"Invoke /doc-suite as background at end of /implement, but supplement with additional rich docs leveraging full implementation context. Supplemental docs go to docs/MODULE-SUMMARIES.md and docs/diagrams/. Purpose is to help understand how the project was thought/designed. Standalone step in /implement, independent of /doc-suite pipeline, using its own agents that leverage the session's full context."

## Challenge Log

**Q1**: Should this (A) invoke /doc-suite as background at the end of /implement, (B) replace /doc-suite with a context-aware alternative, or (C) supplement /doc-suite with additional rich docs that leverage the implementation context?
**A1**: A + C — invoke /doc-suite as background and also generate supplemental docs using full implementation context.

**Q2**: Where should the supplemental docs land, and what is their purpose?
**A2**: docs/MODULE-SUMMARIES.md and docs/diagrams/. Purpose: help understand how the project was thought/designed.

**Q3**: Should this be a standalone step in /implement, or wired through the existing /doc-suite pipeline?
**A3**: Standalone step in /implement, independent of /doc-suite pipeline, using its own agents that leverage the session's full context.

## Related Backlog Items

- BL-030: Persist tasks.md as trackable artifact — Phase 7.5 consumes the same spec directory structure
- BL-029: Persist specs and designs as versioned file artifacts — Phase 7.5 reads spec_path and design_path from state.json
- BL-038: TaskCreate in audit-full and doc-orchestrator — parallel concern (Task-based doc generation)
