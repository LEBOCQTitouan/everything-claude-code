# One-Time TEMPORARY Marker Audit Report

Spec: `docs/specs/2026-04-18-claude-md-temp-marker-lint/spec.md`
Generated: 2026-04-18 (post-fix, commit SHA at capture time recorded in git log)
Tool: `ecc validate claude-md markers --audit-report`

## Context

This is the one-time audit deliverable required by US-004 (AC-004.4) of the v1 spec. It captures the state of `TEMPORARY (BL-NNN)` markers in the repo BOTH before and after the stale-line removal (US-005), to document the regression anchor (AC-004.2) and the exit criterion (AC-004.3).

The v1 lint rule (`ecc validate claude-md markers`) scans every `CLAUDE.md` and `AGENTS.md` under the project root (skip-list: `.git/`, `target/`, `node_modules/`, `.claude/worktrees/`; depth cap 16; symlinks skipped) and flags markers whose backing `docs/backlog/BL-NNN-*.md` file is missing.

## Before (pre-fix state)

The single live failing case that triggered this entire spec. Captured from `CLAUDE.md` line 108 at session-start commit (SHA `1d4e56fb`, 2026-04-17):

| File | Line | Marker ID | Status |
|------|------|-----------|--------|
| CLAUDE.md | 108 | BL-150 | missing |

**Rationale**: BL-150's underlying worktree-GC fix shipped in an earlier release (see `CHANGELOG.md` entry for "Worktree GC deletes active worktrees (BL-150)" — `parent_id()` PID fix, unmerged count `unwrap_or(u64::MAX)`, 30-minute .git recency guard, `WorktreeError → WorktreeGcError` rename). The `TEMPORARY (BL-150)` warning in `CLAUDE.md` was never removed when that work shipped. The v1 lint rule would have caught this drift at the next CI run after it became stale.

## After (post-fix state)

Captured by running `./target/release/ecc validate claude-md markers --audit-report` at commit SHA `811cdc3c` (after the CLAUDE.md:108 removal commit and the BL-158 companion-entry commit):

| File | Line | Marker ID | Status |
|------|------|-----------|--------|

**Zero `missing` rows.** Post-fix, no TEMPORARY markers exist in the repo. The exit criterion for this spec is satisfied (AC-004.3).

## Regression anchor

The row `| CLAUDE.md | 108 | BL-150 | missing |` in the **Before** section serves as the AC-004.2 regression anchor. PC-039 asserts the presence of this exact row in this document.

If future work re-introduces a similar stale marker, the v1 lint rule will catch it at CI time via `ecc validate claude-md markers --strict` (AC-006.1).

## Follow-up tracked

The v1 lint uses presence-only semantics (file on disk = resolved). This creates a governance loophole — archiving an entry silences the warning without doing the work. That loophole is tracked as `BL-158: Frontmatter-aware TEMPORARY marker validation (v2)`, filed as a companion deliverable of this spec (see v1 spec AC-001.12 and decision #4).
