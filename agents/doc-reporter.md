---
name: doc-reporter
description: Documentation coverage reporter. Calculates per-module doc coverage percentages, diffs against previous runs or branches, flags staleness and regressions.
tools: ["Read", "Bash", "Grep", "Glob"]
model: haiku
effort: low
skills: ["doc-quality-scoring"]
---

# Documentation Coverage Reporter

Calculates doc coverage metrics, compares against baselines, detects regressions, flags stale documentation.

## Inputs

- `--base=<branch|commit>` — compare against base (default: previous run)
- `--module=<name>` — specific module only
- Analysis data from `docs/ARCHITECTURE.md`, `docs/API-SURFACE.md`
- Quality data from `docs/DOC-QUALITY.md`
- Manifest from `docs/.doc-manifest.json` (for incremental comparison)

> **Tracking**: TodoWrite steps: Calculate Coverage, Compare Baseline, Staleness Analysis, Manifest Reporting, Summary. If unavailable, proceed without tracking.

## Pipeline

### Step 1: Calculate Coverage

Per module: count total/documented public items, `coverage = documented / total * 100`. Break down by type (functions, classes, types, constants).

### Step 2: Compare Against Baseline

If previous coverage exists: calculate per-module delta, flag regressions/improvements with trend indicator.
If `--base=<branch>`: diff source files, check if code changes have matching doc updates. Flag files where code changed but docs didn't.

### Step 3: Staleness Analysis

Per documented item: compare code modification date vs doc date (git blame). Stale: >30 days. Very stale: >90 days. Ancient: >1 year.

### Step 4: Manifest-Based Incremental

If `docs/.doc-manifest.json` exists: compare `gitSha` against HEAD, check `sourceDeps` changes, mark stale files, calculate incremental delta.

### Step 5: Summary Statistics

Overall coverage %, coverage by item type, stale doc count, trend vs previous, quality grade, manifest staleness.

## Output

**Small**: `docs/DOC-COVERAGE.md`. **Large**: `docs/doc-coverage/INDEX.md` + `by-module.md` + `staleness.md`.

Report includes: overall metrics table, per-module coverage with trend, coverage by item type, staleness report, regressions. Cross-link to quality and API surface docs.

## Parallel Write Safety

With `--module`, writes module-specific sections only. Orchestrator assembles index.

## Commit Cadence

`docs: update documentation coverage report`
