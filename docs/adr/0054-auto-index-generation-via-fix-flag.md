# ADR-0054: Auto-Index Generation via --fix Flag

## Status

Accepted (2026-04-05)

## Context

The pattern library index (`patterns/index.md`) must stay in sync with the 136 pattern files. Manual maintenance is error-prone and breaks validation (every pattern must be listed in the index). Two approaches were considered: a separate `ecc patterns reindex` command vs a `--fix` flag on the existing `ecc validate patterns` command.

## Decision

Use `ecc validate patterns --fix` to auto-regenerate `patterns/index.md` from frontmatter data. The `--fix` flag only generates the index after validation passes (not on failure). This follows the precedent set by `ecc validate cartography --coverage`.

### Alternatives Considered

1. **Separate `ecc patterns reindex` command** — New top-level subcommand. Rejected: adds CLI surface bloat for a single-use operation that logically belongs with validation.
2. **`ecc validate patterns --fix` (chosen)** — Follows existing convention. Validation and index generation are naturally coupled — you validate first, then fix the index.
3. **Hook-based auto-generation** — Generate index on every commit. Rejected: too aggressive for a content operation.

## Consequences

- `--fix` flag added to `Patterns` CLI variant (struct variant with bool field)
- Index generated only after successful validation (safe ordering)
- Generated index includes: category headers, pattern links, language coverage table, tag list, total count
- No separate command needed — validates and fixes in one step
