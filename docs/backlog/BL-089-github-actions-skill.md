---
id: BL-089
title: "GitHub Actions skill + branch isolation hook for CI/CD workflow development"
scope: HIGH
target: "/spec dev"
status: open
tags: [skill, github-actions, ci-cd, hooks, branch-isolation]
created: 2026-03-28
related: [BL-087, BL-088, BL-065]
---

# BL-089: GitHub Actions Skill + Branch Isolation Hook

## Problem

Claude Code can write and debug GitHub Actions workflows, but ECC has no structured skill for it. Developers editing `.github/workflows/` on main risk breaking CI for everyone. No hook prevents this — unlike the phase-gate which protects source files during plan/solution phases.

## Proposed Solution

### 1. Generic `github-actions` Skill

New skill at `skills/github-actions/SKILL.md` covering:

- **Workflow patterns**: CI (test/lint/build matrix), CD (release on tag), cron (scheduled maintenance)
- **Rust-specific patterns**: Cross-compilation matrix (macOS arm64/x64, Linux x64), cargo-specific steps, artifact upload with checksums
- **gh CLI debugging cheat sheet**: `gh run list`, `gh run view --log-failed`, `gh workflow run`
- **Best practices**: Reusable workflows, composite actions, caching (`actions/cache` for `~/.cargo`), secrets management, concurrency groups
- **Pitfalls**: YAML indent errors, missing `permissions:`, shell quoting in `run:` blocks

### 2. ECC-Specific GHA Rule

New rule at `rules/ecc/github-actions.md` with ECC-specific conventions:
- Release pipeline for `ecc` + `ecc-workflow` binaries (cross-platform matrix)
- CI pipeline running `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, `npm run lint`
- Artifact naming conventions, checksum generation
- Tag-based release triggers (v*)

### 3. Branch Isolation Hook

PreToolUse hook that blocks Write/Edit to `.github/workflows/` when the current git branch is `main`:
- Implemented in Rust via `ecc-hook` (consistent with existing hook infrastructure)
- Checks `git rev-parse --abbrev-ref HEAD` — if "main", block writes to `.github/workflows/**`
- Error message: "BLOCKED: Cannot edit GitHub Actions workflows on main. Create a feature branch first."
- Only `.github/workflows/` is blocked — other `.github/` files (CODEOWNERS, dependabot) are allowed on main

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | One item or split? | One item: skill + rule + hook together | Recommended |
| 2 | Skill content? | Both generic GHA patterns AND ECC-specific rule | User |
| 3 | Hook vs soft instruction? | Hook enforcement (PreToolUse blocks writes on main) | Recommended |
| 4 | Scope of hook? | Only .github/workflows/, not all .github/ | Recommended |
| 5 | Multi-session concurrency? | Handled by BL-065 worktree isolation — no additional mechanism needed | Recommended |

## Dependencies

- BL-088 depends on this (release pipeline patterns needed for `ecc update`)
- BL-065 Sub-Spec C (worktree isolation) resolves multi-session concurrency

## Priority

Should be implemented BEFORE BL-088 (`ecc update`) — the release pipeline patterns from this skill are needed to build the CI/CD that BL-088's prod mode depends on.

## Ready-to-Paste Prompt

```
/spec dev

Create a GitHub Actions skill + ECC-specific rule + branch isolation hook:

1. Generic `github-actions` skill: workflow patterns (CI/CD/cron), Rust cross-compilation
   matrix, gh CLI debugging, caching, secrets, pitfalls. At skills/github-actions/SKILL.md.

2. ECC-specific rule: release pipeline for ecc + ecc-workflow binaries, CI pipeline
   (cargo test/clippy/build + npm lint), tag-based release triggers.
   At rules/ecc/github-actions.md.

3. PreToolUse hook: block Write/Edit to .github/workflows/ when on main branch.
   Implemented in Rust via ecc-hook. Only workflows blocked, not other .github/ files.

This is a prerequisite for BL-088 (ecc update) release pipeline.
See BL-089 for full analysis.
```
