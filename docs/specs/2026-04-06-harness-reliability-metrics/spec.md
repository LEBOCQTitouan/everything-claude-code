# Spec: BL-106 Harness Reliability Metrics

## Problem Statement

ECC's harness has strong reliability primitives (hooks, phase gates, state machine, agent orchestration) but no measurement. The metrics infrastructure exists end-to-end (domain types, port trait, SQLite adapter, in-memory test double, app orchestration, CLI commands, 36+ tests) but `record_if_enabled()` is never called from any execution path. Without data, we can't identify degradation, benchmark quality, or prove improvement. This spec wires the existing infrastructure to live execution paths and adds consumer features for trend analysis and benchmarking.

## Research Summary

- Compound failure rate is the core reliability challenge: 20 steps at 95% each yields 36% end-to-end completion. Per-step success rate measurement is essential to identify where chains break down.
- Anthropic recommends separating worker from evaluator in long-running agent harnesses; no numeric reliability targets are published.
- Community implementations (claude-code-hooks-multi-agent-observability) instrument 12 hook lifecycle events with per-event success/failure rates, tool accept/reject rates, and error rate trends.
- No published quantitative harness quality thresholds exist yet. ECC can define its own SLO-style targets as domain constants.
- OpenTelemetry is the emerging standard for agent observability; SQLite+WAL for local storage aligns with ECC's existing log/cost infrastructure.
- Four reliability dimensions to measure: consistency, robustness, calibration, and safety. Hook metrics should map to at least one dimension.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | All 8 user stories in v1 (instrumentation + trends + benchmarks) | User chose full scope | No |
| 2 | ecc-workflow uses existing ecc-infra dependency for metrics recording | ecc-workflow already depends on ecc-infra; no new coupling introduced | No |
| 3 | Agent retry_count: best-effort parse from hook payload, None fallback | User wants extraction attempt; graceful degradation when not available | No |
| 4 | ECC aspirational SLOs as domain constants, configurable via ecc config | No external benchmarks exist; honest labeling as "ECC targets" | No |
| 5 | Test coverage: 100% on critical paths (record_if_enabled, aggregation, kill switch, SLO comparison), 80% on CLI wiring and formatting | Balance thoroughness with pragmatism | No |
| 6 | No security concerns: local-only SQLite metrics, no PII, no external transmission | Kill switch provides opt-out | No |
| 7 | HookPorts field addition is non-breaking (Option wrapper) | Follows existing CostStore pattern | No |
| 8 | Session ID: resolved via `resolve_session_id()` from `metrics_session.rs` (reads CLAUDE_SESSION_ID, falls back to "fallback-<ts>-<pid>"); in ecc-workflow, uses "workflow-<feature>" from state.json | MetricEvent requires non-optional session_id; reuse existing resolution function | No |
| 9 | Timestamps generated at call sites using chrono::Utc::now().to_rfc3339() | ecc-domain must remain I/O-free; timestamps are a call-site concern | No |
| 10 | Trend previous period = mirror window: --since Nd compares [now-N, now] vs [now-2N, now-N] | Intuitive and deterministic algorithm | No |
| 11 | CommitGate instrumentation via post:quality-gate hook AND new /verify instrumentation section in catchup | Existing quality.rs handles formatters; /verify runs build+test+lint but is markdown command | No |
| 12 | Catchup consumes metrics via `ecc metrics summary --session <id> --json` Bash call | Markdown command uses Bash tool; JSON output enables reliable parsing | No |
| 13 | Default SLO targets: hook_success >= 99%, phase_gate_violation <= 5%, agent_recovery >= 80%, commit_atomicity >= 95% | Aspirational starting points; configurable via ecc config | No |

## User Stories

### US-001: Hook Execution Instrumentation

**As a** developer using ECC, **I want** every hook dispatch to record a metric event (success/failure, duration, hook ID), **so that** I can measure hook reliability over time.

#### Acceptance Criteria

