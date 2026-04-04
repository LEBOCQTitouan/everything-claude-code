# Spec: Cost and Token Tracking (BL-096)

## Problem Statement

ECC has zero visibility into token consumption. Without measurement, model routing optimizations (BL-094) and thinking budget tuning (BL-095) cannot be validated. A partial implementation exists (Stop hook writing JSONL) but lacks: (1) queryable storage, (2) agent attribution, (3) thinking token separation, (4) CLI analysis commands, (5) port abstraction for testability. The web radar places cost tracking in the "Adopt" ring with 5/5 strategic fit.

## Research Summary

- **toktrack** (Rust CLI) reads Claude Code's native JSONL session logs, aggregates costs daily/weekly/monthly with immutable daily caches — most relevant architectural reference
- **llm-pricing** crate provides Rust API for model pricing tables (cost per 1M tokens) — potential dependency for rate lookups
- **Anthropic Usage API** provides daily usage reports filtered by model and token type — external validation source
- **Claude Code emits OpenTelemetry-format metrics** that can be pushed to Prometheus/Grafana — team-scale complement to local tracking
- **Pitfall**: Claude Code's 30-day session retention deletes raw JSONL — must copy data to own store
- **Pitfall**: Concurrent writers need explicit flock or per-session files to avoid interleaving (SQLite WAL mode solves this)
- **Pitfall**: SIMD-json requires padding bytes — not drop-in for serde_json

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | SQLite at `~/.ecc/cost/cost.db` instead of JSONL | Indexed queries, GROUP BY aggregation, concurrent WAL writes, consistent with BL-092 pattern | Yes |
| 2 | Separate cost.db from logs ecc.db | Bounded context isolation — independent schema evolution, retention policies, and access patterns | Yes |
| 3 | Stop event instead of PostToolUse | Stop event carries usage.input_tokens/output_tokens in payload; PostToolUse has no billing data | Yes |
| 4 | Extend existing cost_tracker hook | Avoid duplication; refactor to use CostStore port + domain CostCalculator | No |
| 5 | Use `agent_type` field for agent attribution | Available in Stop payload; defaults to "main" for top-level sessions | No |
| 6 | Separate thinking_tokens with distinct pricing | Extended thinking may have different rates; more accurate cost estimates | No |
| 7 | Include `ecc cost prune` in scope | Follows BL-092 pattern; prevents unbounded growth | No |
| 8 | Budget alerts/thresholds out of scope | Defer BudgetThresholdReached domain event to future iteration | No |
| 9 | Real-time dashboard/TUI out of scope | CLI summary commands only for v1 | No |
| 10 | Use f64 for Money (not fixed-point decimal) | This is analytics, not billing; 6 decimal precision is sufficient for cost estimates | No |

## User Stories

### US-001: Cost Domain Model

**As a** developer, **I want** a well-typed cost domain model with value objects, **so that** cost calculations are accurate, testable, and free of primitive obsession.

#### Acceptance Criteria

- AC-001.1: Given a new `cost` module in `ecc-domain`, when compiled, then it exports `ModelId` (non-empty string), `TokenCount` (u64, >= 0), `Money` (f64 USD, 6 decimal precision — analytics not billing), `CostRate` (input/output/thinking rates per MTok, all non-negative), `RecordId` (i64) value objects with validated construction that reject invalid inputs
- AC-001.2: Given a `TokenUsageRecord` entity, when created, then it contains: record_id, session_id, timestamp, model, input_tokens, output_tokens, thinking_tokens, estimated_cost, agent_type, parent_session_id
- AC-001.3: Given a `CostCalculator` domain service with a hardcoded `PricingTable` struct (haiku: $1/$5, sonnet: $3/$15, opus: $15/$75 per MTok input/output, thinking at output rate), when estimating cost for a known model, then it applies the model's rates for input, output, and thinking tokens separately
- AC-001.4: Given a `CostCalculator`, when summarizing a collection of records, then it produces a `CostSummary` with total cost, total tokens, and per-model `CostBreakdown` entries
- AC-001.5: Given an unknown model ID, when estimating cost, then it falls back to a default rate and logs a warning

#### Dependencies

- Depends on: none

### US-002: CostStore Port and SQLite Adapter

**As a** developer, **I want** a `CostStore` driven port with SQLite adapter, **so that** cost data is persisted with indexed queries and concurrent-safe writes.

#### Acceptance Criteria

- AC-002.1: Given a `CostStore` trait in `ecc-ports`, when defined, then it has methods: append, query, summary, prune, export
- AC-002.2: Given a `SqliteCostStore` in `ecc-infra`, when initialized, then it creates `~/.ecc/cost/cost.db` with `token_usage` table and indexes on timestamp, session_id, model
- AC-002.3: Given 10 concurrent threads each appending 100 records via WAL mode, when all complete, then SELECT COUNT(*) = 1000 and all records deserialize correctly (stress test with `#[ignore]`)
- AC-002.4: Given a `CostQuery` with date range filter, when querying, then only matching records are returned (indexed scan, not full table)
- AC-002.5: Given an `InMemoryCostStore` in `ecc-test-support`, when used in tests, then it satisfies the same trait contract

