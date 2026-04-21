---
id: BL-158
title: Frontmatter-aware TEMPORARY marker validation (v2)
status: open
scope: MEDIUM
target: /spec-dev
tier: 6
tags: [validation, backlog, governance, drift]
created: 2026-04-18
---

# BL-158: Frontmatter-aware TEMPORARY marker validation (v2)

## Problem

The v1 `ecc validate claude-md markers` lint (filed in spec `2026-04-18-claude-md-temp-marker-lint`) uses **presence-only** semantics: a `TEMPORARY (BL-NNN)` warning is treated as "resolved" whenever the file `docs/backlog/BL-NNN-*.md` exists on disk, regardless of its `status` frontmatter field.

This creates a **governance loophole** (flagged by the solution-adversary during the v1 spec review and deliberately deferred as a conscious v1 simplification). A contributor can silence a stale TEMPORARY warning simply by changing the backlog entry's status from `open` to `archived` — without doing the underlying work and without removing the warning comment. The warning's purpose (flag unfinished drift) is thereby circumvented.

Concrete scenario: `CLAUDE.md` contains `TEMPORARY (BL-200): Fix race condition in session merge.` The associated BL-200 file exists on disk but was archived six months ago because the team decided not to fix the race condition. The lint reports the marker as "resolved". The race condition still exists. The warning survives. Both signals are wrong.

## Proposed v2 behavior

Parse the backlog entry's YAML frontmatter and treat `status` as semantically meaningful:

| Frontmatter status | v1 verdict | v2 verdict |
|--------------------|------------|------------|
| `open` | resolved | **unresolved** (work not done → warning is legitimate) |
| `in-progress` | resolved | **unresolved** (work in flight → warning is legitimate) |
| `implemented` | resolved | resolved (work shipped → warning can be removed) |
| `archived` | resolved | **unresolved** (considered-and-dropped ≠ shipped; warning still a drift signal) |
| `promoted` | resolved | resolved (entry absorbed into another → upstream tracks it) |
| (missing file) | missing | missing |

This inverts the default for the two "work-not-done" terminal states (`archived`, `open`, `in-progress`).

## Why deferred from v1

- Presence-only is simpler: no YAML parsing, no frontmatter schema coupling, no cross-crate dependency from the lint use case to `parse_frontmatter`.
- BL-150 (the triggering case) was genuinely `implemented` — the warning WAS stale. v1 correctly flags and fixes it.
- Governance loophole is low-exploit-likelihood in a solo-maintainer project. The loophole becomes real only when the archive decision and the warning decision are made by different people with different motivations.

## Acceptance criteria (high-level)

- AC-1: `ecc validate claude-md markers --strict` emits diagnostic when `TEMPORARY (BL-NNN)` references a BL whose status is `open`, `in-progress`, or `archived`.
- AC-2: Same command emits no diagnostic when status is `implemented` or `promoted`.
- AC-3: Missing frontmatter or malformed YAML is treated as `archived` (conservative — warn the maintainer).
- AC-4: A transition document (`docs/adr/` or `CHANGELOG.md` entry) announcing the semantic change must accompany v2 so existing markers aren't silently re-classified.
- AC-5: v1 → v2 transition must include a migration pass: walk all existing `TEMPORARY (BL-NNN)` markers, emit an audit report of newly-unresolved markers under v2 semantics, and require each to be either fixed or re-archived with justification before `--strict` is re-enabled.

## Related

- v1 spec: `docs/specs/2026-04-18-claude-md-temp-marker-lint/` — introduces the lint with presence-only semantics. Decision #4 of that spec records the v1 stance.
- v1 spec AC-001.12: explicitly documents the loophole and names this entry as the counterweight.
- Adversary finding: Round 1 solution-adversary (spec phase) flagged "archived=resolved governance loophole" as a decision-completeness weakness; deferred to this entry.

## Optimized Prompt (for future /spec-dev)

```
Upgrade `ecc validate claude-md markers` from presence-only to frontmatter-aware semantics.

Read the YAML frontmatter of each matched BL-NNN file. The `status` field drives
the resolved/unresolved decision per the table in BL-158. Missing or malformed
frontmatter must be treated conservatively (unresolved).

Include a migration pass that audits all existing markers under the new
semantics BEFORE re-enabling --strict in CI, to prevent a sudden flood of
"newly unresolved" markers from blocking PRs.

Companion to v1 spec at docs/specs/2026-04-18-claude-md-temp-marker-lint/.
Governance loophole documented in v1 spec decision #4 and AC-001.12.
```