- AC-001.1: Given a hook dispatch, when the hook completes successfully, then a HookExecution metric event with outcome Success and duration_ms is recorded via record_if_enabled
- AC-001.2: Given a hook dispatch, when the hook fails (exit_code != 0), then a HookExecution event with outcome Failure and error message is recorded
- AC-001.3: Given ECC_METRICS_DISABLED=1, when a hook dispatches, then no metric event is recorded (zero-cost kill switch)
- AC-001.4: Given the metrics store is unavailable (None), when a hook dispatches, then the hook still executes normally (fire-and-forget)
- AC-001.5: Given HookPorts, when constructed, then it includes metrics_store: Option<&dyn MetricsStore>
- AC-001.6: Given CLAUDE_SESSION_ID is unset or empty during hook dispatch, when a metric event would be recorded, then session_id is resolved via the existing `resolve_session_id()` function in `metrics_session.rs` (returns `"fallback-<timestamp>-<pid>"`)

#### Dependencies

- Depends on: none

### US-002: Workflow Phase Transition Instrumentation

**As a** developer using ECC, **I want** every workflow state transition (success or rejected) to record a metric event, **so that** I can track phase-gate violation rates.

#### Acceptance Criteria

- AC-002.1: Given a transition succeeds, when ecc workflow transition runs, then a PhaseTransition event with outcome Success and from/to phases is recorded
- AC-002.2: Given a transition is rejected (invalid phase change), when ecc workflow transition runs, then a PhaseTransition event with outcome Rejected and rejection reason is recorded
- AC-002.3: Given ECC_METRICS_DISABLED=1, when a transition occurs, then no metric event is recorded
- AC-002.4: Given the metrics database doesn't exist or can't be opened, when a transition occurs, then the transition still succeeds (fire-and-forget)
- AC-002.5: Given CLAUDE_SESSION_ID is unavailable in ecc-workflow, then session_id is derived as "workflow-<feature>" from state.json feature field
- AC-002.6: Given no state.json exists when ecc workflow transition runs, then no PhaseTransition metric event is recorded

#### Dependencies

- Depends on: none (parallel with US-001; ecc-workflow creates its own SqliteMetricsStore instance via existing ecc-infra dependency)

### US-003: Agent Spawn Instrumentation

**As a** developer using ECC, **I want** agent spawn successes and failures to be recorded as metric events, **so that** I can measure agent failure recovery rates.

#### Acceptance Criteria

- AC-003.1: Given subagent:start:log fires, when an agent starts, then an AgentSpawn event with agent_type and outcome Success is recorded
- AC-003.2: Given subagent:stop:log fires with failure indicators, when an agent errored, then an AgentSpawn event with outcome Failure and agent_type is recorded
- AC-003.3: Given the JSON field `$.retry_count` (u32) exists in subagent:stop:log stdin payload, when parseable, then retry_count is populated; otherwise None
- AC-003.4: Given ECC_METRICS_DISABLED=1, when agent spawn metric would be recorded, then it is skipped
- AC-003.5: Given the subagent:stop:log stdin contains `$.error` (non-null string) OR `$.exit_code != 0`, then the AgentSpawn outcome is Failure; otherwise Success
- AC-003.6: Given no recognized failure or retry fields exist in the payload, then outcome defaults to Success and retry_count to None

#### Dependencies

- Depends on: US-001 (HookPorts gains metrics_store field)

### US-004: Commit Gate Instrumentation

**As a** developer using ECC, **I want** commit gate results (build/test/lint pass/fail) to be recorded as metric events, **so that** I can measure commit atomicity scores.

#### Acceptance Criteria

- AC-004.1: Given a quality gate check succeeds (all tools pass), then a CommitGate event with outcome Passed is recorded
- AC-004.2: Given a quality gate check fails, then a CommitGate event with outcome Failure and gates_failed (Build, Test, Lint) is recorded
- AC-004.3: Given ECC_METRICS_DISABLED=1, when a commit gate fires, then no metric event is recorded
- AC-004.4a: Instrumentation point (formatters): the `post:quality-gate` hook handler in quality.rs records CommitGate events for formatter checks (biome, prettier, ruff). This path is compiled Rust, testable with InMemoryMetricsStore.
- AC-004.4b: Instrumentation point (build/test/lint): a new `record_commit_gate()` function in metrics_mgmt.rs is callable from CLI or commands. The /verify command invokes it via `ecc metrics record-gate --kind build|test|lint --outcome pass|fail` Bash calls after each gate step. This path uses the existing CLI → SqliteMetricsStore wiring.

