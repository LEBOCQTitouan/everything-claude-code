# ADR-0064: Party Command Design Decisions

## Status

Accepted

<!-- tags: content-only|zero Rust|no Rust; bmad-role-prefix|flat-agents; sequential|Sequential; ephemeral|Ephemeral -->

## Context

BL-144 introduces `/party`, a multi-agent round-table discussion command. Design required decisions on four axes: implementation scope (Rust vs content-only), agent naming and layout, execution model (sequential vs parallel), and panel lifecycle (ephemeral vs persisted teams).

## Decision

### Decision 1: Content-only — zero Rust changes

`/party` is implemented as a command YAML + agents + skills only. No Rust code is added or modified. The ECC CLI validates agent/command YAML structure; `/party` fits within those existing validation rules without requiring new CLI subcommands or domain types.

### Decision 2: BMAD agents use flat `bmad-` prefix (not subdirectory)

BMAD role agents (`bmad-pm`, `bmad-architect`, `bmad-dev`, `bmad-qa`, `bmad-security`) are placed directly under `agents/` with a `bmad-` prefix, not under `agents/bmad/`. This matches ECC's flat agents convention (no subdirectory nesting). The prefix provides sufficient namespace isolation for `ecc validate agents` to lint them correctly.

### Decision 3: Sequential-only for v1 (parallel deferred)

`party-coordinator` dispatches panelists sequentially. Parallel dispatch (multiple Agent tool calls in one message) is deferred to v2. Sequential ordering provides deterministic output that is easier to review and debug, avoids context-window contention between simultaneous agents, and keeps the coordinator logic simple. Parallel mode can be added later as an opt-in flag.

### Decision 4: Ephemeral panels (not persisted teams)

Each `/party` invocation assembles a fresh panel from the declared role list. Panels are not saved as persistent team configurations in `teams/`. Output is persisted to `docs/party/<slug>/` but the panel composition is not. This keeps `/party` stateless and avoids coupling it to the teams subsystem.

## Consequences

**Positive**: Zero Rust delta means no compilation risk and no domain model churn. Flat naming stays consistent with existing agent conventions. Sequential dispatch is auditable and debuggable. Ephemeral panels avoid state-management complexity.

**Negative**: No parallel speedup in v1 — large panels are slower. Flat naming requires consumers to remember the `bmad-` prefix convention rather than browsing a directory. Ephemeral panels mean recurring discussion setups must be re-specified each invocation (workaround: shell alias or command wrapper).

**Deferred**: Parallel dispatch (v2), saved panel presets (v3), streaming panel output to UI.
