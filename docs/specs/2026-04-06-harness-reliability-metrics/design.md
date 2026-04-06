# Solution: BL-106 Harness Reliability Metrics

## Spec Reference
Concern: dev, Feature: harness-reliability-metrics

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/metrics/targets.rs` | create | ReferenceTargets struct with default SLO constants (hook>=0.99, gate<=0.05, agent>=0.80, commit>=0.95). Pure domain, zero I/O. | US-008, AC-008.3, AC-008.4 |
| 2 | `crates/ecc-domain/src/metrics/trend.rs` | create | TrendComparison struct: current HarnessMetrics, previous HarnessMetrics, computed deltas (each Option<f64>). Pure computation. | US-007, AC-007.1-007.5 |
| 3 | `crates/ecc-domain/src/metrics/mod.rs` | modify | Add `pub mod targets;` and `pub mod trend;` with re-exports. | US-007, US-008 |
| 4 | `crates/ecc-app/src/hook/mod.rs` | modify | Add `metrics_store: Option<&'a dyn MetricsStore>` to HookPorts. Instrument dispatch(): after handler match, build MetricEvent::hook_execution with session_id (via resolve_session_id), duration_ms, outcome, call record_if_enabled. Read ECC_METRICS_DISABLED from ports.env. | US-001, AC-001.1-001.6; US-005, AC-005.1, AC-005.3 |
| 5 | `crates/ecc-app/src/hook/handlers/tier3_session/logging.rs` | modify | Instrument subagent_start_log/subagent_stop_log: record AgentSpawn events. Parse $.error, $.exit_code, $.retry_count from stdin JSON for failure detection and retry count. | US-003, AC-003.1-003.6 |
| 6 | `crates/ecc-app/src/hook/handlers/tier2_tools/quality.rs` | modify | Instrument quality_gate: record CommitGate events after formatter checks (Passed/Failure with CommitGateKind). | US-004, AC-004.1-004.3, AC-004.4a |
| 7 | `crates/ecc-app/src/metrics_mgmt.rs` | modify | Add record_commit_gate() for CLI-callable gate recording. Add trend_summary() querying mirror window [now-N, now] vs [now-2N, now-N]. Replace eprintln! with tracing::warn! (uncle-bob finding). | US-004 AC-004.4b; US-007 AC-007.1-007.2; US-005 AC-005.2 |
| 8 | `crates/ecc-workflow/src/main.rs` | modify | Construct SqliteMetricsStore at binary entry point (DIP fix per uncle-bob). Thread Option<&dyn MetricsStore> to transition command. | US-002 |
| 9 | `crates/ecc-workflow/src/commands/transition.rs` | modify | Accept store: Option<&dyn MetricsStore> parameter. Extract try_record_transition() helper (SRP fix). Record PhaseTransition events: Success on valid transition, Rejected on invalid. Session ID = "workflow-{feature}". Fire-and-forget. | US-002, AC-002.1-002.6 |
| 10 | `crates/ecc-cli/src/commands/hook.rs` | modify | Wire SqliteMetricsStore into HookPorts (fire-and-forget on open failure: set None). Read ECC_METRICS_DISABLED. | US-001, AC-001.5 |
| 11 | `crates/ecc-cli/src/commands/metrics.rs` | modify | Add --json flag (JSON output with Option<f64> fields). Add --trend flag with mirror window. Add --trend+--session mutual exclusion. Add RecordGate subcommand (--kind, --outcome). Add "vs. Target" SLO column with [!] prefix for below-target. | US-006 AC-006.5; US-007 AC-007.1-007.4; US-004 AC-004.4b; US-008 AC-008.1-008.2 |
| 12 | `commands/catchup.md` | modify | Add "Harness Metrics" section: invoke ecc metrics summary --session <id> --json via Bash, parse JSON, display rates as percentages, N/A handling, skip silently on failure. | US-006, AC-006.1-006.5 |
| 13 | All make_ports/HookPorts sites in ecc-app tests (~30 sites across ~28 files) | modify | Add metrics_store: None to every HookPorts construction (mechanical). | build fix |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | ReferenceTargets::default() returns correct SLO values | AC-008.3, AC-008.4 | `cargo test -p ecc-domain -- targets::tests` | PASS |
| PC-002 | unit | TrendComparison::compute() returns correct deltas; both-NA yields NA delta | AC-007.1, AC-007.2, AC-007.3, AC-007.5 | `cargo test -p ecc-domain -- trend::tests` | PASS |
| PC-003 | unit | HookPorts compiles with metrics_store field; dispatch works with None | AC-001.5, AC-001.4 | `cargo test -p ecc-app -- hook::tests::hook_ports_with_metrics_store_none` | PASS |
| PC-004 | unit | dispatch() records HookExecution/Success with duration_ms > 0 | AC-001.1 | `cargo test -p ecc-app -- hook::tests::dispatch_records_hook_success_metric` | PASS |
| PC-005 | unit | dispatch() records HookExecution/Failure with error_message | AC-001.2 | `cargo test -p ecc-app -- hook::tests::dispatch_records_hook_failure_metric` | PASS |
| PC-006 | unit | ECC_METRICS_DISABLED=1 records zero events in dispatch | AC-001.3, AC-005.1 | `cargo test -p ecc-app -- hook::tests::dispatch_metrics_disabled_records_nothing` | PASS |
| PC-007 | unit | metrics_store: None completes normally (fire-and-forget) | AC-001.4 | `cargo test -p ecc-app -- hook::tests::dispatch_none_store_fire_and_forget` | PASS |
| PC-008 | unit | Session ID uses resolve_session_id with env set/unset | AC-001.6 | `cargo test -p ecc-app -- hook::tests::dispatch_session_id_resolution` | PASS |
| PC-009 | unit | subagent_start_log records AgentSpawn/Success with agent_type | AC-003.1 | `cargo test -p ecc-app -- tier3_session::logging::tests::subagent_start_records_agent_spawn_success` | PASS |
| PC-010 | unit | subagent_stop_log with $.error records AgentSpawn/Failure | AC-003.2, AC-003.5 | `cargo test -p ecc-app -- tier3_session::logging::tests::subagent_stop_records_agent_spawn_failure` | PASS |
| PC-011 | unit | subagent_stop_log parses $.retry_count as u32 | AC-003.3 | `cargo test -p ecc-app -- tier3_session::logging::tests::subagent_stop_parses_retry_count` | PASS |
| PC-012 | unit | subagent_stop_log with no recognized fields defaults to Success/None | AC-003.6 | `cargo test -p ecc-app -- tier3_session::logging::tests::subagent_stop_defaults_success_none` | PASS |
| PC-013 | unit | subagent_start_log with ECC_METRICS_DISABLED=1 records nothing | AC-003.4 | `cargo test -p ecc-app -- tier3_session::logging::tests::subagent_start_metrics_disabled` | PASS |
| PC-014 | unit | quality_gate records CommitGate/Passed on formatter success | AC-004.1, AC-004.4a | `cargo test -p ecc-app -- tier2_tools::quality::tests::quality_gate_records_commit_gate_passed` | PASS |
| PC-015 | unit | quality_gate records CommitGate/Failure with gates_failed | AC-004.2, AC-004.4a | `cargo test -p ecc-app -- tier2_tools::quality::tests::quality_gate_records_commit_gate_failure` | PASS |
| PC-016 | unit | quality_gate with ECC_METRICS_DISABLED=1 records nothing | AC-004.3 | `cargo test -p ecc-app -- tier2_tools::quality::tests::quality_gate_metrics_disabled` | PASS |
| PC-017 | unit | record_commit_gate(pass) records CommitGate/Passed | AC-004.4b | `cargo test -p ecc-app -- metrics_mgmt::tests::record_commit_gate_pass` | PASS |
| PC-018 | unit | record_commit_gate(fail, test) records CommitGate/Failure | AC-004.4b | `cargo test -p ecc-app -- metrics_mgmt::tests::record_commit_gate_fail` | PASS |
| PC-019 | unit | record_if_enabled(disabled=true) returns Ok, 0 events (existing) | AC-005.1 | `cargo test -p ecc-app -- metrics_mgmt::tests::metrics_disabled_flag` | PASS |
| PC-020 | unit | record_if_enabled(store=None) returns Ok (fire-and-forget, existing) | AC-005.1, AC-005.3 | `cargo test -p ecc-app -- metrics_mgmt::tests::metrics_fire_and_forget` | PASS |
| PC-021 | unit | summary() works when kill switch active (reads existing data) | AC-005.2 | `cargo test -p ecc-app -- metrics_mgmt::tests::summary_works_with_kill_switch` | PASS |
| PC-022 | unit | trend_summary() with events in both periods returns correct deltas | AC-007.1, AC-007.3 | `cargo test -p ecc-app -- metrics_mgmt::tests::trend_summary_with_events` | PASS |
| PC-023 | unit | trend_summary() with no previous events returns None previous | AC-007.2 | `cargo test -p ecc-app -- metrics_mgmt::tests::trend_summary_no_previous` | PASS |
| PC-024 | unit | trend_summary() both periods NA returns NA deltas | AC-007.5 | `cargo test -p ecc-app -- metrics_mgmt::tests::trend_summary_both_na` | PASS |
| PC-025 | unit | CLI --trend --session returns error | AC-007.4 | `cargo test -p ecc-cli -- metrics::tests::trend_session_incompatible` | PASS |
| PC-026 | unit | CLI summary --json outputs valid JSON with four Option<f64> fields | AC-006.5 | `cargo test -p ecc-cli -- metrics::tests::summary_json_output` | PASS |
| PC-027 | unit | CLI summary shows "vs. Target" column with SLO values | AC-008.1 | `cargo test -p ecc-cli -- metrics::tests::summary_shows_slo_column` | PASS |
| PC-028 | unit | CLI flags [!] prefix when metric below SLO | AC-008.2 | `cargo test -p ecc-cli -- metrics::tests::summary_flags_below_slo` | PASS |
| PC-029 | unit | CLI summary --trend shows Current/Previous/delta columns | AC-007.1 | `cargo test -p ecc-cli -- metrics::tests::summary_trend_columns` | PASS |
| PC-030 | unit | CLI record-gate subcommand records CommitGate event | AC-004.4b | `cargo test -p ecc-cli -- metrics::tests::record_gate_subcommand` | PASS |
| PC-031 | unit | try_record_transition() with success records PhaseTransition/Success | AC-002.1, AC-002.5 | `cargo test -p ecc-workflow -- commands::transition::tests::transition_records_success_metric` | PASS |
| PC-032 | unit | try_record_transition() with rejection records PhaseTransition/Rejected | AC-002.2 | `cargo test -p ecc-workflow -- commands::transition::tests::transition_records_rejected_metric` | PASS |
| PC-033 | unit | try_record_transition() with disabled=true records nothing | AC-002.3 | `cargo test -p ecc-workflow -- commands::transition::tests::transition_metrics_disabled` | PASS |
| PC-034 | unit | transition::run with store=None completes normally | AC-002.4 | `cargo test -p ecc-workflow -- commands::transition::tests::transition_metrics_store_unavailable` | PASS |
| PC-035 | unit | transition::run with no state.json records no metric | AC-002.6 | `cargo test -p ecc-workflow -- commands::transition::tests::transition_no_state_no_metric` | PASS |
| PC-036 | lint | cargo clippy zero warnings | all | `cargo clippy -- -D warnings` | exit 0 |
| PC-037 | build | cargo build workspace succeeds | all | `cargo build --workspace` | exit 0 |
| PC-038 | lint | cargo fmt check passes | all | `cargo fmt --check` | exit 0 |
| PC-039 | unit | catchup.md contains Harness Metrics section with ecc metrics summary --json invocation | AC-006.1, AC-006.2, AC-006.3, AC-006.4 | `grep -q 'ecc metrics summary.*--json' commands/catchup.md && grep -q 'Harness Metrics' commands/catchup.md` | exit 0 |