#### Dependencies

- Depends on: US-001 (HookPorts gains metrics_store field)

### US-005: Metrics Kill Switch (ECC_METRICS_DISABLED)

**As a** developer using ECC, **I want** a zero-cost environment variable kill switch (ECC_METRICS_DISABLED=1) that completely disables metrics recording, **so that** I can opt out of metrics overhead when needed.

#### Acceptance Criteria

- AC-005.1: Given ECC_METRICS_DISABLED=1, when any instrumentation point fires, then record_if_enabled returns immediately without touching the store
- AC-005.2: Given the kill switch is active, when ecc metrics summary is run, then the summary still works (reads existing data; only recording is disabled)
- AC-005.3: Given ECC_METRICS_DISABLED is unset or empty, when instrumentation fires, then metrics are recorded normally

#### Dependencies

- Depends on: US-001 (wiring path for disabled flag)

### US-006: Catchup Metrics Summary Integration

**As a** developer resuming a session with /catchup, **I want** to see a harness metrics summary (hook success rate, phase-gate violation rate, agent recovery rate, commit atomicity score) for the current session, **so that** I can gauge session health at a glance.

#### Acceptance Criteria

- AC-006.1: Given a session with metric events, when /catchup runs, then a "Harness Metrics" section displays all four rates formatted as percentages
- AC-006.2: Given a session with no metric events, when /catchup runs, then "No harness metrics recorded for this session." is shown
- AC-006.3: Given the metrics database is unavailable, when /catchup runs, then the Harness Metrics section is skipped silently (no error)
- AC-006.4: Given rates are N/A (no events of that type), when displayed, then "N/A" is shown for that specific rate
- AC-006.5: Catchup invokes `ecc metrics summary --session <id> --json` via the Bash tool and parses the JSON output. The --json flag outputs a machine-readable JSON object with fields: hook_success_rate, phase_gate_violation_rate, agent_failure_recovery_rate, commit_atomicity_score (each Option<f64>)

#### Dependencies

- Depends on: US-001 through US-004 (all four instrumentation points needed for full display; AC-006.4 provides graceful N/A degradation if partial)

### US-007: Cross-Session Trend Reporting

**As a** developer tracking ECC reliability over time, **I want** ecc metrics summary to optionally show a trend comparison (current period vs. previous period), **so that** I can see if harness reliability is improving or degrading.

#### Acceptance Criteria

- AC-007.1: Given ecc metrics summary --since 7d --trend, when executed, then two columns are shown: "Current" and "Previous" with delta indicators (+/-%)
- AC-007.2: Given no events in the previous period, when trend is requested, then "N/A" is shown for the previous column
- AC-007.3: Given a positive delta in hook success rate, when displayed, then a "+" prefix is shown (e.g., "+5.2%")
- AC-007.4: Given --trend and --session are both specified, then return an error: "--trend is incompatible with --session"
- AC-007.5: Given both current and previous periods have N/A for a metric type, then delta column shows "N/A"

#### Dependencies

- Depends on: US-001 (metrics must be recorded for trends to exist)

### US-008: Benchmarking with ECC SLO Targets

**As a** developer evaluating ECC quality, **I want** a benchmark comparison showing how ECC metrics compare against aspirational SLO targets, **so that** I can identify gaps and improvement areas.

#### Acceptance Criteria

- AC-008.1: Given ecc metrics summary output, when displayed, then a "vs. Target" column shows the ECC SLO for each metric
- AC-008.2: Given ECC's hook success rate is below the SLO target, when displayed, then the metric is flagged with a `[!]` prefix (e.g., `[!] Hook Success: 94.2% (target: 99%)`)
- AC-008.3: Given SLO targets, when defined, then they are stored as constants in ecc-domain (not hard-coded in CLI) with configurable overrides via ecc config
- AC-008.4: Default SLO targets: hook_success >= 99%, phase_gate_violation <= 5%, agent_recovery >= 80%, commit_atomicity >= 95%