#### Dependencies

- Depends on: US-001

### US-003: Refactor Existing Cost Tracker Hook

**As a** developer, **I want** the existing `cost_tracker` hook refactored to use domain types and CostStore port, **so that** cost data flows through proper hexagonal boundaries.

#### Acceptance Criteria

- AC-003.1: Given the existing `cost_tracker` handler, when refactored, then it constructs a `TokenUsageRecord` using `CostCalculator::estimate()` from the domain layer
- AC-003.2: Given the refactored hook, when it fires on Stop event, then it calls `CostStore::append()` instead of raw `FileSystem::write`
- AC-003.3: Given a Stop event with `agent_type` field, when the hook fires, then agent_type is captured in the record (defaulting to "main" if absent)
- AC-003.4: Given a Stop event with `usage.thinking_tokens` field, when the hook fires, then thinking tokens are stored separately from output tokens
- AC-003.5: Given the `HookPorts` struct, when modified, then it includes an optional `cost_store: Option<&dyn CostStore>` field
- AC-003.6: Given the existing `estimate_cost()` in app layer, when refactored, then the business logic moves to `CostCalculator` in domain layer (DDD violation fix)
- AC-003.7: Given CostStore is None (not yet wired), when the hook fires, then it falls back to the existing JSONL write behavior (backward compatibility during transition)

#### Dependencies

- Depends on: US-002

### US-004: CLI Cost Commands

**As a** user, **I want** `ecc cost` CLI subcommands, **so that** I can analyze my token spending by model, agent, and time period.

#### Acceptance Criteria

- AC-004.1: Given `ecc cost summary [--since 7d] [--model M]`, when run, then it displays aggregated cost breakdown by model with total cost in USD
- AC-004.2: Given `ecc cost breakdown --by agent|model [--since 7d]`, when run, then it displays per-agent or per-model breakdown with token counts and cost
- AC-004.3: Given `ecc cost compare --before DATE --after DATE`, when run, then it displays side-by-side cost comparison between two date ranges
- AC-004.4: Given `ecc cost export --format json|csv [--since 7d]`, when run, then it outputs cost data in the requested format to stdout
- AC-004.5: Given `ecc cost prune [--older-than 90d]`, when run, then records older than the threshold are deleted and count is reported
- AC-004.6: Given an empty database (no records), when any cost command runs, then it displays a helpful message instead of an error

#### Dependencies

- Depends on: US-002

### US-005: Legacy Data Migration

**As a** user, **I want** `ecc cost migrate` to import my existing JSONL cost data, **so that** historical data is preserved in the new SQLite store.

#### Acceptance Criteria

- AC-005.1: Given `~/.claude/metrics/costs.jsonl` exists, when `ecc cost migrate` runs, then all valid records are imported into SQLite with correct field mapping
- AC-005.2: Given malformed lines in the JSONL file, when migrating, then they are skipped with a warning (not a hard failure)
- AC-005.3: Given migration already run, when run again, then it is idempotent — duplicates detected by (timestamp, session_id, model) composite uniqueness, skipped with count reported
- AC-005.4: Given the JSONL file does not exist, when `ecc cost migrate` runs, then it reports "No legacy data found" and exits cleanly

#### Dependencies

- Depends on: US-002

### US-006: Orchestrator Agent Cost Reporting

**As a** user, **I want** orchestrator agents to report per-subagent cost summaries, **so that** I can see which agents in a pipeline are most expensive.

#### Acceptance Criteria

- AC-006.1: Given the `audit-orchestrator` agent definition, when updated, then it includes instructions to query CostStore for the current session's agent breakdown after all subagents complete
- AC-006.2: Given the `doc-orchestrator` agent definition, when updated, then it includes the same cost reporting instructions
- AC-006.3: Given the cost summary output, when displayed, then it shows per-agent token counts and estimated cost in a table format

#### Dependencies