### Coverage Check

All 40 ACs covered:
- AC-001.1→PC-004, AC-001.2→PC-005, AC-001.3→PC-006, AC-001.4→PC-003/007, AC-001.5→PC-003, AC-001.6→PC-008
- AC-002.1→PC-031, AC-002.2→PC-032, AC-002.3→PC-033, AC-002.4→PC-034, AC-002.5→PC-031, AC-002.6→PC-035
- AC-003.1→PC-009, AC-003.2→PC-010, AC-003.3→PC-011, AC-003.4→PC-013, AC-003.5→PC-010, AC-003.6→PC-012
- AC-004.1→PC-014, AC-004.2→PC-015, AC-004.3→PC-016, AC-004.4a→PC-014/015, AC-004.4b→PC-017/018/030
- AC-005.1→PC-006/019, AC-005.2→PC-021, AC-005.3→PC-020
- AC-006.1-006.4→PC-039 (catchup content verification), AC-006.5→PC-026
- AC-007.1→PC-022/029, AC-007.2→PC-023, AC-007.3→PC-022, AC-007.4→PC-025, AC-007.5→PC-024
- AC-008.1→PC-027, AC-008.2→PC-028, AC-008.3→PC-001, AC-008.4→PC-001

Zero uncovered ACs.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | MetricsStore write | SqliteMetricsStore | MetricsStore::record | Verify hook dispatch writes event to real SQLite DB | ignored | MetricsStore adapter modified |
| 2 | CLI metrics output | SqliteMetricsStore | MetricsStore::summarize | Verify ecc metrics summary --json outputs valid JSON from real DB | ignored | CLI output format changed |