#### Dependencies

- Depends on: US-001 (metrics must be recorded for comparison)

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| ecc-domain | Domain | Add TrendComparison type, ReferenceTargets struct with SLO defaults |
| ecc-ports | Ports | No changes needed (MetricsStore trait already complete) |
| ecc-app | App | Add metrics_store to HookPorts, instrument handlers, add trend function |
| ecc-infra | Infra | No changes needed (SqliteMetricsStore already complete) |
| ecc-cli | CLI | Wire SqliteMetricsStore to HookPorts, add --trend flag, add SLO column |
| ecc-workflow | Binary | Instrument transition.rs with MetricEvent recording |
| commands/ | Markdown | Add Harness Metrics section to catchup.md |

## Constraints

- `record_if_enabled` must remain fire-and-forget: never block the caller's operation on metrics failure
- `ECC_METRICS_DISABLED` kill switch must be zero-cost when active (check before any store interaction)
- `ecc-domain` must have zero I/O imports: pure business logic only
- All tests use `InMemoryMetricsStore` from `ecc-test-support`, never `SqliteMetricsStore`
- Metrics database path: `~/.ecc/metrics/metrics.db` (matches existing convention)
- Concurrent metric writes from parallel hook dispatches must not block or fail (SQLite WAL mode + busy_timeout handle this; constraint validates the assumption)
- All MetricsStore instances (hooks, ecc-workflow, CLI) must use the same database path: `~/.ecc/metrics/metrics.db`

## Non-Requirements