- Depends on: US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain` | Domain | New `cost` module: value objects, CostCalculator service, CostSummary projection |
| `ecc-ports` | Ports (driven) | New `CostStore` trait, `CostQuery`, `CostStoreError` |
| `ecc-infra` | Adapter (driven) | New `SqliteCostStore` adapter |
| `ecc-app` | Application | New `cost_mgmt` use case module; refactored `cost_tracker` hook handler |
| `ecc-cli` | Adapter (driving) | New `ecc cost` subcommand with 6 sub-subcommands |
| `ecc-test-support` | Test | New `InMemoryCostStore` test double |
| `agents/audit-orchestrator.md` | Agent def | Cost reporting instructions |
| `agents/doc-orchestrator.md` | Agent def | Cost reporting instructions |

## Constraints

- Domain layer (`ecc-domain`) must have zero I/O imports — pure business logic only
- CostStore port returns domain types only — no SQLite row types escape the boundary
- Existing `cost_tracker` hook must continue working during migration (backward compatible)
- Model pricing rates must be centralized in CostCalculator, not scattered across hooks
- SQLite WAL mode required for concurrent session writes
- TEST-005 audit finding: existing tracking.rs has 342 untested lines — this refactor must add tests

## Non-Requirements

- Budget alerts / threshold notifications (future iteration)
- Real-time dashboard / TUI cost view
- Anthropic API integration for usage verification
- Per-tool-call granularity (Stop event gives per-response only)
- Cache hit percentage tracking (not available in Stop payload)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| CostStore (new driven port) | New | Integration tests with real SQLite |
| SqliteCostStore (new adapter) | New | E2E: CLI -> app -> infra -> SQLite |
| cost_tracker hook (refactored) | Modified | Existing hook behavior preserved, new storage backend |
| ecc cost CLI (new driving adapter) | New | E2E: argument parsing -> use case -> port -> adapter |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI commands | CLAUDE.md | CLI Commands section | Add `ecc cost` subcommands |
| New bounded context | docs/domain/bounded-contexts.md | Cost context | Add cost bounded context definition |
| Architecture change | docs/ARCHITECTURE.md | Crate map | Add cost module to crate descriptions |
| ADR | docs/adr/ | 3 new ADRs | SQLite over JSONL, separate DB, Stop event |
| Module summary | docs/MODULE-SUMMARIES.md | Per-crate | Update affected crate summaries |

## Open Questions

None — all questions resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Storage format: SQLite or JSONL? | SQLite at `~/.ecc/cost/cost.db` | Recommended |
| 2 | Extend existing hook or create new? | Extend existing cost_tracker | Recommended |
| 3 | Stop event or PostToolUse? | Keep Stop event (has billing data) | Recommended |
| 4 | What is out of scope? | Budget alerts, real-time dashboard | User |
| 5 | Agent name: required or optional? | Use agent_type from Stop payload, required, default "main" | User + Recommended |
| 6 | Thinking tokens: separate or combined? | Separate field with distinct pricing rates | Recommended |
| 7 | Include pruning? | Yes, `ecc cost prune` in scope | Recommended |
| 8 | Which ADRs? | SQLite over JSONL, separate cost.db, Stop event | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Cost Domain Model | 5 | none |
| US-002 | CostStore Port and SQLite Adapter | 5 | US-001 |
| US-003 | Refactor Existing Cost Tracker Hook | 7 | US-002 |
| US-004 | CLI Cost Commands | 6 | US-002 |
| US-005 | Legacy Data Migration | 4 | US-002 |
| US-006 | Orchestrator Agent Cost Reporting | 3 | US-003, US-004 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Value objects with validated construction | US-001 |
| AC-001.2 | TokenUsageRecord entity fields | US-001 |
| AC-001.3 | CostCalculator with hardcoded PricingTable | US-001 |
| AC-001.4 | CostSummary and CostBreakdown aggregation | US-001 |
| AC-001.5 | Unknown model fallback with warning | US-001 |
| AC-002.1 | CostStore trait methods | US-002 |
| AC-002.2 | SQLite schema with indexes | US-002 |
| AC-002.3 | Concurrent write stress test (10x100) | US-002 |
| AC-002.4 | Date range indexed query | US-002 |
| AC-002.5 | InMemoryCostStore test double | US-002 |
| AC-003.1 | Hook uses CostCalculator::estimate() | US-003 |
| AC-003.2 | Hook calls CostStore::append() | US-003 |
| AC-003.3 | Agent type captured (default "main") | US-003 |
| AC-003.4 | Thinking tokens stored separately | US-003 |
| AC-003.5 | HookPorts gets Optional CostStore | US-003 |
| AC-003.6 | estimate_cost() moves to domain (DDD fix) | US-003 |
| AC-003.7 | Fallback to JSONL when CostStore is None | US-003 |
| AC-004.1 | ecc cost summary command | US-004 |
| AC-004.2 | ecc cost breakdown --by agent/model | US-004 |
| AC-004.3 | ecc cost compare date ranges | US-004 |
| AC-004.4 | ecc cost export json/csv | US-004 |
| AC-004.5 | ecc cost prune older-than | US-004 |
| AC-004.6 | Empty database graceful message | US-004 |
| AC-005.1 | JSONL import to SQLite | US-005 |
| AC-005.2 | Malformed lines skipped with warning | US-005 |
| AC-005.3 | Idempotent migration (composite key) | US-005 |
| AC-005.4 | Missing JSONL graceful exit | US-005 |
| AC-006.1 | audit-orchestrator cost reporting | US-006 |
| AC-006.2 | doc-orchestrator cost reporting | US-006 |
| AC-006.3 | Per-agent cost table format | US-006 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Ambiguity | 72 | 86 | PASS | Value object invariants and pricing source now explicit |
| Edge Cases | 65 | 80 | PASS | Concurrent write oracle, idempotency mechanism defined |
| Scope | 78 | 80 | PASS | Well-bounded with clear non-requirements |
| Dependencies | 82 | 82 | PASS | Correct DAG, no deadlocks |
| Testability | 70 | 84 | PASS | All ACs now automatable |
| Decisions | 85 | 85 | PASS | 10 decisions with rationale |
| Rollback | 88 | 88 | PASS | Additive changes, JSONL fallback |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-04-cost-token-tracking/spec.md | Full spec with Phase Summary |
