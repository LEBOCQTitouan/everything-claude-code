# 0009. Phase Summary Convention

Date: 2026-03-22

## Status

Accepted

## Context

The spec-driven pipeline commands (`/spec-*`, `/design`, `/implement`) produced sparse bullet-point summaries at completion. Users could not see grill-me decisions, per-dimension adversary verdicts, artifact file paths, or commit inventories without scrolling through conversation history. A consistent, table-based summary convention was needed across all three pipeline phases.

## Decision

Each pipeline phase appends a `## Phase Summary` section to its persisted artifact file:
- `/spec-*` commands append to `docs/specs/YYYY-MM-DD-<slug>/spec.md`
- `/design` appends to `docs/specs/YYYY-MM-DD-<slug>/design.md`
- `/implement` appends to `docs/specs/YYYY-MM-DD-<slug>/tasks.md`

The Phase Summary uses pipe-delimited markdown tables (existing ECC convention). Each command defines its own set of tables appropriate to its phase. If `## Phase Summary` already exists in the target file, it is overwritten (idempotent).

## Consequences

- **Positive**: Users can review phase outcomes at a glance in both conversation and persisted files. Re-runs produce consistent, idempotent output.
- **Positive**: Table-based format enables future tooling to parse phase results programmatically.
- **Negative**: Persisted artifact files grow slightly larger with the appended summary section.
- **Negative**: Commands must track accumulator data (grill-me Q&A, commit SHAs) throughout their execution to populate summary tables.
