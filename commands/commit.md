---
description: "Create a git commit with auto-generated conventional message, atomic-commit enforcement, and pre-flight gates."
allowed-tools: [Bash, Read, Grep, Glob, AskUserQuestion]
---

# Commit

> Narrate per `skills/narrative-conventions/SKILL.md`.

Create git commit with staging, conventional message, atomicity, and build/test pre-flight.

## Arguments

Optional commit message string. If omitted, auto-generated from diff.

## Phase 1: Workflow State Check

If `state.json` phase = `"implement"`: warn about active workflow, AskUserQuestion Proceed/Cancel.

## Phase 2: Working Tree Analysis

1. **Clean**: If nothing to commit, display message and stop.
2. **Merge conflicts**: Block with conflict list.
3. **Staging**: If already staged, respect. If not, propose from session history (fallback: git status). AskUserQuestion to confirm.
4. Run `git add` for confirmed files.

## Phase 3: Atomic Commit Enforcement

Analyze staged diff for concern separation. Multiple unrelated concerns → warn, suggest splitting. AskUserQuestion: single commit or split.

## Phase 4: Conventional Message

1. Type: feat/fix/refactor/docs/test/chore/perf/ci from diff analysis
2. Scope: infer from directory (omit if multi-directory)
3. Format: `<type>[(<scope>)]: <description>` — concise, <72 chars, imperative
4. Override: use `$ARGUMENTS` message if provided
5. AskUserQuestion: Accept/Edit/Cancel

## Phase 5: Build/Test Pre-Flight

Detect toolchain from `state.json` or project files. Run build then test. Failure blocks commit.

## Phase 6: Execute

`git commit -m "<message>"` (HEREDOC for body). Display hash + summary.

## Phase 7: Summary

> **Committed:** `<hash>` — `<message>` | **Files:** N | **Pre-flight:** build + tests pass

Stop. No push/PR.
