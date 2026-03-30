---
id: BL-086
title: "Knowledge sources registry — curated reference list with quadrant organization and command integration"
scope: HIGH
target: "/spec"
status: implemented
tags: [knowledge, sources, research, registry, documentation, spec, audit, implement, design, review, catchup]
created: 2026-03-28
promoted_to: ""
---

# BL-086: Knowledge sources registry — curated reference list with quadrant organization and command integration

## Optimized Prompt

```
/spec

Build a knowledge sources registry for ECC — a curated, project-scoped list of
reference sources (GitHub repos, docs, blogs, packages, talks, academic and
conference papers) that Claude and humans can consult during research, design,
and audit phases.

Tech stack: detect from project (Rust + Markdown conventions in ECC).

## Context

ECC commands like /spec, /audit, /implement, and /design currently do ad-hoc
web research with no persistent record of what sources are authoritative for
this project. This creates duplicated research effort across sessions and no
shared vocabulary of trusted references. The registry solves this by making
sources a first-class project artifact committed to docs/.

## Acceptance Criteria

### File format
- Registry lives at `docs/sources.md` — committed, human-readable Markdown
- Four quadrants organize entries: Adopt, Trial, Assess, Hold (Technology Radar model)
- Each quadrant contains named subjects (e.g., "Async Rust", "Claude API") mapping
  to codebase modules where relevant
- Each source entry includes: URL, title, type (repo/doc/blog/package/talk/paper),
  quadrant, subject, added-by (human/claude), added-date, last-checked date,
  and an optional deprecation reason

### Lifecycle rules
- Stale or unreachable sources are flagged for human review — never silently removed
- Deprecated sources remain in the file with a deprecation reason
- All lifecycle operations are deterministic: adding, re-interrogating, flagging,
  and deprecating must produce the same result when run twice on the same input

### Human contribution flow
- Humans add entries to a designated `## Inbox` section at the top of the file
- During processing, Claude moves inbox entries into the correct quadrant and subject
- CLAUDE.md gets a single line summary + pointer to docs/sources.md — no content
  duplication into CLAUDE.md

### Command integrations
- `/spec` Phase 0 research: check docs/sources.md for prior art in the relevant
  subject before designing
- `/implement` Phase 0 research: consult sources mapped to the module being modified
- `/design`: reference architectural sources in the relevant quadrant during design
- `/audit` (especially audit-evolution and audit-web): re-interrogate sources in
  relevant subjects for new releases, deprecations, and security advisories
- `/review`: check if implementation aligns with patterns from watched sources
- `/catchup`: summarize what changed in watched sources since last check
- All integrations are on-demand fetch/check — do not preload into context;
  return only actionable findings to avoid polluting context

### Rust CLI support
- `ecc sources list [--quadrant <q>] [--subject <s>]` — list sources
- `ecc sources add <url> --title <t> --type <type> --quadrant <q> --subject <s>` — add
- `ecc sources check` — check all sources for reachability, flag stale ones
- `ecc sources reindex` — rebuild docs/sources.md from canonical data (if split)

## Scope Boundaries

- Do NOT implement automated source scraping or content ingestion
- Do NOT add sources to LLM context automatically on every command invocation
- Do NOT replace docs/specs/ or docs/adr/ — this is a reference list, not a spec store
- Do NOT build a web UI or external service
- Start with docs/sources.md as the single source of truth; split only if it
  grows beyond 800 lines

## Phased Breakdown

Phase 1: File format + schema
- Define the Markdown schema for docs/sources.md
- Write the Inbox → quadrant flow documentation
- Add CLAUDE.md pointer

Phase 2: Command integrations
- Update /spec, /implement, /design to consult sources in Phase 0
- Update /audit-evolution and /audit-web to re-interrogate sources
- Update /review and /catchup with source-awareness hooks

Phase 3: Rust CLI commands
- Implement `ecc sources` subcommands
- Add reachability check with stale flagging
- Add deterministic reindex

## Verification Steps

- docs/sources.md exists with at least one entry per quadrant after Phase 1
- Inbox entries are correctly moved to quadrants during processing
- A stale URL is flagged (not deleted) after `ecc sources check`
- /spec Phase 0 produces a "Consulted sources" section when relevant entries exist
- /catchup lists source changes since last check when sources have been updated
- All ecc sources CLI commands pass `cargo test` and `cargo clippy -- -D warnings`
```

## Original Input

A sources/knowledge-base registry for ECC. A project-scoped, committed document
in docs/ listing all reference sources — GitHub repos, docs, blogs, packages,
talks, academic and conference papers — organized into quadrants with module
mapping, consumed on-demand by /spec, /audit, /implement, /design, /review,
and /catchup. Humans and Claude both contribute. Stale sources are flagged, not
deleted. Deprecated sources stay with a reason. CLAUDE.md stays short with just
a pointer.

## Challenge Log

Mode: backlog-mode (escalated to HIGH scope — all 5 stages)
Questions answered: 8/8

### Stage 1: Clarity
**Q1**: What types of sources does this registry need to track?
**A**: All source types — GitHub repos, docs, blogs, packages, talks, academic
papers, research papers, conference talks.

**Q2**: Where does this registry live and who consumes it?
**A**: Project-scoped, lives in docs/, committed to repo for human readability.

### Stage 2: Assumptions
**Q3**: When should sources be consulted — preloaded into context on every run,
or fetched on demand?
**A**: On-demand fetch/check. Return only actionable findings to avoid polluting context.

**Q4**: Who adds entries — only humans, or also Claude? How are entries organized?
**A**: Both human and Claude add sources. Quadrant organization. Humans add to a
designated spot; Claude rearranges into correct quadrants during processing.

### Stage 3: Edge Cases
**Q5**: What happens to stale or unreachable sources?
**A**: Flagged for human review, not silently removed.

**Q6**: How does the quadrant/subject organization map to code?
**A**: Quadrants/subjects map to codebase modules. Sources re-interrogated when
modifying related modules.

### Stage 4: Alternatives
**Q7**: Should CLAUDE.md be updated to reference sources, or stay separate?
**A**: CLAUDE.md stays short — just a link/summary to the sources file.

### Stage 5: Stress Test
**Q8**: What is the policy for deprecated or outdated sources?
**A**: Deprecated sources stay with a reason. All source lifecycle operations
are deterministic.

## Related Backlog Items

- BL-028: Web search in plan commands — sources registry provides a curated alternative to raw web search
- BL-049: Offload web research phase to Task subagents — sources registry feeds Phase 0 research
- BL-081: Web-based upgrade audit command — sources registry is the input for re-interrogation during audit-evolution
