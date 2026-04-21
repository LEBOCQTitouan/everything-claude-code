# Spec: Reduce Cartography and Memory System Noise

## Problem Statement

The cartography and agent-memory subsystems produce high-noise, low-signal output. Cartography writes a `pending-delta-*.json` file on every session stop via `git diff --name-only HEAD`, with only a `.claude/` prefix filter — 127/158 historical deltas (2026-04-05 → 2026-04-18) contain zero code changes and 54/158 are self-referential tracking loops. The agent memory system (loaded into every LLM conversation via `MEMORY.md`) has no prune mechanism: every `project_bl<N>_*.md` memory file persists after the backlog item ships, with all 6 current entries now stale (BL-031, BL-064, BL-091, BL-092, BL-093, BL-131 — all `implemented`). A partial cartography fix on 2026-04-12 (commit `eb6fdfc1`) addressed self-reference but left `docs/` churn unfiltered. Root cause: both systems use session boundaries as logical-change boundaries (cartography) or write without corresponding cleanup triggers (memory). The fix expands the write-time filter, adds content-hash dedupe, introduces lifecycle-driven memory prune on backlog status transitions, and bundles adjacent hygiene fixes (`ERR-002`, `stop:daily-summary`, ~500 LOC of legacy dead code in the same file).

## Research Summary

