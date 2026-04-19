# ADR-0069: Cartography noise filter architecture

- **Status**: Accepted
- **Date**: 2026-04-19

## Context

Cartography pipeline produced 80% noise before this change. See spec Problem Statement in `docs/specs/2026-04-19-reduce-cartography-memory-noise/spec.md`.

## Decision

1. Preserve `Stop` hook as the trigger (not switch to post-commit). Trigger change violates ADR-0037/0038; deferred to a future BL.
2. Dedupe window: `N=20` default (configurable via `ECC_CARTOGRAPHY_DEDUPE_WINDOW`). Covers ~2 weeks of sessions at current activity rate.
3. Flock fail-open: 500 ms timeout on `.dedupe.lock`; on timeout, write the delta (benign duplicate preferred over lost write). Local single-user context.
4. Tracing observability: `tracing::info!(target: "cartography::filter", ...)` on filter-skip; `tracing::debug!(target: "cartography::dedupe", dup_of = ...)` on dedupe-skip.

## Consequences

Positive: write-time filter reduces noise at the source. Fail-open keeps dedupe advisory.
Negative: benign duplicates are possible under lock contention; accepted for local tooling.

## References

- Spec: `docs/specs/2026-04-19-reduce-cartography-memory-noise/spec.md`
- Design: `docs/specs/2026-04-19-reduce-cartography-memory-noise/design.md`
- ADR-0037, ADR-0038 (hook trigger contracts)
