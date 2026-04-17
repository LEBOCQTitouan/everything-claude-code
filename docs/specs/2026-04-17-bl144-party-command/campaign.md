# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Scope boundaries? | 6 items out of scope: no streaming, no GUI, no spec integration, no agent dual-mode, no cross-session state, no subdirectory support | recommended |
| 2 | Execution mode? | Sequential only for v1 — each agent sees prior outputs (threaded context). Parallel deferred to v2. | recommended |
| 3 | Test strategy? | 100% on: command validates (ecc validate commands), BMAD agents validate (ecc validate agents), output file structure. 80% on: agent auto-generation (best-effort, graceful degradation). | recommended |
| 4 | Performance? | No latency requirements — party sessions are interactive, user-initiated. Token budget concern: sequential mode grows linearly with panel size; cap at 8 agents mitigates. | recommended |
| 5 | Security? | No auth, no user data, no external APIs beyond Claude Code harness. Auto-generated agents in .claude/agents/ are project-local and not validated by ecc validate (documented). | recommended |
| 6 | Breaking changes? | None — purely additive. No existing commands, agents, or CLI contracts change. | recommended |
| 7 | Glossary? | Add: party-coordinator (synthesis orchestrator agent), party session (ephemeral multi-agent discussion invoked via /party), BMAD role agent (cross-functional specialist agent for round-table panels). | recommended |
| 8 | ADR needed? | Yes — ADR for /party command design: sequential-only v1, flat bmad- prefix, ephemeral panels not persisted as teams, auto-generation best-effort. | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