- **Write-time / ingest-time filtering is the 2026 best practice** for telemetry pipelines (OpenTelemetry Collector's `Log Deduplication Processor`; SOC/SIEM shape-before-write pattern). Filter is most powerful before data reaches storage, not after.
- **LLM agent memory staleness is the #1 production failure mode**: "memories added months ago may contradict current facts; without TTL policies or manual eviction, stale memories inject false context into agent decisions" (State of AI Agent Memory 2026, mem0.ai).
- **AMV-L paper (arXiv 2603.04443)**: lifecycle-managed memory beats TTL alone by 3.1× throughput, 4.2× p50 latency. Value-driven lifecycle management (event-driven prune) > pure age-based eviction.
- **Consolidation as a recurring pattern**: periodically asking the agent to consolidate — merge duplicates, drop outdated facts, tighten descriptions — keeps the store lean (supermemory.ai, Spring AI memory tools 2026-04-07).
- **Post-commit triggers are preferred over session-boundary triggers for telemetry** because they align with logical-change boundaries (git-scm.com, Atlassian git-hooks). Deferred in this spec as a separate BL per architect recommendation — ADR-0037/0038 constraints.
- **Deduplication at aggregation layer** is a documented anti-pattern when write-time dedupe is possible; the latter saves storage and downstream compute.
- **Key pitfall**: switching from session-id keyed to commit-sha keyed artifacts breaks downstream consumers — doc-orchestrator assumes session IDs. Reason to stage.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Keep `Stop` hook trigger for cartography; add filter + hash dedupe only | Architect: trigger change contradicts ADR-0037/0038; 3-5× blast radius; doc-orchestrator consumer rework required | No (preserves existing ADRs) |
| 2 | Both memory systems in scope (file-based LLM memory + SQLite `MemoryStore`) | User decision; file-based is the 6-stale dataset, SQLite adds backlog-scoped deletion for future-proofing | Yes (new ADR: memory-prune-lifecycle) |
| 3 | Zero new port traits | Architect: `MemoryStore`, `FileSystem`, `Environment` cover all needs; direct app-layer call from backlog → memory | No |
| 4 | Bundle ERR-002 fix (23 `let _ =` suppressions) in same PR | Same file as filter change; mechanical conversion to `?` + `HookError` variant; already audit-flagged MEDIUM | No |
| 5 | Bundle `stop:daily-summary` filter using same `is_noise_path` predicate | Shares session-boundary anti-pattern; reusing domain predicate is trivially compatible | No |
| 6 | One-shot CLI `ecc memory prune --orphaned-backlogs` | Deterministic catch-up for the 6 pre-existing stale files, independent of future hook | No |
| 7 | Delete ~500 LOC legacy dead code in `delta_helpers.rs` (process_cartography, collect_pending_deltas, clean_corrupt_deltas) | Already marked `#[allow(dead_code)]`; reduces review surface; doc-orchestrator superseded the pipeline | No |
| 8 | Forward-only data policy; archived 158 deltas untouched | User decision; historical record preserved; rollback via `git revert` with zero data migration | No |
| 9 | Full-coverage test strategy + regression corpus (`tests/fixtures/cartography-corpus/`) | User decision; 10 representative delta samples committed so future filter changes must preserve behavior | No |
| 10 | Noise-path predicate lives in `ecc-domain` as pure function | Hexagonal: pure logic, no I/O; reusable by cartography + daily-summary + potential future session hooks | No |
| 11 | Dedupe window `N=20` (configurable via `ECC_CARTOGRAPHY_DEDUPE_WINDOW`) | Covers ~2 weeks of sessions at current activity rate (~10 sessions/week); hashing 20 small JSON files adds <5 ms per session-stop; configurable for future scale | No |
| 12 | Deleted memory files go to `<memory_root>/.trash/<YYYY-MM-DD>/` for 7 days before final removal, NOT `rm -f` | Recovery path for false-positive prunes; matches precedent from `sqlite_cost_store::prune` time-based retention; 7 days covers most "oh wait I need that back" windows | No |
| 13 | CLI `prune --orphaned-backlogs` defaults to dry-run; `--apply` required for destructive action | Destructive-by-default is a well-known antipattern; dry-run-default mirrors `rm -i` and `git clean -n` conventions | No |
| 14 | Concurrency via `ecc-flock` with 500 ms timeout, fail-open (write the delta if lock unavailable) | Benign duplicate preferred over lost write; matches `ecc-flock` usage pattern in backlog lock protocol | No |
| 15 | Filter skips emit `tracing::info!` with filtered-path list | Observability requirement: a silent filter is an invisible bug; info-level ensures skip events surface in normal logs without needing `ECC_VERBOSITY=debug` | No |

## User Stories

### US-001: Expand cartography write-time filter

**As a** developer using ECC, **I want** session-stop cartography to skip deltas containing only workflow or documentation churn, **so that** the `pending-delta-*.json` archive only reflects substantive code changes.

#### Acceptance Criteria

- AC-001.1: Given a session that modified only `.claude/workflow/state.json` and `.claude/workflow/implement-done.md`, when `stop_cartography` runs, then no delta file is written.
- AC-001.2: Given a session that modified only `docs/specs/<slug>/spec.md`, `docs/specs/<slug>/design.md`, or `docs/specs/<slug>/tasks.md`, when `stop_cartography` runs, then no delta file is written.
- AC-001.3: Given a session that modified only `docs/backlog/**` files, when `stop_cartography` runs, then no delta file is written.
- AC-001.4: Given a session that modified only `docs/cartography/**` files, when `stop_cartography` runs, then no delta file is written.
- AC-001.5: Given a session that modified `Cargo.lock` alone, when `stop_cartography` runs, then no delta file is written.
- AC-001.6: Given a session that modified `crates/ecc-domain/src/foo.rs` and `docs/specs/slug/spec.md`, when `stop_cartography` runs, then a delta is written containing only the crate file (`spec.md` filtered out).
- AC-001.7: The noise-path predicate lives in `ecc-domain` as a pure function `is_noise_path(path: &str) -> bool` with zero I/O dependencies.
- AC-001.8: Matching semantics are pinned as: ASCII-lowercase **prefix match** after normalizing path separators to `/`. The complete noise set is exactly these prefixes and exact matches (fail-open — anything NOT in this set is signal):
  - Prefixes: `.claude/workflow/`, `.claude/cartography/`, `.claude/worktrees/`, `docs/specs/`, `docs/backlog/`, `docs/cartography/`
  - Exact matches: `Cargo.lock`, `.claude/workflow` (without trailing slash, rare git edge case)
  - Cartography-emitted docs (`docs/cartography/**`) ARE classified as noise — self-ingestion of cartography output is not a signal.
- AC-001.9: Symlinks are classified by the symlink path only (no target resolution). `crates/foo/link.rs → .claude/workflow/state.json` is classified as `crates/...` (signal).
- AC-001.10: When all changed_files are classified as noise (i.e. no delta is written), the hook emits `tracing::info!(target: "cartography::filter", paths_skipped = N, skipped = ?paths_list)` listing all filtered paths. This provides audit trail for "why wasn't a delta written?".
- AC-001.11: When `git diff --name-only HEAD` returns an empty list (clean working tree), no delta is written and `tracing::debug!` logs the clean-tree skip reason.

#### Dependencies

- Depends on: none (foundational)

### US-002: Add content-hash dedupe for cartography deltas

**As a** developer, **I want** identical delta payloads to be written only once, **so that** repeated sessions with the same uncommitted state don't bloat the archive.

#### Acceptance Criteria

- AC-002.1: Given two sessions where the `changed_files` set is identical (same paths, same classifications, same project type), when `stop_cartography` runs on the second, then no new delta file is written and the reason is logged via `tracing::debug!`.
- AC-002.2: The hash function is SHA-256 of the canonical JSON form of `changed_files`, deterministic under key reordering.
- AC-002.3: Dedupe checks against the last N=20 pending + archived deltas by reading existing files through the `FileSystem` port (no new port).
- AC-002.4: Dedupe is disabled if `ECC_CARTOGRAPHY_DEDUPE=0` is set (opt-out escape hatch for testing).
- AC-002.5: Given `changed_files` is empty post-filter (all noise) OR empty from `git diff` (clean tree), when `stop_cartography` runs, then no delta file is written and no hash comparison is performed.
- AC-002.6: Concurrency contract — the dedupe read + write sequence uses `ecc-flock` around the `.claude/cartography/.dedupe.lock` file to prevent two simultaneously-ending sessions from both writing. If the lock cannot be acquired within 500 ms, dedupe is skipped and the delta is written (benign duplicate preferred over lost write).
- AC-002.7: Ordering of "last N=20" deltas is by filename lexicographic descending (pending-delta-*.json session IDs sort naturally: session-<timestamp>-<pid> format). Both `.claude/cartography/pending-delta-*.json` and `.claude/cartography/processed/pending-delta-*.json` are scanned.
- AC-002.8: The dedupe window size is configurable via `ECC_CARTOGRAPHY_DEDUPE_WINDOW` (default 20). Unit tests cover window=0 (disabled), window=1 (adjacent duplicate only), and window=100 (deep history).

#### Dependencies

- Depends on: US-001 (shared domain module location)

### US-003: Apply noise-path filter to `stop:daily-summary`

**As a** developer, **I want** the daily summary hook to skip sessions with no substantive code change, **so that** `docs/memory/action-log.json` reflects actual work sessions.

#### Acceptance Criteria

- AC-003.1: Given a session that modified only noise-paths (per `is_noise_path`), when `stop_daily_summary` runs, then no daily entry is appended.
- AC-003.2: Given a session that modified at least one non-noise path, when `stop_daily_summary` runs, then a daily entry IS appended (existing behavior preserved).
- AC-003.3: The hook reuses the same `is_noise_path` predicate from US-001 (no duplicated logic).

#### Dependencies

- Depends on: US-001

### US-004: Fix ERR-002 swallowed errors in `delta_helpers.rs`

**As a** maintainer, **I want** filesystem errors in the cartography pipeline to be logged rather than silently discarded, **so that** downstream failures have diagnostic trails.

#### Acceptance Criteria

- AC-004.1: All 23 `let _ = ...` call sites for `fs.create_dir_all`, `fs.write`, `fs.rename`, `fs.remove_file`, and `shell.run_command_in_dir` in `delta_helpers.rs` are replaced with either `?` or `if let Err(e) = ... { tracing::warn!(...) }`.
- AC-004.2: A new `HookError::CartographyIo { operation: String, path: PathBuf, source: IoError }` variant is added.
- AC-004.3: Existing cartography tests still pass after the conversion.
- AC-004.4: `cargo clippy -- -D warnings` is clean on the edited file.

#### Dependencies

- Depends on: none (independent of US-001)

### US-005: Remove legacy dead code from `delta_helpers.rs`

**As a** maintainer, **I want** the superseded `process_cartography`, `collect_pending_deltas`, and `clean_corrupt_deltas` functions removed, **so that** the cartography module has a single clear pipeline.

#### Acceptance Criteria

- AC-005.1: `#[allow(dead_code, unused_imports)]` at `delta_helpers.rs:9` is removed along with all gated code (~500 LOC).
- AC-005.2: No other module in the workspace imports the removed functions (verified via `cargo check`).
- AC-005.3: Test file sizes for `delta_helpers.rs` test module shrink accordingly; no orphaned tests remain.

#### Dependencies

- Depends on: US-004 (same-file touch; sequence after error-handling conversion)

### US-006: File-based memory prune hook on backlog `implemented` transition

**As a** user of ECC, **I want** `project_bl<N>_*.md` memory files to be deleted when BL-N transitions to `implemented`, **so that** stale memories don't pollute future LLM conversations.

#### Acceptance Criteria

- AC-006.0: The project-memory root directory is resolved via a new `memory::lifecycle::resolve_project_memory_root(env: &dyn Environment) -> PathBuf` function. Resolution order: (1) `ECC_PROJECT_MEMORY_ROOT` env var if set; (2) `$HOME/.claude/projects/<project-hash>/memory/` where `<project-hash>` is derived by the same algorithm currently used in `crates/ecc-workflow/src/commands/memory_write.rs::resolve_project_memory_dir` (path-based SHA-256). The algorithm is documented in the new ADR-0068 with test vectors. Test harnesses inject a fake `Environment` to redirect the root.
- AC-006.1: When `update_entry_status(BL-N, "implemented")` succeeds in `ecc-app/src/backlog.rs`, a post-hook scans the resolved project-memory root for files matching the pattern `^project_bl0*<N>(_[a-z0-9_-]+)?\.md$` (case-insensitive) and moves them to `<root>/.trash/<YYYY-MM-DD>/`. Trash entries older than 7 days are garbage-collected on hook re-entry.
- AC-006.2: The hook also scans `MEMORY.md` index and removes rows referencing the trashed files (atomic write via `mktemp + mv`).
- AC-006.3: Deletion/trash failures are logged via `tracing::warn!` with operation context (`target: "memory::prune"`) but do not fail the status transition (fire-and-forget).
- AC-006.4: The hook is gated on transition TO `implemented` only. Testability framing: running `ecc backlog migrate` on a BACKLOG.md containing 10 `implemented` entries does NOT delete any memory files (verified by integration test with seeded memory directory).
- AC-006.5: A one-shot CLI `ecc memory prune --orphaned-backlogs` scans all `project_bl<N>_*.md` files and moves those whose BL-N is `implemented` per BACKLOG.md into the trash dir.
- AC-006.6: The CLI defaults to `--dry-run` behavior. Actual trash operation requires `--apply`. (This is safer than dry-run-as-opt-in for a destructive operation.)
- AC-006.7: The prune hook is idempotent — running it twice for the same BL-N is a no-op on the second run (files already trashed).
- AC-006.8: BL-ID collision handling — regex uses left-anchored zero-padded numeric match. `project_bl031_*.md` matches BL-031 only, NOT BL-3 or BL-310. Test cases: `project_bl10_foo.md` does NOT match BL-100; `project_bl100_foo.md` does NOT match BL-10. Collision test in corpus.
- AC-006.9: A recovery path is documented: `ecc memory restore --trash <YYYY-MM-DD>` lists and optionally restores trashed files within the 7-day window.

#### Dependencies

- Depends on: none (parallel with cartography track)

### US-007: Backlog-scoped prune on SQLite-tiered `MemoryStore`

**As a** user of the three-tier memory system (BL-093), **I want** memory entries tagged with a BL identifier to be deletable when that BL ships, **so that** the tiered memory store stays aligned with the backlog lifecycle.

#### Acceptance Criteria

- AC-007.1: A new app-layer use case `memory::lifecycle::prune_by_backlog(store: &dyn MemoryStore, backlog_id: &str) -> Result<u32>` returns the count of deleted entries.
- AC-007.2: Implementation uses existing `MemoryStore::get_by_source_path` + `delete` in a loop (no new port method).
- AC-007.3: The use case is called from the same backlog-status-transition hook added in US-006 (single hook, two targets: file system + SQLite).
- AC-007.4: If the SQLite store is empty or has no matching entries, the call is a no-op and returns `Ok(0)`.
- AC-007.5: Integration test spins up an in-memory `MemoryStore`, seeds 3 entries tagged BL-050 + 2 tagged BL-051, calls `prune_by_backlog(store, "BL-050")`, asserts return = 3 and only BL-051 entries remain.

#### Dependencies

- Depends on: US-006 (shared hook invocation site)

### US-008: Regression corpus for cartography filter

**As a** maintainer, **I want** 10 representative delta samples committed as test fixtures, **so that** future filter changes cannot silently regress the noise ratio.

#### Acceptance Criteria

- AC-008.1: `tests/fixtures/cartography-corpus/` contains 10 JSON files (5 noise-only, 3 mixed, 2 pure-code) with expected-outcome metadata in a sidecar `expected.yaml`.
- AC-008.2: A corpus test in `crates/ecc-app/tests/cartography_corpus.rs` reads each fixture, runs the current filter logic, and asserts the outcome matches `expected.yaml`.
- AC-008.3: The corpus test runs on every `cargo test` invocation (not gated behind a feature flag).
- AC-008.4: Documentation in `tests/fixtures/cartography-corpus/README.md` explains how to add new fixtures and what "noise" vs "signal" means.

#### Dependencies

- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|------------------|
| `ecc-domain/src/cartography/noise_filter.rs` (NEW) | Domain | Pure function `is_noise_path(&str) -> bool`; table-driven test suite |
| `ecc-domain/src/cartography/dedupe.rs` (NEW) | Domain | Pure function `canonical_hash(&ChangedFiles) -> String` (SHA-256) |
| `ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs` | App | Replace `.claude/` prefix check at line 58 with domain predicate; inject dedupe check before write |
| `ecc-app/src/hook/handlers/tier3_session/cartography/delta_helpers.rs` | App | ERR-002 conversion (23 sites) + delete ~500 LOC legacy dead code |
| `ecc-app/src/hook/handlers/tier3_session/daily.rs` | App | Call `is_noise_path` before appending daily entry |
| `ecc-app/src/hook/errors.rs` | App | Add `HookError::CartographyIo` variant |
| `ecc-app/src/backlog.rs` | App | Hook `update_status` → memory-prune call (file-based + SQLite), gated on TO-`implemented` |
| `ecc-app/src/memory/lifecycle.rs` | App | Add `prune_by_backlog` (SQLite) + `prune_orphaned_file_memories` (file-based) use cases |
| `ecc-cli/src/commands/memory.rs` | CLI | Add `prune --orphaned-backlogs` subcommand with `--dry-run` flag |
| `tests/fixtures/cartography-corpus/` (NEW) | Tests | 10 JSON fixtures + `expected.yaml` + README |
| `crates/ecc-app/tests/cartography_corpus.rs` (NEW) | Tests | Corpus runner |
| `docs/adr/0068-memory-prune-lifecycle.md` (NEW) | Docs | New ADR per Decision #2 |

## Constraints

- **ADR-0037 (two-phase cartography)** and **ADR-0038 (session-scoped delta files)** preserved — trigger remains `Stop`, keying remains `session_id`.
- **No new port traits** — reuse `FileSystem`, `Environment`, `MemoryStore`.
- **No change to delta JSON schema** — existing 158 archived deltas remain valid without transformation.
- **Existing `.claude/` filter test** (`filters_out_dot_claude_paths`) extends but does not break.
- **`ecc-domain` zero I/O** — noise filter + hash are pure functions.
- **Fire-and-forget hook** — memory prune must not fail backlog status transitions.
- **Worktree-scoped** — all writes stay in the active worktree per write-guard.

## Non-Requirements

- Switching cartography trigger from `Stop` to `PostToolUse:git commit` — deferred to a new backlog entry (contradicts ADR-0037/0038, 3-5× blast radius, doc-orchestrator consumer rework required).
- Refining `classify_file` in `ecc-domain` to sub-bucket `.claude/workflow`, `.claude/cartography` separately — architect recommended deferring; would break 7 existing tests for marginal benefit.
- Adding a pub/sub port for backlog-lifecycle events — single subscriber, in-process; premature abstraction per architect.
- Migrating or reclassifying the 158 archived deltas — forward-only policy.
- Touching the `stop:craft-velocity` hook — out of scope; different responsibility surface.
- Building a promote-on-validate tier for memory (AMV-L full pattern) — research-grade; value-driven lifecycle is enough for this fix.
- Consolidation pass on the existing file-based memories — superseded by orphaned-backlogs CLI.

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `FileSystem` (ecc-ports) | Consumer added (hash read for dedupe) | Existing adapters unchanged; 1 new read call path per session-stop |
| `MemoryStore` (ecc-ports) | New consumer (`prune_by_backlog`) | Uses existing `get_by_source_path` + `delete`; no adapter change |
| `Environment` (ecc-ports) | Consumer added (`ECC_CARTOGRAPHY_DEDUPE` opt-out) | New env var read at `stop_cartography` entry |
| `~/.claude/projects/<hash>/memory/` (direct FS) | New deletion site | Non-port FS access (symmetric to existing LLM write site); documented in new ADR |
| Backlog status transition (`update_status`) | Side-effect added (memory prune) | Non-fatal; fire-and-forget; integration test required |
| `stop:daily-summary` behavior | Filtering added | Sessions with only noise-paths no longer append to `action-log.json`; existing daily-report consumers see fewer entries |
| `.claude/cartography/.dedupe.lock` (NEW) | Lock file | New file for concurrency guard; created/released by `ecc-flock` per session |
| `<memory_root>/.trash/<YYYY-MM-DD>/` (NEW) | Recovery dir | New directory for prune recovery; GC'd on hook re-entry after 7 days |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New noise-filter behavior | HIGH | `docs/cartography/elements/cartography-system.md` | Update "File Classification Rules" + add "Write-Time Noise Filter" section |
| New ADR | HIGH | `docs/adr/0068-memory-prune-lifecycle.md` | Create (document both file + SQLite prune decisions, reference AMV-L 2603.04443) |
| New CLI subcommand | MEDIUM | `docs/commands-reference.md` + `CLAUDE.md` | Document `ecc memory prune --orphaned-backlogs --dry-run` |
| CHANGELOG entry | MEDIUM | `CHANGELOG.md` | Add Changed/Removed/Fixed entries under Unreleased |
| Hook behavior change | MEDIUM | `CLAUDE.md` Gotchas section | Note `stop:daily-summary` now filters noise-only sessions |
| Legacy removal | LOW | Release notes | Note removal of ~500 LOC legacy cartography pipeline |
| Corpus README | LOW | `tests/fixtures/cartography-corpus/README.md` | Create |

## Rollback & Recovery

- **Cartography filter false-positive** (legitimate code change silently dropped): detected via `tracing::info!(target: "cartography::filter")` logs on every skip; user can inspect with `ecc log --target cartography::filter --since 1h`. Recovery: revert the filter commit; the 158-delta archive is unaffected.
- **Dedupe false-positive** (two legitimately different changesets hash identical — near-impossible with SHA-256 on canonical JSON): disable via `ECC_CARTOGRAPHY_DEDUPE=0`.
- **Memory prune false-positive** (BL-N wrongly marked `implemented`): recovery via `ecc memory restore --trash <YYYY-MM-DD>` within 7 days, OR manual `mv <trash>/project_blN_*.md <memory_root>/`.
- **Over-aggressive daily-summary filter**: reverts to old behavior by setting `ECC_DAILY_SUMMARY_NOISE_FILTER=0` (feature flag to be added in US-003).

## Observability

- `tracing::info!(target: "cartography::filter", paths_skipped = N, skipped = ?paths)` on every skipped delta write.
- `tracing::debug!(target: "cartography::dedupe", dup_of = <session_id>)` on every deduped write.
- `tracing::warn!(target: "memory::prune", bl_id, path, error = ?e)` on prune failure.
- `tracing::info!(target: "memory::prune", bl_id, trashed_count)` on successful prune.
- New counter surfaced by `ecc status --json`: `cartography.skipped_deltas_24h`, `memory.pruned_files_24h`.

## Open Questions

None after grill-me and adversarial review.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Root cause vs symptom | Both systems share "over-eager writes + under-pruned reads"; cartography root = session boundary ≠ logical-change boundary (defer trigger fix); memory root = no lifecycle event ties prune to backlog status | Recommended |
| 2 | Which memory system is in scope for the prune fix? | Both systems: file-based agent memory + SQLite-tiered MemoryStore | User |
| 3 | For cartography, fix symptom or root cause? | Staged: filter+dedupe now in this spec, trigger change deferred to a new BL | User |
| 4 | Which adjacent concerns do we bundle vs defer? | All four bundled: ERR-002 fix + stop:daily-summary filter + one-shot memory prune CLI + legacy dead-code cleanup (~500 LOC) | User |
| 5 | How should we validate the filter/dedupe works? | Full coverage + regression corpus: ~25 new tests + 10 fixture files committed under `tests/fixtures/cartography-corpus/` | User |
| 6 | What's the policy for archived deltas, stale memory files, and rollback path? | Forward-only + explicit one-shot cleanup: archived 158 deltas untouched (historical); run `ecc memory prune --orphaned-backlogs` once to trash the 6 stale files; rollback = `git revert` | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Expand cartography write-time filter | 11 | none |
| US-002 | Add content-hash dedupe for cartography deltas | 8 | US-001 |
| US-003 | Apply noise-path filter to stop:daily-summary | 3 | US-001 |
| US-004 | Fix ERR-002 swallowed errors in delta_helpers.rs | 4 | none |
| US-005 | Remove legacy dead code from delta_helpers.rs | 3 | US-004 |
| US-006 | File-based memory prune hook on backlog `implemented` | 10 | none |
| US-007 | Backlog-scoped prune on SQLite-tiered MemoryStore | 5 | US-006 |
| US-008 | Regression corpus for cartography filter | 4 | US-001, US-002 |
| **Total** | **8 stories** | **48 ACs** | |

### Acceptance Criteria (summary)

| AC ID | Description | Source US |
|-------|-------------|-----------|
| AC-001.1–7 | Filter drops `.claude/workflow/`, `docs/specs/`, `docs/backlog/`, `docs/cartography/`, `Cargo.lock`; pure domain predicate | US-001 |
| AC-001.8–11 | Formal match semantics (ASCII-lowercase prefix), symlink policy, filter-skip observability, empty git-diff handling | US-001 |
| AC-002.1–4 | SHA-256 canonical-JSON hash dedupe against last N=20; opt-out env var | US-002 |
| AC-002.5–8 | Empty-post-filter handling, `ecc-flock` concurrency + fail-open, lexicographic ordering, configurable window | US-002 |
| AC-003.1–3 | Daily-summary reuses `is_noise_path`; skip noise-only sessions | US-003 |
| AC-004.1–4 | 23 `let _ =` suppressions replaced; `HookError::CartographyIo` variant; clippy clean | US-004 |
| AC-005.1–3 | ~500 LOC dead-code deleted; no workspace imports; orphaned tests removed | US-005 |
| AC-006.0–9 | Env-injectable memory root; trash-dir with 7-day retention; dry-run default; BL-ID regex with collision safety; idempotent; restore CLI | US-006 |
| AC-007.1–5 | `prune_by_backlog` use case reuses existing `MemoryStore` port methods; integration test | US-007 |
| AC-008.1–4 | 10 fixtures + `expected.yaml` + README; CI-gated | US-008 |

### Adversary Findings

| Dimension | Round 1 | Round 2 | Key Rationale |
|-----------|---------|---------|---------------|
| Ambiguity | 55 | 88 | Noise-set exhaustively pinned (AC-001.8); symlink policy; BL-ID regex with collision tests |
| Edge Cases | 45 | 90 | Empty diff, empty post-filter, `ecc-flock` race, window=0/1/100, BL-10 vs BL-100 all covered |
| Scope | 72 | 85 | Non-Requirements explicit; bundled scope justified by shared files + shared predicate |
| Dependencies | 80 | 90 | US graph acyclic; new ADR-0068 referenced |
| Testability | 48 | 88 | AC-006.4 reframed as behavior (migrate-doesn't-delete) not implementation; env-injected root |
| Decisions | 65 | 90 | Decisions #11–15 added: N=20 rationale, trash dir, dry-run default, flock, tracing |
| Rollback & Failure | 40 | 88 | New Rollback & Recovery section + Observability section; named recovery commands for all 4 failure modes |
| **Average** | **57.9** | **88.4** | **Verdict: PASS** |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-19-reduce-cartography-memory-noise/spec.md` | Full spec (all 10 sections + Phase Summary) |
| `docs/specs/2026-04-19-reduce-cartography-memory-noise/campaign.md` | 6 grill-me decisions |
| `<git-dir>/ecc-workflow/state.json` | Phase: solution; spec_path persisted |