- External metric export (OpenTelemetry, Datadog, Grafana) -- local SQLite only for v1
- Real-time dashboards or WebSocket streaming
- Per-command cost attribution (covered by BL-096)
- Distributed tracing or correlation IDs across sessions
- Agent retry orchestration (only observation, not control)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| MetricsStore | Wiring only (no trait changes) | Existing adapter gains new call sites; no schema changes |
| HookPorts | Additive field (Option<&dyn MetricsStore>) | All HookPorts constructors updated; backward-compatible via Option |
| ecc-workflow transition | New store instantiation | Transition gains SQLite write; fire-and-forget on failure |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI flag | CLAUDE.md | CLI Commands section | Add --trend to ecc metrics summary |
| Kill switch | CLAUDE.md | Gotchas section | Document ECC_METRICS_DISABLED env var |
| SLO targets | docs/ | New section in ARCHITECTURE.md or dedicated doc | Document default target values |
| Catchup enhancement | commands/ | catchup.md | Add Harness Metrics section |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is out of scope? | Nothing deferred -- all 8 user stories included | User |
| 2 | How should ecc-workflow record phase transition metrics? | Option (a): use existing ecc-infra dependency directly | User |
| 3 | Agent retry count availability? | Best-effort parse from subagent:stop:log payload; None fallback | User |
| 4 | How to define reference targets? | ECC aspirational SLOs as domain constants, configurable via ecc config | Recommended |
| 5 | Test coverage strategy? | 100% on critical paths (record_if_enabled, aggregation, kill switch, SLO); 80% on CLI/formatting | Recommended |
| 6 | Security concerns? | None -- local-only SQLite, no PII, no external transmission | Recommended |
| 7 | Breaking changes? | None -- Option wrapper is additive, ecc-workflow dep already exists | Recommended |
| 8 | ADR for ecc-workflow? | Not needed -- ecc-workflow already depends on ecc-infra (moot) | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Hook Execution Instrumentation | 6 | none |
| US-002 | Workflow Phase Transition Instrumentation | 6 | none (parallel) |
| US-003 | Agent Spawn Instrumentation | 6 | US-001 |
| US-004 | Commit Gate Instrumentation | 5 | US-001 |
| US-005 | Metrics Kill Switch | 3 | US-001 |
| US-006 | Catchup Metrics Summary | 5 | US-001 through US-004 |
| US-007 | Cross-Session Trend Reporting | 5 | US-001 |
| US-008 | Benchmarking with ECC SLO Targets | 4 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Hook success records HookExecution/Success with duration_ms | US-001 |
| AC-001.2 | Hook failure records HookExecution/Failure with error | US-001 |
| AC-001.3 | ECC_METRICS_DISABLED=1 skips recording | US-001 |
| AC-001.4 | None store = fire-and-forget | US-001 |
| AC-001.5 | HookPorts gains metrics_store: Option | US-001 |
| AC-001.6 | Session ID via resolve_session_id() fallback | US-001 |
| AC-002.1 | Transition success records PhaseTransition/Success | US-002 |
| AC-002.2 | Transition rejected records PhaseTransition/Rejected | US-002 |
| AC-002.3 | Kill switch disables in ecc-workflow | US-002 |
| AC-002.4 | Missing DB = fire-and-forget | US-002 |
| AC-002.5 | Session ID = "workflow-<feature>" in ecc-workflow | US-002 |
| AC-002.6 | No state.json = no metric recorded | US-002 |
| AC-003.1 | subagent:start:log records AgentSpawn/Success | US-003 |
| AC-003.2 | subagent:stop:log failure records AgentSpawn/Failure | US-003 |
| AC-003.3 | $.retry_count (u32) parsed if present | US-003 |
| AC-003.4 | Kill switch disables agent metrics | US-003 |
| AC-003.5 | Failure detection: $.error non-null OR $.exit_code != 0 | US-003 |
| AC-003.6 | No recognized fields = Success/None defaults | US-003 |
| AC-004.1 | Quality gate pass records CommitGate/Passed | US-004 |
| AC-004.2 | Quality gate fail records CommitGate/Failure + gates_failed | US-004 |
| AC-004.3 | Kill switch disables commit gate metrics | US-004 |
| AC-004.4a | Formatter gates via quality.rs hook handler | US-004 |
| AC-004.4b | Build/test/lint gates via ecc metrics record-gate CLI | US-004 |
| AC-005.1 | ECC_METRICS_DISABLED=1 = immediate return | US-005 |
| AC-005.2 | Kill switch active: summary still reads data | US-005 |
| AC-005.3 | Unset = normal recording | US-005 |
| AC-006.1 | Catchup shows four rates as percentages | US-006 |
| AC-006.2 | No events = "No harness metrics recorded" | US-006 |
| AC-006.3 | DB unavailable = skip silently | US-006 |
| AC-006.4 | N/A for metric types with no events | US-006 |
| AC-006.5 | Catchup uses ecc metrics summary --session --json | US-006 |
| AC-007.1 | --trend shows current vs previous with deltas | US-007 |
| AC-007.2 | No previous events = "N/A" | US-007 |
| AC-007.3 | Positive delta shows "+" prefix | US-007 |
| AC-007.4 | --trend + --session = error | US-007 |
| AC-007.5 | Both periods N/A = "N/A" delta | US-007 |
| AC-008.1 | vs. Target column in summary | US-008 |
| AC-008.2 | Below SLO flagged with [!] prefix | US-008 |
| AC-008.3 | SLO targets as domain constants, configurable | US-008 |
| AC-008.4 | Default SLOs: hook>=99%, gate<=5%, agent>=80%, commit>=95% | US-008 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 62 → ~78 | PASS | All phantom payloads defined, warning indicator specified |
| Edge Cases | 55 → ~75 | PASS | Session ID, concurrent writes, missing state.json covered |
| Scope Creep Risk | 82 | PASS | Non-requirements crisp, scope well-bounded |
| Dependency Gaps | 72 → ~78 | PASS | US-006 deps corrected, ecc-workflow note added |
| Testability | 58 → ~78 | PASS | Instrumentation points defined, integration paths specified |
| Decision Completeness | 65 → ~80 | PASS | 13 decisions total, session ID/timestamps/trends/SLOs covered |
| Rollback & Failure | 78 | PASS | Fire-and-forget, Option wrapper, kill switch |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-06-harness-reliability-metrics/spec.md | Full spec with Phase Summary |
