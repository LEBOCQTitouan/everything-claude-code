# Solution: Cost and Token Tracking (BL-096)

## Spec Reference
Concern: dev, Feature: Add cost and token tracking to ECC - BL-096

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/cost/mod.rs` | Create | Module root with re-exports | US-001 |
| 2 | `crates/ecc-domain/src/cost/value_objects.rs` | Create | ModelId, TokenCount, Money, CostRate, RecordId newtypes with validated construction | AC-001.1 |
| 3 | `crates/ecc-domain/src/cost/record.rs` | Create | TokenUsageRecord entity with all fields | AC-001.2 |
| 4 | `crates/ecc-domain/src/cost/calculator.rs` | Create | PricingTable (injectable) + CostCalculator (estimate, summarize) + CostSummary/CostBreakdown | AC-001.3, AC-001.4, AC-001.5 |
| 5 | `crates/ecc-domain/src/cost/error.rs` | Create | CostError enum: InvalidModelId, InvalidMoney | AC-001.1 |
| 6 | `crates/ecc-domain/src/lib.rs` | Modify | Add `pub mod cost;` | US-001 |
| 7 | `crates/ecc-ports/src/cost_store.rs` | Create | CostStore trait (append, query, summary, prune, export), CostQuery, CostStoreError, ExportFormat | AC-002.1 |
| 8 | `crates/ecc-ports/src/lib.rs` | Modify | Add `pub mod cost_store;` | US-002 |
| 9 | `crates/ecc-infra/src/cost_schema.rs` | Create | ensure_schema(): token_usage table, indexes, PRAGMA journal_mode=WAL, unique constraint (timestamp, session_id, model) | AC-002.2, AC-005.3 |
| 10 | `crates/ecc-infra/src/sqlite_cost_store.rs` | Create | SqliteCostStore implementing CostStore, Mutex<Connection>, WAL mode | AC-002.2, AC-002.3, AC-002.4 |
| 11 | `crates/ecc-infra/src/lib.rs` | Modify | Add cost modules | US-002 |
| 12 | `crates/ecc-test-support/src/in_memory_cost_store.rs` | Create | InMemoryCostStore implementing CostStore for testing | AC-002.5 |
| 13 | `crates/ecc-test-support/src/lib.rs` | Modify | Add `pub mod in_memory_cost_store;` | AC-002.5 |
| 14 | `crates/ecc-app/src/cost_mgmt.rs` | Create | Use-case functions: summary, breakdown, compare, export, prune, migrate | US-004, US-005 |
| 15 | `crates/ecc-app/src/lib.rs` | Modify | Add `pub mod cost_mgmt;` | US-004 |
| 16 | `crates/ecc-app/src/hook/mod.rs` | Modify | Add `cost_store: Option<&'a dyn CostStore>` to HookPorts. **Blast radius**: all `make_ports()` test helpers in `mod.rs` and `helpers.rs` test modules (8+ tests) must add `cost_store: None` | AC-003.5 |
| 17 | `crates/ecc-app/src/hook/handlers/tier3_session/tracking.rs` | Modify | Refactor cost_tracker: CostCalculator, CostStore::append, agent_type, thinking_tokens, JSONL fallback | AC-003.1..3.4, AC-003.6, AC-003.7 |
| 18 | `crates/ecc-app/src/hook/handlers/tier3_session/helpers.rs` | Modify | Remove estimate_cost() (moved to domain CostCalculator) | AC-003.6 |
| 19 | `crates/ecc-cli/src/commands/cost.rs` | Create | CostArgs, CostAction enum, run() wiring SqliteCostStore to cost_mgmt | AC-004.1..4.6, AC-005.1..5.4 |
| 20 | `crates/ecc-cli/src/commands/mod.rs` | Modify | Add `pub mod cost;` | US-004 |
| 21 | `crates/ecc-cli/src/main.rs` | Modify | Add Cost variant to Subcommand enum | US-004 |
| 22 | `agents/audit-orchestrator.md` | Modify | Add cost reporting instructions | AC-006.1, AC-006.3 |
| 23 | `agents/doc-orchestrator.md` | Modify | Add cost reporting instructions | AC-006.2, AC-006.3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | ModelId rejects empty string | AC-001.1 | `cargo test -p ecc-domain cost::value_objects::tests::rejects_empty_model_id` | PASS |
| PC-002 | unit | ModelId accepts valid string | AC-001.1 | `cargo test -p ecc-domain cost::value_objects::tests::accepts_valid_model_id` | PASS |
| PC-003 | unit | Money rounds to 6 decimals | AC-001.1 | `cargo test -p ecc-domain cost::value_objects::tests::money_rounds_to_six_decimals` | PASS |
| PC-004 | unit | TokenCount zero is valid | AC-001.1 | `cargo test -p ecc-domain cost::value_objects::tests::token_count_zero_valid` | PASS |
| PC-005 | unit | TokenUsageRecord construction | AC-001.2 | `cargo test -p ecc-domain cost::record::tests::record_construction_all_fields` | PASS |
| PC-006 | unit | PricingTable haiku rates | AC-001.3 | `cargo test -p ecc-domain cost::calculator::tests::haiku_rates` | PASS |
| PC-007 | unit | Sonnet cost estimate | AC-001.3 | `cargo test -p ecc-domain cost::calculator::tests::sonnet_cost_estimate` | PASS |
| PC-008 | unit | Opus with thinking tokens | AC-001.3 | `cargo test -p ecc-domain cost::calculator::tests::opus_with_thinking_tokens` | PASS |
| PC-009 | unit | Summarize groups by model | AC-001.4 | `cargo test -p ecc-domain cost::calculator::tests::summarize_groups_by_model` | PASS |
| PC-010 | unit | Unknown model fallback | AC-001.5 | `cargo test -p ecc-domain cost::calculator::tests::unknown_model_falls_back` | PASS |
| PC-011 | unit | CostStore trait + error display | AC-002.1 | `cargo test -p ecc-ports cost_store::tests::error_display` | PASS |
| PC-012 | unit | InMemoryCostStore round-trip | AC-002.5 | `cargo test -p ecc-test-support in_memory_cost_store::tests::append_and_query_round_trip` | PASS |
| PC-013 | unit | InMemoryCostStore summary | AC-002.5 | `cargo test -p ecc-test-support in_memory_cost_store::tests::summary_aggregates` | PASS |
| PC-014 | integration | SqliteCostStore schema idempotent | AC-002.2 | `cargo test -p ecc-infra sqlite_cost_store::tests::schema_creation_idempotent` | PASS |
| PC-015 | integration | Query with date range filter | AC-002.4 | `cargo test -p ecc-infra sqlite_cost_store::tests::query_date_range_filter` | PASS |
| PC-016 | integration | Prune removes old records | AC-002.2 | `cargo test -p ecc-infra sqlite_cost_store::tests::prune_removes_old_records` | PASS |
| PC-017 | integration | Export JSON format | AC-002.2 | `cargo test -p ecc-infra sqlite_cost_store::tests::export_json_format` | PASS |
| PC-018 | stress | 10 threads x 100 appends WAL | AC-002.3 | `cargo test -p ecc-infra sqlite_cost_store::tests::concurrent_writes_wal -- --ignored` | PASS |
| PC-019 | unit | cost_mgmt summary delegation | AC-004.1 | `cargo test -p ecc-app cost_mgmt::tests::summary_delegates_to_store` | PASS |
| PC-020 | unit | cost_mgmt prune delegation | AC-004.5 | `cargo test -p ecc-app cost_mgmt::tests::prune_delegates_to_store` | PASS |
| PC-021 | unit | Migrate imports valid JSONL | AC-005.1, AC-005.3 | `cargo test -p ecc-app cost_mgmt::tests::migrate_imports_valid_jsonl` | PASS |
| PC-022 | unit | Migrate skips malformed lines | AC-005.2 | `cargo test -p ecc-app cost_mgmt::tests::migrate_skips_malformed_lines` | PASS |
| PC-023 | unit | Migrate missing file | AC-005.4 | `cargo test -p ecc-app cost_mgmt::tests::migrate_missing_file` | PASS |
| PC-024 | unit | Hook uses CostStore::append | AC-003.1, AC-003.2, AC-003.6 | `cargo test -p ecc-app hook::handlers::tier3_session::tests::cost_tracker_uses_cost_store` | PASS |
| PC-025 | unit | Hook JSONL fallback | AC-003.7 | `cargo test -p ecc-app hook::handlers::tier3_session::tests::cost_tracker_falls_back_to_jsonl` | PASS |
| PC-026 | unit | Hook extracts agent_type | AC-003.3 | `cargo test -p ecc-app hook::handlers::tier3_session::tests::cost_tracker_extracts_agent_type` | PASS |
| PC-027 | unit | Hook extracts thinking_tokens | AC-003.4 | `cargo test -p ecc-app hook::handlers::tier3_session::tests::cost_tracker_extracts_thinking_tokens` | PASS |
| PC-028 | unit | HookPorts compiles with cost_store | AC-003.5 | `cargo test -p ecc-app hook::tests::hook_result_passthrough` | PASS |
| PC-029 | unit | CLI parse summary args | AC-004.1 | `cargo test -p ecc-cli commands::cost::tests::parse_summary_args` | PASS |
| PC-030 | unit | CLI parse breakdown args | AC-004.2 | `cargo test -p ecc-cli commands::cost::tests::parse_breakdown_args` | PASS |
| PC-031 | unit | CLI parse compare args | AC-004.3 | `cargo test -p ecc-cli commands::cost::tests::parse_compare_args` | PASS |
| PC-032 | unit | CLI parse export args | AC-004.4 | `cargo test -p ecc-cli commands::cost::tests::parse_export_args` | PASS |
| PC-033 | unit | CLI parse prune args | AC-004.5 | `cargo test -p ecc-cli commands::cost::tests::parse_prune_args` | PASS |
| PC-034 | unit | CLI empty DB message | AC-004.6 | `cargo test -p ecc-cli commands::cost::tests::missing_db_prints_message` | PASS |
| PC-035 | unit | CLI parse migrate args | AC-005.1 | `cargo test -p ecc-cli commands::cost::tests::parse_migrate_args` | PASS |
| PC-036 | unit | cost_mgmt breakdown delegates to store | AC-004.2 | `cargo test -p ecc-app cost_mgmt::tests::breakdown_delegates_to_store` | PASS |
| PC-037 | unit | cost_mgmt compare delegates to store | AC-004.3 | `cargo test -p ecc-app cost_mgmt::tests::compare_delegates_to_store` | PASS |
| PC-038 | unit | cost_mgmt export delegates to store | AC-004.4 | `cargo test -p ecc-app cost_mgmt::tests::export_delegates_to_store` | PASS |
| PC-039 | unit | HookPorts with cost_store None compiles and dispatches | AC-003.5 | `cargo test -p ecc-app hook::tests::hook_ports_with_cost_store_none` | PASS |
| PC-040 | lint | Zero clippy warnings | All | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-041 | build | Release build succeeds | All | `cargo build --release` | exit 0 |

