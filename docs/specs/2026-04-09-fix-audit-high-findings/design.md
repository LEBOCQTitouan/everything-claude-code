# Design: Fix All HIGH Findings from Full Audit (2026-04-09)

## Overview

Five user stories fix HIGH audit findings: deterministic prune test via Clock injection (US-001), six oversized file decompositions (US-002), LazyLock regex standardization + is_leap dedup (US-003), doc-comment coverage increase (US-004), and swallowed-error-to-tracing::warn conversion (US-005). All changes are internal refactors — no public API changes, no new dependencies.

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/time.rs` | Add `pub fn is_leap_year(y: u64) -> bool` | Single source of truth for 5 duplicate copies | US-003 AC-003.4 |
| 2 | `crates/ecc-infra/src/sqlite_bypass_store.rs` | Add `clock: Arc<dyn Clock>` field, inject via constructors, compute cutoff in Rust with parameterized SQL | Deterministic prune, eliminate format! SQL interpolation | US-001 AC-001.1/4/5 |
| 3 | `crates/ecc-cli/src/commands/bypass.rs` | Pass `SystemClock` to `SqliteBypassStore::new` (4 call sites) | Wire production clock into adapter | US-001 |
| 4 | `crates/ecc-app/src/validate/patterns.rs` → decompose | Extract: `frontmatter_validation.rs`, `cross_ref_validation.rs`, `section_validation.rs`, `code_block_scanning.rs`; keep `patterns.rs` as orchestrator with re-exports; move `#[cfg(test)]` block to `patterns_tests.rs` | 2,355 lines → 5 files each <800 | US-002 AC-002.1 |
| 5 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/tests_helpers.rs` → decompose | Split into: `start_test_helpers.rs`, `stop_test_helpers.rs`, `delta_test_helpers.rs`; original file becomes module root re-exporting | 2,009 lines → 3 files each <800 | US-002 AC-002.2 |
| 6 | `crates/ecc-app/src/update/orchestrator_tests.rs` → decompose | Split into: `happy_path_tests.rs`, `error_tests.rs`, `rollback_tests.rs`; original becomes module root | 962 lines → 3 files each <800 | US-002 AC-002.3 |
| 7 | `crates/ecc-infra/src/sqlite_memory.rs` → decompose | Split into: `memory_schema.rs`, `memory_queries.rs`, `memory_tests.rs`; original becomes module root with `pub use` re-exports | 907 lines → 3 files each <800 | US-002 AC-002.4 |
| 8 | `crates/ecc-app/src/hook/mod.rs` → decompose | Extract: `dispatch.rs` (dispatch fn + helpers), `bypass_handling.rs` (bypass token logic), `hook_tests.rs` (tests); keep `mod.rs` as thin re-export layer | 873 lines → 3 files + mod.rs each <800 | US-002 AC-002.5 |
| 9 | `crates/ecc-workflow/src/commands/merge.rs` → decompose | Extract `merge_tests.rs` module; keep production logic in `merge.rs` | 809 lines → 2 files each <800 | US-002 AC-002.6 |
| 10 | `crates/ecc-domain/src/audit_web/report_validation.rs` | Convert 3 `OnceLock<Regex>` to `LazyLock<Regex>` | Standardize regex init pattern | US-003 AC-003.2 |
| 11 | `crates/ecc-domain/src/audit_web/dimension.rs` | Convert 1 `OnceLock<Regex>` to `LazyLock<Regex>` | Standardize regex init pattern | US-003 AC-003.2 |
| 12 | `crates/ecc-domain/src/spec/ac.rs` | Convert 4 inline `Regex::new()` to `LazyLock<Regex>` (AC_ID_RE, AC_DEF_RE, MALFORMED_RE, FENCE_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 13 | `crates/ecc-domain/src/spec/pc.rs` | Convert 2 inline `Regex::new()` to `LazyLock<Regex>` (PC_ID_RE, SEPARATOR_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 14 | `crates/ecc-domain/src/spec/ordering.rs` | Convert 1 inline `Regex::new()` to `LazyLock<Regex>` (SEPARATOR_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 15 | `crates/ecc-domain/src/drift/mod.rs` | Convert 2 inline `Regex::new()` to `LazyLock<Regex>` (AC_ID_RE, PC_ID_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 16 | `crates/ecc-domain/src/memory/migration.rs` | Convert 1 inline `Regex::new()` to `LazyLock<Regex>` (BL_REF_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 17 | `crates/ecc-domain/src/docs/claude_md.rs` | Convert 1 inline `Regex::new()` to `LazyLock<Regex>` (CLAIM_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 18 | `crates/ecc-domain/src/detection/package_manager.rs` | Convert 2 inline `Regex::new()` to `LazyLock<Regex>` (SAFE_NAME_RE, SAFE_ARGS_RE) | Eliminate per-call compilation | US-003 AC-003.1 |
| 19 | `crates/ecc-domain/src/ansi.rs` | Already LazyLock — no change needed | Baseline confirmation | — |
| 20 | `crates/ecc-app/src/bypass_mgmt.rs` | Remove `#[allow(clippy::manual_is_multiple_of)]`, use `.is_multiple_of()`, replace `is_leap` call with `ecc_domain::time::is_leap_year` | Remove suppression + dedup | US-003 AC-003.3/4 |
| 21 | `crates/ecc-app/src/memory/consolidation.rs` | Replace `is_leap_year` with `ecc_domain::time::is_leap_year` | Dedup | US-003 AC-003.4 |
| 22 | `crates/ecc-infra/src/sqlite_cost_store.rs` | Replace `is_leap` with `ecc_domain::time::is_leap_year` | Dedup | US-003 AC-003.4 |
| 23 | `crates/ecc-infra/src/sqlite_metrics_store.rs` | Replace `is_leap` with `ecc_domain::time::is_leap_year` | Dedup | US-003 AC-003.4 |
| 24 | `crates/ecc-infra/src/sqlite_log_store.rs` | Replace `is_leap` with `ecc_domain::time::is_leap_year` | Dedup | US-003 AC-003.4 |
| 25 | `crates/ecc-domain/src/**/*.rs` (multiple) | Add `///` doc comments to public types, traits, functions | ecc-domain >50% coverage | US-004 AC-004.1 |
| 26 | `crates/ecc-app/src/**/*.rs` (multiple) | Add `///` doc comments to public types, traits, functions | ecc-app >30% coverage | US-004 AC-004.2 |
| 27 | `crates/ecc-infra/src/**/*.rs` (multiple) | Add `///` doc comments to public types, traits, functions | ecc-infra >30% coverage | US-004 AC-004.3 |
| 28 | `crates/ecc-ports/src/*.rs` | Add `///` doc comments to all port traits | All port traits documented | US-004 AC-004.4 |
| 29 | `crates/ecc-app/src/hook/mod.rs` (post-decomp: `dispatch.rs` or `bypass_handling.rs`) | Convert `let _ = store.record(&decision)` to `if let Err(e) = ...` + `tracing::warn!` | Observable bypass audit failure | US-005 AC-005.1/2 |
| 30 | `crates/ecc-app/src/drift_check.rs` | Convert 2 `let _ =` (L57, L59) to `if let Err(e) = ...` + `tracing::warn!` | Observable file op failure | US-005 AC-005.1/3 |
| 31 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | Convert `let _ = ports.fs.write(...)` (L76) to `if let Err(e) = ...` + `tracing::warn!` | Observable recovery write failure | US-005 AC-005.1/4 |
| 32 | `crates/ecc-workflow/src/commands/merge.rs` (post-decomp: prod part) | Convert `let _ = Command::new("git")` (L141) to `if let Err(e) = ...` + `tracing::warn!` | Observable rebase abort failure | US-005 AC-005.1/5 |
| 33 | `crates/ecc-app/src/hook/mod.rs` (post-decomp: dispatch part) L349 | Add `// Intentional fire-and-forget: metrics recording is best-effort` comment | Document deliberate discard | US-005 AC-005.6 |
| 34 | `crates/ecc-app/src/hook/handlers/tier2_tools/quality.rs` L205 | Add `// Intentional fire-and-forget: metrics recording is best-effort` comment | Document deliberate discard | US-005 AC-005.6 |
| 35 | Remaining ~10 production `let _ =` sites across ecc-app (install/global/steps.rs:232, dev/toggle.rs:69, backlog.rs:153, update/orchestrator.rs:156/184/217/218/236/246/267/271/330/339, update/swap.rs:29/160) | Convert to `if let Err(e) = ...` + `tracing::warn!` for error-significant ones; add comment for cleanup-only ones | Complete 15-site coverage | US-005 AC-005.1 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | Unit | Clock-injected prune test: fixed clock at epoch 1712400000 (2024-04-06T12:00:00Z), insert record at "2020-01-01T00:00:00Z", prune(1) deletes it, recent record survives | AC-001.2 | `cargo test -p ecc-infra bypass_prune` | PASS, deleted=1 |
| PC-002 | Unit | Prune equivalence: for identical data + identical clock, old SQL-datetime approach and new Rust-computed cutoff produce same deletion set | AC-001.3 | `cargo test -p ecc-infra prune_equivalence` | PASS |
| PC-003 | Lint | No format! interpolation in prune SQL — grep for `format!.*datetime` in sqlite_bypass_store.rs returns 0 matches | AC-001.4 | `grep -c "format!.*datetime" crates/ecc-infra/src/sqlite_bypass_store.rs` | 0 |
| PC-004 | Build | BypassStore trait signature unchanged — `cargo build -p ecc-ports` succeeds; trait has same 4 methods with same signatures | AC-001.5 | `cargo build -p ecc-ports` | PASS |
| PC-005 | Decomp | patterns.rs decomposed: each new file <800 lines, `cargo test -p ecc-app validate` passes | AC-002.1 | `cargo test -p ecc-app validate && wc -l crates/ecc-app/src/validate/patterns*.rs crates/ecc-app/src/validate/frontmatter_validation.rs crates/ecc-app/src/validate/cross_ref_validation.rs crates/ecc-app/src/validate/section_validation.rs crates/ecc-app/src/validate/code_block_scanning.rs` | All tests PASS, each file <800 lines |
| PC-006 | Decomp | tests_helpers.rs decomposed: each <800 lines, cartography tests pass | AC-002.2 | `cargo test -p ecc-app cartography && wc -l crates/ecc-app/src/hook/handlers/tier3_session/cartography/start_test_helpers.rs crates/ecc-app/src/hook/handlers/tier3_session/cartography/stop_test_helpers.rs crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_test_helpers.rs` | All tests PASS, each file <800 lines |
| PC-007 | Decomp | orchestrator_tests.rs decomposed: each <800 lines, update tests pass | AC-002.3 | `cargo test -p ecc-app update && wc -l crates/ecc-app/src/update/happy_path_tests.rs crates/ecc-app/src/update/error_tests.rs crates/ecc-app/src/update/rollback_tests.rs` | All tests PASS, each file <800 lines |
| PC-008 | Decomp | sqlite_memory.rs decomposed: each <800 lines, memory tests pass | AC-002.4 | `cargo test -p ecc-infra sqlite_memory && wc -l crates/ecc-infra/src/sqlite_memory/*.rs` | All tests PASS, each file <800 lines |
| PC-009 | Decomp | hook/mod.rs decomposed: each <800 lines, hook tests pass | AC-002.5 | `cargo test -p ecc-app hook && wc -l crates/ecc-app/src/hook/mod.rs crates/ecc-app/src/hook/dispatch.rs crates/ecc-app/src/hook/bypass_handling.rs crates/ecc-app/src/hook/hook_tests.rs` | All tests PASS, each file <800 lines |
| PC-010 | Decomp | merge.rs decomposed: each <800 lines, merge tests pass | AC-002.6 | `cargo test -p ecc-workflow merge && wc -l crates/ecc-workflow/src/commands/merge.rs crates/ecc-workflow/src/commands/merge_tests.rs` | All tests PASS, each file <800 lines |
| PC-011 | Build | All public items re-exported at original paths after all decompositions | AC-002.7 | `cargo build --workspace` | PASS (no import errors) |
| PC-012 | Grep | No inline Regex::new() outside LazyLock closures in targeted files (ac.rs, pc.rs, ordering.rs, drift/mod.rs, migration.rs, claude_md.rs, package_manager.rs) | AC-003.1 | `grep -rn 'Regex::new(' crates/ecc-domain/src/spec/ac.rs crates/ecc-domain/src/spec/pc.rs crates/ecc-domain/src/spec/ordering.rs crates/ecc-domain/src/drift/mod.rs crates/ecc-domain/src/memory/migration.rs crates/ecc-domain/src/docs/claude_md.rs crates/ecc-domain/src/detection/package_manager.rs \| grep -v 'LazyLock'` | 0 matches (all converted to LazyLock) |
| PC-013 | Grep | No `OnceLock<Regex>` in workspace | AC-003.2 | `grep -rn 'OnceLock.*Regex' crates/ --include='*.rs'` | 0 matches |
| PC-014 | Lint | clippy passes without `manual_is_multiple_of` suppression | AC-003.3 | `cargo clippy -- -D warnings` | PASS, 0 warnings |
| PC-015 | Grep | No duplicate `is_leap` functions outside ecc-domain | AC-003.4 | `grep -rn 'fn is_leap' crates/ --include='*.rs' \| grep -v ecc-domain \| grep -v '_test'` | 0 matches |
| PC-016 | Doc | ecc-domain doc coverage >50% | AC-004.1 | `ecc docs coverage --scope crates/ecc-domain/src` | >50% coverage |
| PC-017 | Doc | ecc-app doc coverage >30% | AC-004.2 | `ecc docs coverage --scope crates/ecc-app/src` | >30% coverage |
| PC-018 | Doc | ecc-infra doc coverage >30% | AC-004.3 | `ecc docs coverage --scope crates/ecc-infra/src` | >30% coverage |
| PC-019 | Doc | All port traits have doc comments | AC-004.4 | `for f in crates/ecc-ports/src/*.rs; do grep -B1 'pub trait' "$f" \| grep -q '///' || echo "MISSING: $f"; done` | No "MISSING" output (every pub trait has /// above it) |
| PC-020 | Grep | Swallowed errors converted: tracing::warn present at bypass_handling (hook_id), drift_check (path), session_merge (path), merge (rebase abort), install/steps (path) | AC-005.1-5 | `grep -rn 'tracing::warn' crates/ecc-app/src/hook/bypass_handling.rs crates/ecc-app/src/drift_check.rs crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs crates/ecc-workflow/src/commands/merge.rs crates/ecc-app/src/install/global/steps.rs` | 5+ matches (one per site minimum) |
| PC-021 | Grep | Fire-and-forget metrics sites have documenting comments | AC-005.6 | `grep -B1 'let _ = .*record_if_enabled' crates/ecc-app/src/hook/ crates/ecc-app/src/hook/handlers/tier2_tools/quality.rs --include='*.rs'` | Comment present above each |
| PC-022 | Grep | All new LazyLock<Regex> have .expect() — count LazyLock declarations equals count of .expect() calls in targeted files | AC-003.5 | `LAZY=$(grep -rc 'LazyLock.*Regex' crates/ecc-domain/src/spec/ac.rs crates/ecc-domain/src/spec/pc.rs crates/ecc-domain/src/spec/ordering.rs crates/ecc-domain/src/drift/mod.rs crates/ecc-domain/src/memory/migration.rs crates/ecc-domain/src/docs/claude_md.rs crates/ecc-domain/src/detection/package_manager.rs crates/ecc-domain/src/audit_web/report_validation.rs crates/ecc-domain/src/audit_web/dimension.rs \| awk -F: '{s+=$2}END{print s}') && EXPECT=$(grep -rc '.expect(' crates/ecc-domain/src/spec/ac.rs crates/ecc-domain/src/spec/pc.rs crates/ecc-domain/src/spec/ordering.rs crates/ecc-domain/src/drift/mod.rs crates/ecc-domain/src/memory/migration.rs crates/ecc-domain/src/docs/claude_md.rs crates/ecc-domain/src/detection/package_manager.rs crates/ecc-domain/src/audit_web/report_validation.rs crates/ecc-domain/src/audit_web/dimension.rs \| awk -F: '{s+=$2}END{print s}') && [ "$LAZY" -le "$EXPECT" ]` | exit 0 (expect count >= LazyLock count) |
| PC-023 | Grep | Positive assertion: tracing::warn count >= 15 across converted files | AC-005.1 | `WARNS=$(grep -rc 'tracing::warn' crates/ecc-app/src/drift_check.rs crates/ecc-app/src/hook/bypass_handling.rs crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs crates/ecc-workflow/src/commands/merge.rs crates/ecc-app/src/install/global/steps.rs crates/ecc-app/src/dev/toggle.rs crates/ecc-app/src/backlog.rs crates/ecc-app/src/update/orchestrator.rs crates/ecc-app/src/update/swap.rs crates/ecc-workflow/src/commands/merge_cleanup.rs 2>/dev/null \| awk -F: '{s+=$2}END{print s}') && [ "$WARNS" -ge 15 ]` | exit 0 (>= 15 tracing::warn calls) |
| PC-024 | Lint | CHANGELOG.md updated with audit fix entries | All | `grep -c 'audit' CHANGELOG.md` | >=1 match |
| PC-025 | Lint | CLAUDE.md test count updated | All | `grep 'cargo test' CLAUDE.md \| head -1` | Reflects actual test count |
| PC-026 | Full | Full test suite passes | All | `cargo test` | All tests PASS |
| PC-027 | Lint | Full clippy clean | All | `cargo clippy -- -D warnings` | 0 warnings |
| PC-028 | Build | Workspace builds | All | `cargo build` | PASS |

## TDD Ordering Rationale

The phases are ordered to respect dependency chains and minimize rework:

**Phase 1 (US-003 partial — is_leap extraction)** comes first because it creates the shared `is_leap_year` function in `ecc-domain::time` that Phases 3 (regex/dedup), Phase 2 (decomposition of files that contain `is_leap`), and Phase 5 (swallowed errors in files being decomposed) all depend on. Starting here means the domain layer stabilizes first.

**Phase 2 (US-001 — prune fix)** comes second because it is a contained change in one file (`sqlite_bypass_store.rs`) with no downstream dependencies. The Clock injection pattern is additive — it adds a field, changes constructors, and rewrites the prune SQL. This must land before decompositions because `bypass_mgmt.rs` references the store and will be modified for is_leap in Phase 3.

**Phase 3 (US-003 remainder — regex + clippy + is_leap wiring)** follows because it modifies files that Phase 4 (decomposition) will split. Converting inline `Regex::new()` to `LazyLock<Regex>` in `ac.rs`, `pc.rs`, `ordering.rs`, `drift/mod.rs`, etc. is simpler while files are still monolithic. Similarly, wiring all 5 `is_leap` call sites to `ecc_domain::time::is_leap_year` and removing the clippy suppression in `bypass_mgmt.rs` should happen before that file structure changes.

**Phase 4 (US-002 — file decompositions)** is the largest mechanical change. It comes after Phases 1-3 so that the content being split is already in its final form (no `OnceLock`, no duplicate `is_leap`, no format! SQL). Each decomposition is independent and can be committed atomically. The order within the phase — patterns.rs first (largest), then test helpers, orchestrator tests, sqlite_memory, hook/mod.rs, merge.rs — minimizes cross-file conflicts.

**Phase 5 (US-005 — swallowed errors)** depends on Phase 4 because three of the 15 `let _ =` sites are in files being decomposed: `hook/mod.rs:294`, `hook/mod.rs:349`, and `merge.rs:141`. Converting them after decomposition means editing the final, smaller target files rather than the originals. The domain constraint (no tracing in `ecc-domain`) is respected — the one `let _ =` in `sqlite_memory.rs:548` is a ROLLBACK cleanup, not an error-significant site.

**Phase 6 (US-004 — doc comments)** comes last because it touches files modified in all previous phases. Doc comments are additive and never change semantics, but working on final file structures avoids merge conflicts. The `ecc docs coverage` CLI command provides machine-verifiable thresholds.

## Phase Detail

### Phase 1: Extract shared is_leap_year to ecc-domain
Layers: [Entity]

Add `pub fn is_leap_year(y: u64) -> bool` to `crates/ecc-domain/src/time.rs`. Uses `.is_multiple_of()` (no clippy suppression needed). Unit test: known leap years (2000, 2024, 2400) and non-leap years (1900, 2023, 2100).

Commit cadence:
1. `test: add is_leap_year unit tests in ecc-domain::time`
2. `feat: add is_leap_year to ecc-domain::time`

### Phase 2: Fix prune test via Clock injection
Layers: [Adapter]

Modify `SqliteBypassStore`:
- Add `clock: Arc<dyn Clock>` field
- Update `new()` to accept `clock: Arc<dyn Clock>` parameter
- Update `in_memory()` to accept `clock: Arc<dyn Clock>` parameter
- Rewrite `prune()`: compute cutoff as ISO 8601 string in Rust using `clock.now_epoch_secs()` minus `older_than_days * 86400`, format to ISO 8601, use parameterized `DELETE ... WHERE timestamp < ?1`
- Update all 4 CLI call sites to pass `Arc::new(SystemClock)`
- Update all 4 test `in_memory()` calls to pass a `MockClock`

Commit cadence:
1. `test: add clock-injected prune and equivalence tests`
2. `feat: inject Clock into SqliteBypassStore, parameterize prune SQL`

### Phase 3: Standardize regex on LazyLock + wire is_leap + remove clippy suppression
Layers: [Entity, Adapter]

Three sub-changes committed separately:

**3a: OnceLock → LazyLock** (4 sites in ecc-domain):
- `report_validation.rs`: 3 `OnceLock<Regex>` → `static SCORE_RE/SECTION_RE/LINK_RE: LazyLock<Regex>`
- `dimension.rs`: 1 `OnceLock<Regex>` → `static RE: LazyLock<Regex>`

**3b: Inline Regex::new → LazyLock** (~13 sites):
- `spec/ac.rs`: 4 regex (AC_ID_RE, AC_DEF_RE, MALFORMED_RE, FENCE_RE)
- `spec/pc.rs`: 2 regex (PC_ID_RE, SEPARATOR_RE) — note: `SEPARATOR_RE` shared with `ordering.rs`, extract to `spec/mod.rs` or duplicate as both are 1-line
- `spec/ordering.rs`: 1 regex (SEPARATOR_RE)
- `drift/mod.rs`: 2 regex (AC_ID_RE, PC_ID_RE)
- `memory/migration.rs`: 1 regex (BL_REF_RE)
- `docs/claude_md.rs`: 1 regex (CLAIM_RE)
- `detection/package_manager.rs`: 2 regex (SAFE_NAME_RE, SAFE_ARGS_RE)
- Note: `memory/secrets.rs` has dynamic regex from function args — excluded per spec

**3c: is_leap dedup + clippy suppression removal**:
- `bypass_mgmt.rs`: remove `#[allow(clippy::manual_is_multiple_of)]`, delete local `is_leap`, call `ecc_domain::time::is_leap_year`
- `memory/consolidation.rs`: delete local `is_leap_year`, call `ecc_domain::time::is_leap_year`
- `sqlite_cost_store.rs`: delete local `is_leap`, call `ecc_domain::time::is_leap_year`
- `sqlite_metrics_store.rs`: delete local `is_leap`, call `ecc_domain::time::is_leap_year`
- `sqlite_log_store.rs`: delete local `is_leap`, call `ecc_domain::time::is_leap_year`

Commit cadence:
1. `refactor: convert OnceLock<Regex> to LazyLock<Regex> in ecc-domain`
2. `refactor: convert inline Regex::new to LazyLock<Regex> across workspace`
3. `refactor: deduplicate is_leap to ecc-domain::time::is_leap_year`

### Phase 4: Decompose oversized files
Layers: [Adapter, UseCase]

Six independent decompositions, each committed atomically:

**4a: patterns.rs (2,355 lines)**
Create `crates/ecc-app/src/validate/` submodules. Keep `patterns.rs` as the orchestrator with `pub(super) fn validate_patterns(...)` and `pub(super) fn generate_index(...)`. Extract internal validation functions into themed submodules. Move `#[cfg(test)] mod tests` to `patterns_tests.rs`.

**4b: tests_helpers.rs (2,009 lines)**
Convert `tests_helpers.rs` to a directory module. Create `start_test_helpers.rs`, `stop_test_helpers.rs`, `delta_test_helpers.rs`. Shared test fixtures (`make_ports`, helper fns) stay in the module root.

**4c: orchestrator_tests.rs (962 lines)**
Convert to directory module. Split by test category: `happy_path_tests.rs` (success flows), `error_tests.rs` (failure modes), `rollback_tests.rs` (swap + rollback scenarios).

**4d: sqlite_memory.rs (907 lines)**
Extract `memory_schema.rs` (schema init, migration, corruption recovery), `memory_queries.rs` (CRUD operations), `memory_tests.rs` (test module). Keep `SqliteMemoryStore` struct and `pub use` re-exports in root.

**4e: hook/mod.rs (873 lines)**
Extract `dispatch.rs` (dispatch fn, HookContext, HookResult, HookPorts, helpers), `bypass_handling.rs` (bypass token checking logic), `hook_tests.rs` (test module). Keep `mod.rs` as thin re-export layer with `pub use dispatch::*;`.

**4f: merge.rs (809 lines)**
Extract `merge_tests.rs` module containing `#[cfg(test)] mod tests`. Production code stays in `merge.rs`.

Commit cadence (one per decomposition):
1. `refactor: decompose validate/patterns.rs into submodules`
2. `refactor: decompose cartography tests_helpers.rs into submodules`
3. `refactor: decompose update/orchestrator_tests.rs into submodules`
4. `refactor: decompose infra/sqlite_memory.rs into submodules`
5. `refactor: decompose hook/mod.rs into submodules`
6. `refactor: decompose workflow/merge.rs tests into submodule`

### Phase 5: Convert swallowed errors to tracing::warn
Layers: [UseCase, Adapter]

Convert 15 error-significant production `let _ =` sites to `if let Err(e) = ... { tracing::warn!(...) }`. Add documenting comments on 2 fire-and-forget metric sites.

Specific sites (post-decomposition file paths):
1. `hook/bypass_handling.rs` (was mod.rs:294): `store.record(&decision)` → warn with hook_id, error
2. `drift_check.rs:57`: `fs.create_dir_all(parent)` → warn with path
3. `drift_check.rs:59`: `fs.write(report_path, ...)` → warn with path
4. `session_merge.rs:76`: `ports.fs.write(&recovery_path, ...)` → warn with path
5. `merge.rs:141`: `Command::new("git").args(["rebase", "--abort"])` → warn with error
6. `install/global/steps.rs:232`: `fs.write(agent_path, ...)` → warn with path
7. `dev/toggle.rs:69`: `fs.remove_file(&link)` → warn with path
8. `backlog.rs:153`: `fs.remove_file(&tmp_path)` → warn with path
9-15. `update/orchestrator.rs` sites: warn on cleanup failures (remove_dir_all, remove_file)

Fire-and-forget metric sites (comment only):
- `hook/dispatch.rs` (was mod.rs:349): add comment
- `quality.rs:205`: add comment

Commit cadence:
1. `fix: convert swallowed errors to tracing::warn in error-significant paths`
2. `docs: document intentional fire-and-forget metric discards`

### Phase 6: Doc-comment coverage increase
Layers: [Entity, UseCase, Adapter]

Add `///` doc comments to public types, traits, and functions across four crates. Target thresholds:
- ecc-domain: >50% (from 4.7%)
- ecc-app: >30% (from 2.7%)
- ecc-infra: >30% (from 2.1%)
- ecc-ports: all traits documented

Priority order: port traits first (highest visibility), then domain (most stable), then app/infra.

Commit cadence:
1. `docs: add doc comments to ecc-ports trait definitions`
2. `docs: add doc comments to ecc-domain public items`
3. `docs: add doc comments to ecc-app public items`
4. `docs: add doc comments to ecc-infra public items`

## E2E Assessment

- **Touches user-facing flows?** No — all changes are internal refactors (no CLI behavior change, no API change)
- **Crosses 3+ modules end-to-end?** No — each change is scoped to 1-2 crates
- **New E2E tests needed?** No — existing test suite covers all paths
- Existing `cargo test` (2,569+ tests) + `cargo clippy` serves as the gate

## Risks & Mitigations

- **Risk**: Re-export breakage after file decompositions — external crates importing from decomposed modules get compile errors
  - Mitigation: Every decomposition preserves `pub use` re-exports at the original module path. `cargo build --workspace` after each decomposition verifies no import breakage.

- **Risk**: Clock injection changes constructor signatures, breaking CLI call sites
  - Mitigation: Update all 4 `SqliteBypassStore::new()` call sites in `bypass.rs` and all 4 `in_memory()` test sites simultaneously. Compile check catches any missed sites.

- **Risk**: LazyLock regex conversion introduces panics on first access if regex pattern is wrong
  - Mitigation: All patterns are compile-time string literals already proven valid by existing tests. Each LazyLock keeps the `.expect("BUG: ...")` message convention from the session/manager.rs precedent.

- **Risk**: is_leap_year extraction breaks date calculations in stores
  - Mitigation: The function body is identical (`(y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)`) — only the location changes. Existing prune/cutoff tests in each store verify correctness.

## Success Criteria

- [ ] Clock-injected prune test is deterministic (PC-001, PC-002)
- [ ] No format! SQL interpolation in prune (PC-003)
- [ ] BypassStore trait signature unchanged (PC-004)
- [ ] All 6 files decomposed, each subfile <800 lines (PC-005 through PC-010)
- [ ] All re-exports preserved — workspace builds (PC-011)
- [ ] No inline static Regex::new() in production (PC-012)
- [ ] No OnceLock<Regex> in workspace (PC-013)
- [ ] Clippy clean, no manual_is_multiple_of suppression (PC-014)
- [ ] Single is_leap_year in ecc-domain only (PC-015)
- [ ] Doc coverage: domain >50%, app >30%, infra >30%, ports all traits (PC-016 through PC-019)
- [ ] 15 swallowed errors converted to tracing::warn (PC-020)
- [ ] Fire-and-forget metrics documented (PC-021)
- [ ] All LazyLock regex have .expect() (PC-022)
- [ ] No error-significant let _ = remaining (PC-023)
- [ ] CHANGELOG updated (PC-024)
- [ ] CLAUDE.md test count updated (PC-025)
- [ ] Full test suite passes (PC-026)
- [ ] Full clippy clean (PC-027)
- [ ] Workspace builds (PC-028)

## Rollback Plan

Reverse dependency order — if implementation fails, undo in this order:

1. **Phase 6 (doc comments)**: `git revert` doc comment commits — additive only, zero risk
2. **Phase 5 (swallowed errors)**: Revert tracing::warn conversions back to `let _ =` — must revert BEFORE Phase 4 rollback since file paths changed
3. **Phase 4 (decompositions)**: Revert all 6 decomposition commits in reverse order (merge → hook → sqlite_memory → orchestrator_tests → tests_helpers → patterns). Each revert restores the monolithic file.
4. **Phase 3 (regex/clippy/is_leap)**: Revert in reverse: 3c (is_leap wiring), 3b (inline→LazyLock), 3a (OnceLock→LazyLock)
5. **Phase 2 (prune fix)**: Atomic revert of SqliteBypassStore constructor change + all 4 CLI call sites + all 4 test sites
6. **Phase 1 (is_leap extraction)**: Delete `crates/ecc-domain/src/time.rs`, remove `pub mod time` from lib.rs

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 3 prescriptions (addressed) |
| Robert | CLEAN | 0 warnings |
| Security | PASS | 0 findings |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 85 | PASS | All 27 ACs covered by 28 PCs |
| Order | 80 | PASS | 6 phases in dependency order |
| Fragility | 70 | PASS | Grep PCs rewritten as positive assertions (R3) |
| Rollback | 75 | PASS | 6-phase reverse dependency order documented |
| Architecture | 85 | PASS | Clean hexagonal compliance, no boundary violations |
| Blast Radius | 80 | PASS | ~30 files across 4 crates, no public API change |
| Missing PCs | 78 | PASS | Added PC-022-028 in R1/R2 for expect, docs, gates |
| Doc Plan | 75 | PASS | CHANGELOG + CLAUDE.md test count, path fixed R2 |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | ecc-domain/src/time.rs | create | US-003 AC-003.4 |
| 2 | ecc-infra/src/sqlite_bypass_store.rs | modify | US-001 AC-001.1/4/5 |
| 3 | ecc-cli/src/commands/bypass.rs | modify | US-001 |
| 4-5 | ecc-domain/src/audit_web/*.rs | modify | US-003 AC-003.2 |
| 6-13 | ecc-domain/src/spec,drift,memory,docs,detection | modify | US-003 AC-003.1 |
| 14-18 | bypass_mgmt, consolidation, sqlite_*_store.rs | modify | US-003 AC-003.3/4 |
| 19-24 | 6 decomposition targets | decompose | US-002 AC-002.1-7 |
| 25-32 | swallowed error sites + fire-and-forget | modify | US-005 AC-005.1-6 |
| 33-36 | doc comments across 4 crates | modify | US-004 AC-004.1-4 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-09-fix-audit-high-findings/spec.md | Full spec |
| docs/specs/2026-04-09-fix-audit-high-findings/design.md | Full design |
| docs/specs/2026-04-09-fix-audit-high-findings/campaign.md | Grill-me decisions |
