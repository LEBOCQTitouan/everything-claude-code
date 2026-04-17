---
name: backlog-management
description: >-
  Persistent backlog for capturing implementation ideas outside active /spec
  sessions. Defines entry format, optimization rules, matching heuristics,
  and index management. Used by the backlog-curator agent and /backlog command.
  TRIGGER when: user says "add to backlog", "backlog idea", "park this idea",
  "save for later", or invokes /backlog.
  DO NOT TRIGGER when: user wants immediate execution or is already in /spec.
origin: core
metadata:
  version: "1.0.0"
---

# Backlog Management

Capture, challenge, optimize, and manage implementation ideas in a persistent
backlog so they are ready to execute when the time comes.

## When to Use

- Capturing an idea during code review, debugging, or casual thinking
- Parking a feature idea that isn't ready for `/spec` yet
- Reviewing what's in the backlog before starting new work
- Cross-referencing a prompt against existing backlog entries

### Do Not Use When

- The idea is ready to execute now (use `/spec` directly)
- You are already inside a `/spec` session (finish the plan first)

## Entry Format

Each backlog entry is a Markdown file with YAML frontmatter:

```markdown
---
id: BL-NNN
title: Short descriptive title
status: open          # open | in-progress | implemented | promoted | archived
created: YYYY-MM-DD
promoted_to: ""       # e.g., "US-001" or PR link (set on promotion)
tags: [tag1, tag2]
scope: MEDIUM         # TRIVIAL | LOW | MEDIUM | HIGH | EPIC
target_command: /spec # /spec | /spec refactor | /spec security | /e2e | /doc-suite | /audit
---

## Optimized Prompt

The idea rewritten as a self-contained, ready-to-paste prompt optimized
for the target command. Includes context, acceptance criteria, scope
boundaries, and verification steps.

## Original Input

The raw idea as the user expressed it, preserved verbatim.

## Challenge Log

The Challenge Log contains the grill-me output: stages, questions, answers, and challenge threads.
Captures decisions and rationale from each grill-me stage.

## Related Backlog Items

Links to other entries that overlap or depend on this one.
```

## ID Generation

- Sequential: `BL-001`, `BL-002`, etc.
- Read `docs/backlog/BACKLOG.md` to find the highest existing ID
- If no backlog exists, start at `BL-001`

## File Naming

- Pattern: `docs/backlog/BL-NNN-<slug>.md`
- Slug: lowercase, hyphens, max 40 characters, derived from title
- Example: `docs/backlog/BL-003-rate-limiting-api-endpoints.md`

## Index File

`docs/backlog/BACKLOG.md` is a status table of all entries:

```markdown
# Backlog

| ID | Title | Status | Scope | Target | Created |
|----|-------|--------|-------|--------|---------|
| BL-001 | Add rate limiting to API endpoints | open | MEDIUM | /spec | 2026-03-15 |
| BL-002 | Refactor auth middleware | promoted | HIGH | /spec refactor | 2026-03-10 |
```

Update this index whenever an entry is added, promoted, or archived.

## Status Transitions

Use `ecc backlog update-status BL-NNN <status>` to change status programmatically. This updates the individual file and auto-reindexes BACKLOG.md.

```
open → in-progress   Claimed by active worktree (automatic via reindex)
open → implemented   Implementation complete, verified
open → promoted      Picked up by /spec or another command
open → archived      No longer relevant
```

Any-to-any transitions are allowed (no validation graph enforced).

- **in-progress**: Claimed by an active worktree session.
- **implemented**: The feature described by this entry has been built and verified.
- **promoted**: The idea was picked up by `/spec` or another command.
  Set `promoted_to` to the User Story ID, PR link, or commit hash.
- **archived**: The idea is no longer relevant. Keep the file for history.

## Optimization Rules

Transform a raw idea into a ready-to-execute prompt for the target command:

### By Target Command

| Target | Optimization Focus |
|--------|--------------------|
| `/spec` | Acceptance criteria, scope boundaries, phased breakdown, test targets |
| `/spec refactor` | Current pain points, target architecture, files affected, risk areas |
| `/spec security` | Threat model, attack vectors, compliance requirements, affected surfaces |
| `/e2e` | User flows, critical paths, edge cases, expected outcomes |
| `/doc-suite` | Documentation gaps, target audience, modules to document |
| `/audit` | Audit scope, specific concerns, areas to focus on |

### Optimization Checklist

The optimized prompt MUST include:
- [ ] Clear task description with context
- [ ] Tech stack reference (or "detect from project")
- [ ] Target command invocation
- [ ] Acceptance criteria (how to know it's done)
- [ ] Scope boundaries (what NOT to do)
- [ ] Verification steps

## Matching Heuristics

Cross-reference a prompt against open backlog entries using these signals:

| Signal | Weight | How to Check |
|--------|--------|--------------|
| Title keyword overlap | 1 | Tokenize title, compare word intersection |
| Tag intersection | 1 | Compare tag arrays |
| Scope overlap (same files/modules) | 1 | Compare mentioned files or directories |
| Content similarity | 1 | Keyword overlap in Optimized Prompt body |
| Same target command | 0.5 | Compare target_command fields |

### Confidence Scoring

| Score | Confidence | Action |
|-------|------------|--------|
| 3+ | HIGH | Surface: "These backlog items are directly related" |
| 2 | MEDIUM | Surface: "These items may be related" |
| 1 | LOW | Do not surface |
| 0 | NONE | Do not surface |

Surface HIGH and MEDIUM matches only.

## Duplicate Detection

Before adding a new entry, compare against all open entries:

1. Tokenize the new title into keywords (remove stop words)
2. Compare against each open entry's title keywords and tags
3. If 60%+ keyword overlap with any existing entry → flag as potential duplicate
4. Ask: merge into existing, replace existing, or add separately

## Related Components

| Component | Relationship |
|-----------|-------------|
| `prompt-optimizer` | Used internally to optimize the raw idea into a polished prompt |
| `/spec` | Primary consumer — cross-references backlog in Phase 0.25 |
| `backlog-curator` | Agent that implements the curation flow using this skill |
| `/backlog` | Command that invokes the backlog-curator agent |
