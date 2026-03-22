---
description: Incrementally fix build and type errors with minimal, safe changes.
---

# Build and Fix

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each phase transition, tell the user what is happening and why.

Incrementally fix build and type errors with minimal, safe changes.

> **Tracking**: Create a TodoWrite checklist for the build-fix workflow. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Step 0: Prompt Refinement"
- "Step 1: Detect Build System"
- "Step 1.5: Error Classification"
- "Step 2: Parse and Group Errors"
- "Step 3: Fix Loop"
- "Step 4: Guardrails"
- "Step 5: Summary"
- "Step 6: Commit After Each Fix"

Mark each item complete as the step finishes.

## Step 0: Prompt Refinement

Before executing, analyze the user's input using the `prompt-optimizer` skill:
1. Identify intent and match to available ECC skills/commands/agents
2. Check for ambiguity or missing context
3. Rewrite the task description for clarity and specificity
4. Display the refined prompt to the user

If the refined prompt differs significantly, show both original and refined versions.
Proceed with the refined version unless the user objects.

## Step 1: Detect Build System

Identify the project's build tool and run the build:

| Indicator | Build Command |
|-----------|---------------|
| `package.json` with `build` script | `npm run build` or `pnpm build` |
| `tsconfig.json` (TypeScript only) | `npx tsc --noEmit` |
| `Cargo.toml` | `cargo build 2>&1` |
| `pom.xml` | `mvn compile` |
| `build.gradle` | `./gradlew compileJava` |
| `go.mod` | `go build ./...` |
| `pyproject.toml` | `python -m py_compile` or `mypy .` |

## Step 1.5: Error Classification

> After classifying errors, explain the classification to the user: how many are Structural, Contractual, or Incidental, and what each category means for the fix strategy.

Before fixing, classify each error to guide the response:

| Classification | Signal | Response |
|---------------|--------|----------|
| **Structural** | Error spans multiple layers, import graph broken | Suggest `/spec refactor` — this is an architecture problem, not a quick fix |
| **Contractual** | Interface mismatch, missing trait impl, wrong return type | Fix the contract + add a note about the abstraction leak that caused it |
| **Incidental** | Typo, missing import, wrong variable name | Fix immediately and move on |

Structural errors that recur after 2 fix attempts are a signal to stop and rethink the architecture.

## Step 2: Parse and Group Errors

1. Run the build command and capture stderr
2. Group errors by file path
3. Sort by dependency order (fix imports/types before logic errors)
4. Count total errors for progress tracking

## Step 3: Fix Loop (One Error at a Time)

For each error:

1. **Read the file** — Use Read tool to see error context (10 lines around the error)
2. **Diagnose** — Identify root cause (missing import, wrong type, syntax error)
3. **Fix minimally** — Use Edit tool for the smallest change that resolves the error
4. **Re-run build** — Verify the error is gone and no new errors introduced
5. **Move to next** — Continue with remaining errors

## Step 4: Guardrails

Stop and ask the user if:
- A fix introduces **more errors than it resolves**
- The **same error persists after 3 attempts** (likely a deeper issue)
- The fix requires **architectural changes** (not just a build fix)
- Build errors stem from **missing dependencies** (need `npm install`, `cargo add`, etc.)

## Step 5: Summary

Show results:
- Errors fixed (with file paths)
- Errors remaining (if any)
- New errors introduced (should be zero)
- Suggested next steps for unresolved issues

## Recovery Strategies

| Situation | Action |
|-----------|--------|
| Missing module/import | Check if package is installed; suggest install command |
| Type mismatch | Read both type definitions; fix the narrower type |
| Circular dependency | Identify cycle with import graph; suggest extraction |
| Version conflict | Check `package.json` / `Cargo.toml` for version constraints |
| Build tool misconfiguration | Read config file; compare with working defaults |
| Primitive obsession | When fixing type errors, if you see repeated primitive wrapping/unwrapping (`as String`, `.to_string()`, `&str` → `String` conversions for the same concept across 3+ locations), suggest introducing a newtype. Flag: "Consider a newtype for `<concept>` — repeated primitive conversions indicate a missing domain type." |

## Step 6: Commit After Each Fix

After each error is fixed and verified:
1. Stage only the files changed for this fix
2. Commit: `fix: resolve <error description>`
3. Move to the next error

Never batch multiple error fixes into one commit.

Fix one error at a time for safety. Prefer minimal diffs over refactoring.