### Coverage Check

All 30 ACs covered:

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001, PC-002, PC-003, PC-004 |
| AC-001.2 | PC-005 |
| AC-001.3 | PC-006, PC-007, PC-008 |
| AC-001.4 | PC-009 |
| AC-001.5 | PC-010 |
| AC-002.1 | PC-011 |
| AC-002.2 | PC-014, PC-016, PC-017 |
| AC-002.3 | PC-018 |
| AC-002.4 | PC-015 |
| AC-002.5 | PC-012, PC-013 |
| AC-003.1 | PC-024 |
| AC-003.2 | PC-024 |
| AC-003.3 | PC-026 |
| AC-003.4 | PC-027 |
| AC-003.5 | PC-039 |
| AC-003.6 | PC-024 |
| AC-003.7 | PC-025 |
| AC-004.1 | PC-019, PC-029 |
| AC-004.2 | PC-030, PC-036 |
| AC-004.3 | PC-031, PC-037 |
| AC-004.4 | PC-032, PC-038 |
| AC-004.5 | PC-020, PC-033 |
| AC-004.6 | PC-034 |
| AC-005.1 | PC-021, PC-035 |
| AC-005.2 | PC-022 |
| AC-005.3 | PC-021 |
| AC-005.4 | PC-023 |
| AC-006.1 | Validated by ecc validate agents |
| AC-006.2 | Validated by ecc validate agents |
| AC-006.3 | Validated by grep in agent definition |

