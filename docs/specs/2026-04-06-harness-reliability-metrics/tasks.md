# Tasks: BL-106 Harness Reliability Metrics

## Pass Conditions

| ID | Description | Status | Updated |
|----|-------------|--------|---------|
| PC-001 | ReferenceTargets::default() returns correct SLO values | pending | -- |
| PC-002 | TrendComparison::compute() returns correct deltas | pending | -- |
| PC-003 | HookPorts compiles with metrics_store; dispatch works with None | pending | -- |
| PC-004 | dispatch() records HookExecution/Success with duration_ms | pending | -- |
| PC-005 | dispatch() records HookExecution/Failure with error_message | pending | -- |
| PC-006 | ECC_METRICS_DISABLED=1 records zero events in dispatch | pending | -- |
| PC-007 | metrics_store: None completes normally | pending | -- |
| PC-008 | Session ID via resolve_session_id | pending | -- |
| PC-009 | subagent_start_log records AgentSpawn/Success | pending | -- |
| PC-010 | subagent_stop_log with $.error records AgentSpawn/Failure | pending | -- |
| PC-011 | subagent_stop_log parses $.retry_count | pending | -- |
| PC-012 | subagent_stop_log defaults to Success/None | pending | -- |
| PC-013 | subagent_start_log with ECC_METRICS_DISABLED records nothing | pending | -- |
| PC-014 | quality_gate records CommitGate/Passed | pending | -- |
| PC-015 | quality_gate records CommitGate/Failure with gates_failed | pending | -- |
| PC-016 | quality_gate with ECC_METRICS_DISABLED records nothing | pending | -- |
| PC-017 | record_commit_gate(pass) records CommitGate/Passed | pending | -- |
| PC-018 | record_commit_gate(fail) records CommitGate/Failure | pending | -- |
| PC-019 | record_if_enabled(disabled=true) returns Ok, 0 events | existing | -- |
| PC-020 | record_if_enabled(store=None) returns Ok | existing | -- |
| PC-021 | summary() works when kill switch active | pending | -- |
| PC-022 | trend_summary() with events in both periods | pending | -- |
| PC-023 | trend_summary() with no previous events | pending | -- |
| PC-024 | trend_summary() both periods NA | pending | -- |
| PC-025 | CLI --trend --session returns error | pending | -- |
| PC-026 | CLI summary --json outputs valid JSON | pending | -- |
| PC-027 | CLI summary shows vs. Target column | pending | -- |
| PC-028 | CLI flags [!] prefix below SLO | pending | -- |
| PC-029 | CLI summary --trend shows columns | pending | -- |
| PC-030 | CLI record-gate subcommand | pending | -- |
| PC-031 | try_record_transition() success records PhaseTransition/Success | pending | -- |
| PC-032 | try_record_transition() rejection records PhaseTransition/Rejected | pending | -- |
| PC-033 | try_record_transition() disabled records nothing | pending | -- |
| PC-034 | transition::run with store=None completes normally | pending | -- |
| PC-035 | transition::run with no state.json records no metric | pending | -- |
| PC-036 | cargo clippy -- -D warnings | pending | -- |
| PC-037 | cargo build --workspace | pending | -- |
| PC-038 | cargo fmt --check | pending | -- |
| PC-039 | catchup.md contains Harness Metrics section | pending | -- |

## Post-TDD

| Task | Status | Updated |
|------|--------|---------|
| E2E tests | pending | -- |
| Code review | pending | -- |
| Doc updates | pending | -- |
| Supplemental docs | pending | -- |
| Write implement-done.md | pending | -- |