### E2E Activation Rules

No E2E tests activated for this implementation — all instrumentation is fire-and-forget with no schema changes. Unit tests with InMemoryMetricsStore provide full coverage. E2E tests above exist for regression detection only.

## Test Strategy

TDD order (dependency-first):

1. PC-001, PC-002 — Domain types (no deps)
2. PC-003 — HookPorts field + build fix including all ~30 make_ports sites (no deps)
3. PC-004, PC-005, PC-006, PC-007, PC-008 — Hook dispatch (depends on Phase 2)
4. PC-009, PC-010, PC-011, PC-012, PC-013 — Agent spawn (depends on Phase 2)
5. PC-014, PC-015, PC-016 — Commit gate hook (depends on Phase 2)
6. PC-017, PC-018, PC-019, PC-020, PC-021 — Commit gate app + kill switch (independent)
7. PC-022, PC-023, PC-024 — Trend summary (depends on Phase 1)
8. PC-031, PC-032, PC-033, PC-034, PC-035 — Workflow transition (independent)
9. PC-025, PC-026, PC-027, PC-028, PC-029, PC-030 — CLI wiring (depends on Phases 1, 6, 7)
10. Catchup.md — Manual (depends on Phase 9)
11. PC-036, PC-037, PC-038, PC-039 — Final gates (build, clippy, fmt, catchup content check)

