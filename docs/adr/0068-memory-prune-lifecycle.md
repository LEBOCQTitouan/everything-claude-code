# ADR-0068: Memory prune lifecycle

- **Status**: Accepted
- **Date**: 2026-04-19

## Context

The ECC memory system writes per-BL project memories as `project_bl<N>_*.md` files to `~/.claude/projects/<hash>/memory/`. Prior to this ADR, no lifecycle hook tied memory retention to backlog status transitions, and stale memories persisted indefinitely after the backlog item shipped — polluting every subsequent LLM conversation that auto-loaded `MEMORY.md`.

Audit 2026-04-18 observed 6 of 6 project memories in the user's workspace describing shipped backlog items (BL-031, BL-064, BL-091, BL-092, BL-093, BL-131).

## Decision

1. **Lifecycle trigger**: on `backlog::update_status(BL-N, "implemented")` succeeding, fire a fire-and-forget prune hook that:
   a. Trashes matching `project_bl<N>_*.md` files to `<memory_root>/.trash/<YYYY-MM-DD>/`
   b. Removes corresponding rows from `MEMORY.md` via atomic temp+rename rewrite
   c. Deletes matching `MemoryStore` entries via `prune_by_backlog` use case
2. **Migration exemption**: `backlog::migrate_statuses` uses a separate code path that does NOT fire the hook — prevents mass deletion on one-shot BACKLOG.md sync.
3. **Recovery window**: 7-day trash retention via `trash_gc::gc_trash` — callable from CLI `ecc memory restore --trash <date>`.
4. **SafePath boundary**: path construction uses the `ecc_domain::memory::SafePath` pure newtype. The newtype performs string-prefix bounds-check only. Canonicalization happens at the app boundary in `memory::paths::resolve_project_memory_root`, preserving `ecc-domain`'s zero-I/O rule. This split closes three security findings at the type level: path traversal via crafted BL-IDs (SEC-001), `ECC_PROJECT_MEMORY_ROOT` escape outside `$HOME` (SEC-002), and CLI `--trash <date>` injection (SEC-003).
5. **CLI dry-run default**: `ecc memory prune --orphaned-backlogs` requires explicit `--apply` for destructive operation. Dry-run by default mirrors `git clean -n`. Restore CLI similarly.
6. **Canonical hashing via serde_jcs**: cartography content-hash uses RFC 8785 JSON Canonicalization Scheme (serde_jcs crate) to stabilize hashes across `serde_json` minor bumps (SEC-004 addressed).
7. **Event-driven lifecycle per AMV-L**: value-driven lifecycle management (backlog-status-driven prune) over age-based TTL alone. Inspired by AMV-L (arXiv 2603.04443) which demonstrates 3.1× throughput + 4.2× latency improvements versus pure TTL.

## Consequences

Positive:
- Stale memories no longer pollute LLM conversations.
- Prune is idempotent and recoverable (7-day trash).
- Boundary-typed paths prevent traversal at the type level.

Negative / trade-offs:
- Horizontal app coupling: `backlog` module calls `memory::file_prune` + `memory::lifecycle` directly. Single subscriber; a pub/sub port was deemed premature abstraction. If a second subscriber appears, extract a `PostTransitionHooks` port.
- Flock 500ms timeout is fail-open (benign duplicate preferred over lost write) — accepted trade-off documented in ADR-0069 (cartography).
- Migration flow bypasses the hook; users who expect mass-clean on migrate must run `ecc memory prune --orphaned-backlogs --apply` manually.

## References

- Spec: `docs/specs/2026-04-19-reduce-cartography-memory-noise/spec.md`
- Design: `docs/specs/2026-04-19-reduce-cartography-memory-noise/design.md`
- AMV-L: https://arxiv.org/abs/2603.04443
- RFC 8785 (JSON Canonicalization): https://tools.ietf.org/html/rfc8785
- State of AI Agent Memory 2026 (mem0.ai)
