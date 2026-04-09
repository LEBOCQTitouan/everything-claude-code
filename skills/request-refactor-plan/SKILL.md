---
name: request-refactor-plan
description: Interview-driven refactoring plan with tiny-commit decomposition, dependency ordering, and per-commit risk assessment.
origin: ECC
---

# Request Refactor Plan

Decompose a refactoring into a sequence of tiny, safe commits through structured interview and codebase exploration.

## When to Activate

- User says "refactor plan" or "plan a refactoring"
- User says "how should I restructure" or "tiny commits for this change"
- User wants to decompose a large refactoring into safe steps

## Flow

Six steps, sequential. Use AskUserQuestion for each user input — one question at a time. If AskUserQuestion is unavailable, fall back to conversational questions.

### Step 1: Interview

Ask what needs to change and why. Probe for: current pain points, desired end state, constraints (backward compatibility, API stability, performance).

### Step 2: Codebase Exploration

Use Read, Grep, and Glob to understand current structure. Verify user's assertions. Map dependencies between modules that will be affected.

### Step 3: Identify Affected Files

List every file that will be touched. Group by module. Flag files with high test coverage vs untested areas.

### Step 4: Decompose into Tiny Commits

Break the refactoring into the smallest possible commits. Each commit MUST leave the codebase green (compiling + all tests passing). Prefer: rename → move → extract → inline → delete. Never mix behavioral changes with structural changes in one commit.

### Step 5: Order by Dependency

Sequence commits so each builds on the previous. No commit should depend on a future commit. Identify parallelizable commits (independent files/modules).

### Step 6: Write Plan

Write to `docs/refactors/{name}-plan.md` (kebab-case slug, max 40 chars). Create the directory automatically if missing. If a plan already exists at the path, ask whether to overwrite or append a revision.

## Per-Commit Template

Each commit in the plan uses this structure:

| Field | Content |
|-------|---------|
| Change Description | What this commit does (one sentence) |
| Affected Files | List of files modified |
| Risk Level | LOW (no behavior change), MEDIUM (behavior change, tested), HIGH (behavior change, untested) |
| Rollback | How to undo this commit safely |
