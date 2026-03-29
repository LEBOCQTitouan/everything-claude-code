# ADR-031: Deterministic Artifact Scaffolding

## Status

Accepted (2026-03-29)

## Context

tasks.md was generated manually by the LLM during `/implement` Phase 2, reading the design's PC table and constructing the file line by line. This was error-prone — the LLM could misformat entries, skip PCs, or use inconsistent separators.

## Decision

- **`ecc-workflow tasks init <design-path> --output <tasks-path>`** generates tasks.md deterministically from the design file's PC table.
- The renderer uses `→` (arrow) as the status trail separator, replacing the ambiguous `|` (pipe) used previously.
- The parser reads both old (`|`) and new (`→`) formats for backward compatibility.
- Post-TDD entries (E2E tests, Code review, Doc updates, Supplemental docs, Write implement-done.md) are always appended.
- Duplicate PC ID detection prevents scaffolding with invalid designs.
- `--force` flag allows overwriting an existing tasks.md (blocked by default).

## Consequences

- tasks.md format is deterministic and reproducible from the design file
- LLM no longer needs to parse and reconstruct the file manually
- Old tasks.md artifacts remain readable (backward compat via dual-separator parsing)
- The `→` separator eliminates ambiguity with error messages containing `|`
