---
description: Report documentation coverage per module, diff against baselines, flag staleness and regressions.
---

# Documentation Coverage

Calculate documentation coverage metrics, compare against previous runs or branches, and flag stale or regressing documentation.

## What This Command Does

1. Calculate coverage % per module (documented exports / total exports)
2. Break down by item type (functions, classes, types, constants)
3. Compare against previous run or specified branch/commit
4. Flag regressions (coverage decreased)
5. Detect stale docs (code changed but docs didn't)

## Arguments

- `--base=<branch|commit>` — compare against a specific baseline
- `--module=<name>` — report on a specific module only

## Output

Writes coverage report to `docs/DOC-COVERAGE.md` (small codebase) or `docs/doc-coverage/` folder (large codebase).

Includes:
- Overall coverage % and grade
- Per-module coverage table with trend indicators
- Coverage by item type
- Staleness report (code-changed-since-doc-updated)
- Regressions since last run

## Prerequisites

Requires `docs/API-SURFACE.md` from a previous `/doc-analyze` run. If missing, suggests running `/doc-analyze` first.

## When to Use

- To check documentation health before a release
- In PR reviews to catch coverage regressions
- To track documentation improvement over time
- As part of the full `/doc-suite`

## Related

- Full suite: `/doc-suite`
- Prerequisite: `/doc-analyze`
- Agent: `agents/doc-reporter.md`