Phases 1+2 run first (foundation). Phases 3, 4, 5, 8 are parallelizable. Phase 6+7 depend on Phase 1. Phase 9 depends on 1+6+7. Phase 10 depends on 9.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | project | modify | Add --trend, --json to ecc metrics summary; add ecc metrics record-gate subcommand | US-007, US-006, US-004 |
| 2 | CLAUDE.md | project | modify | Add ECC_METRICS_DISABLED to Gotchas section | US-005 |
| 3 | CHANGELOG.md | project | modify | feat: wire harness reliability metrics + trends + SLO targets | all |
| 4 | commands/catchup.md | command | modify | Add Harness Metrics section | US-006 |
| 5 | docs/MODULE-SUMMARIES.md | project | modify | Add entries for ReferenceTargets and TrendComparison domain types | US-007, US-008 |

No ADRs needed (all decisions marked "No" in spec).

## SOLID Assessment

From uncle-bob agent (NEEDS WORK, 3 findings):
- **HIGH — DIP**: Resolved. SqliteMetricsStore constructed in ecc-workflow/src/main.rs (not transition.rs). Passed as Option<&dyn MetricsStore> to transition::run().
- **MEDIUM — SRP**: Resolved. try_record_transition() extracted as pure testable function. transition::run() delegates to it.
- **MEDIUM — Clean Code**: Resolved. eprintln! in record_if_enabled replaced with tracing::warn!.

Additional findings noted but deferred:
- ISP (MetricsRecorder trait split) — deferred, MetricsStore has only 5 methods
- CRP tension in metrics_mgmt.rs — deferred, acceptable at current scope

## Robert's Oath Check

CLEAN with 1 warning:
- Warning: AC-004.4b (CLI record-gate path) coverage tier is 80%, not explicitly enumerated
- Rework ratio: 0.20 (healthy)

## Security Notes

CLEAR — 2 MEDIUM findings (defense-in-depth, not blockers):
- Field length validation on MetricEvent string fields (session_id, hook_id, agent_type ≤ 256 bytes; error_message ≤ 4096 bytes). Add in domain constructors.
- JSON-extracted field caps: truncate agent_type and error strings from hook stdin before recording.

4 LOW findings (hygiene): dirs::home_dir() fallback, --session length validation, --since validation, hand-rolled JSON in export. Noted for future cleanup.

## Rollback Plan

Reverse dependency order (undo if implementation fails):
1. Revert commands/catchup.md
2. Revert crates/ecc-cli/src/commands/metrics.rs
3. Revert crates/ecc-cli/src/commands/hook.rs
4. Revert crates/ecc-workflow/src/commands/transition.rs
5. Revert crates/ecc-workflow/src/main.rs
6. Revert crates/ecc-app/src/metrics_mgmt.rs
7. Revert crates/ecc-app/src/hook/handlers/tier2_tools/quality.rs
8. Revert crates/ecc-app/src/hook/handlers/tier3_session/logging.rs
9. Revert crates/ecc-app/src/hook/mod.rs (+ all make_ports sites)
10. Revert crates/ecc-domain/src/metrics/mod.rs
11. Delete crates/ecc-domain/src/metrics/trend.rs
12. Delete crates/ecc-domain/src/metrics/targets.rs

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| Metrics | value objects + aggregation | targets.rs (new), trend.rs (new), mod.rs |
| Hook Dispatch | orchestration | mod.rs (HookPorts + dispatch) |

Other domain modules (not registered as bounded contexts):
- hook/handlers/tier3_session: logging.rs (instrumentation only)
- hook/handlers/tier2_tools: quality.rs (instrumentation only)
- metrics_mgmt: app orchestration layer
- ecc-workflow: standalone binary, transition command
- ecc-cli: framework wiring layer