Uncovered ACs: **None.**

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | CostStore write | SqliteCostStore | CostStore::append | Append record to real SQLite, verify round-trip | ignored | ecc-infra cost files modified |
| 2 | CostStore query | SqliteCostStore | CostStore::query/summary | Query with filters against real SQLite | ignored | ecc-infra cost files modified |
| 3 | CLI full stack | ecc cost summary | CostStore::summary | CLI args -> cost_mgmt -> SqliteCostStore | ignored | ecc-cli cost.rs or ecc-app cost_mgmt.rs modified |
| 4 | Hook -> Store | cost_tracker | CostStore::append | Refactored hook writes to real SQLite | ignored | tracking.rs modified |

### E2E Activation Rules

All 4 E2E tests un-ignored during this implementation (all boundaries are new/modified).

## Test Strategy

TDD order (dependency chain):

1. **PC-001..010** — Domain value objects + CostCalculator (no dependencies, pure logic)
2. **PC-011** — CostStore trait definition (depends on domain types)
3. **PC-012..013** — InMemoryCostStore test double (depends on trait)
4. **PC-014..018** — SqliteCostStore integration tests (depends on trait + schema)
5. **PC-019..023, PC-036..038** — cost_mgmt use cases including breakdown, compare, export (depends on InMemoryCostStore)
6. **PC-024..027, PC-039** — Hook refactor + HookPorts validation (depends on domain + InMemoryCostStore)
7. **PC-029..035** — CLI arg parsing (depends on cost_mgmt types)
8. **PC-040..041** — Lint + build gates (final verification)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CLAUDE.md` | Project | Modify | Add `ecc cost` subcommands to CLI Commands section | US-004 |
| 2 | `docs/domain/bounded-contexts.md` | Architecture | Add | Cost bounded context with ubiquitous language | US-001 |
| 3 | `docs/ARCHITECTURE.md` | Architecture | Modify | Add cost module to crate descriptions | US-001, US-002 |
| 4 | `docs/adr/NNN-sqlite-over-jsonl-for-cost.md` | ADR | Create | Decision 1: SQLite instead of JSONL for cost storage | D-1 |
| 5 | `docs/adr/NNN-separate-cost-db.md` | ADR | Create | Decision 2: Separate cost.db from logs ecc.db | D-2 |
| 6 | `docs/adr/NNN-stop-event-for-cost-tracking.md` | ADR | Create | Decision 3: Stop event over PostToolUse | D-3 |
| 7 | `CHANGELOG.md` | Project | Add | feat: add cost and token tracking (BL-096) | All |
| 8 | `docs/MODULE-SUMMARIES.md` | Reference | Modify | Add entries for cost domain, port, infra, app, cli | US-001, US-002 |

## SOLID Assessment

**PASS** (2 MEDIUM, 1 LOW):
- **M-001**: PricingTable should be constructor-injected into CostCalculator, not hardcoded in domain. Rates defined as defaults in ecc-cli/ecc-infra call site. (`calculator.rs`)
- **M-002**: CostStore trait has 5 methods mixing read/write. Acceptable given project convention (MemoryStore has 12). Flag for future ISP split. (`cost_store.rs`)
- **L-001**: Consider inlining cost_schema.rs into sqlite_cost_store.rs if only one consumer. (`cost_schema.rs`)

## Robert's Oath Check

**CLEAN** with 1 advisory:
- Document `Money` value object as "analytics precision only — switch to fixed-point if used for budget enforcement."
- Pricing rate discrepancy with existing code noted (haiku 0.8/4.0 vs spec $1/$5) — spec rates are the target.

## Security Notes

**CLEAR** (2 LOW advisories):
- Set `~/.ecc/cost/` directory permissions to 0700 during initialization
- Non-atomic JSONL write (existing) being fixed by SQLite migration

## Rollback Plan

Reverse dependency order:
1. Revert `agents/doc-orchestrator.md` and `agents/audit-orchestrator.md`
2. Revert `crates/ecc-cli/src/main.rs`, `commands/mod.rs`, remove `commands/cost.rs`
3. Revert `crates/ecc-app/src/hook/handlers/tier3_session/tracking.rs` and `helpers.rs`
4. Revert `crates/ecc-app/src/hook/mod.rs` (remove cost_store from HookPorts)
5. Remove `crates/ecc-app/src/cost_mgmt.rs`
6. Remove `crates/ecc-test-support/src/in_memory_cost_store.rs`
7. Remove `crates/ecc-infra/src/sqlite_cost_store.rs` and `cost_schema.rs`
8. Remove `crates/ecc-ports/src/cost_store.rs`
9. Remove `crates/ecc-domain/src/cost/` directory
10. Delete `~/.ecc/cost/cost.db` file

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 3 (0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW) |
| Robert | CLEAN | 1 advisory |
| Security | CLEAR | 2 LOW advisories |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Coverage | 88 | 95 | PASS | All 30 ACs covered with dual tests |
| Order | 95 | 95 | PASS | Correct TDD dependency chain |
| Fragility | 82 | 88 | PASS | HookPorts blast radius documented |
| Rollback | 90 | 90 | PASS | Clean additive, 10-step reverse |
| Architecture | 95 | 95 | PASS | Hexagonal compliance verified |
| Blast Radius | 80 | 90 | PASS | make_ports() impact documented |
| Missing PCs | 78 | 92 | PASS | breakdown/compare/export PCs added |
| Doc Plan | 85 | 85 | PASS | 8 doc updates, 3 ADRs |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1-5 | `ecc-domain/src/cost/{mod,value_objects,record,calculator,error}.rs` | Create | US-001 |
| 6 | `ecc-domain/src/lib.rs` | Modify | US-001 |
| 7 | `ecc-ports/src/cost_store.rs` | Create | AC-002.1 |
| 8 | `ecc-ports/src/lib.rs` | Modify | US-002 |
| 9-10 | `ecc-infra/src/{cost_schema,sqlite_cost_store}.rs` | Create | AC-002.2-4 |
| 11 | `ecc-infra/src/lib.rs` | Modify | US-002 |
| 12 | `ecc-test-support/src/in_memory_cost_store.rs` | Create | AC-002.5 |
| 13 | `ecc-test-support/src/lib.rs` | Modify | AC-002.5 |
| 14 | `ecc-app/src/cost_mgmt.rs` | Create | US-004, US-005 |
| 15 | `ecc-app/src/lib.rs` | Modify | US-004 |
| 16 | `ecc-app/src/hook/mod.rs` | Modify | AC-003.5 |
| 17-18 | `ecc-app/src/hook/handlers/tier3_session/{tracking,helpers}.rs` | Modify | AC-003.1-7 |
| 19 | `ecc-cli/src/commands/cost.rs` | Create | AC-004.1-6, AC-005.1-4 |
| 20-21 | `ecc-cli/src/commands/mod.rs`, `main.rs` | Modify | US-004 |
| 22-23 | `agents/{audit,doc}-orchestrator.md` | Modify | AC-006.1-3 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-04-cost-token-tracking/spec.md | Full spec with Phase Summary |
| docs/specs/2026-04-04-cost-token-tracking/design.md | Full design with Phase Summary |
