---
description: Incrementally fix build and type errors with minimal, safe changes.
---

# Build and Fix

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

> **Tracking**: TodoWrite checklist. If unavailable, proceed without tracking.

## Step 0: Prompt Refinement

Analyze input via `prompt-optimizer` skill. Identify intent, check ambiguity, rewrite for clarity. Show refined prompt.

## Step 1: Detect Build System

| Indicator | Build Command |
|-----------|---------------|
| `package.json` | `npm run build` / `pnpm build` |
| `tsconfig.json` | `npx tsc --noEmit` |
| `Cargo.toml` | `cargo build 2>&1` |
| `pom.xml` | `mvn compile` |
| `build.gradle` | `./gradlew compileJava` |
| `go.mod` | `go build ./...` |
| `pyproject.toml` | `python -m py_compile` / `mypy .` |

## Step 1.5: Error Classification

| Type | Signal | Response |
|------|--------|----------|
| **Structural** | Multi-layer, import graph broken | Suggest `/spec refactor` |
| **Contractual** | Interface mismatch, wrong return type | Fix contract + note abstraction leak |
| **Incidental** | Typo, missing import | Fix immediately |

Structural errors recurring after 2 fixes → stop and rethink.

## Step 2: Parse and Group Errors

Run build, capture stderr. Group by file, sort by dependency order, count total.

## Step 3: Fix Loop (One at a Time)

Read file → diagnose → fix minimally → re-run build → verify → next.

## Step 4: Guardrails

Stop and ask if: fix introduces more errors, same error persists 3x, architectural change needed, missing dependencies.

## Step 5: Summary

Errors fixed, remaining, new (should be 0), next steps.

## Recovery Strategies

| Situation | Action |
|-----------|--------|
| Missing module | Check package installed; suggest install |
| Type mismatch | Read both types; fix narrower |
| Circular dep | Identify cycle; suggest extraction |
| Version conflict | Check version constraints |
| Build misconfiguration | Compare with defaults |
| Primitive obsession | If 3+ locations wrap/unwrap same concept, suggest newtype |

## Step 6: Commit Each Fix

Stage changed files, commit `fix: resolve <error>`, move to next. Never batch.
