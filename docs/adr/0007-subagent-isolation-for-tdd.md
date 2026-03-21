# 7. Subagent isolation for TDD PC execution

Date: 2026-03-21

## Status

Accepted

## Context

Long `/implement` sessions with 10+ Pass Conditions suffer from context window degradation. By PC-8 or PC-9, the main context is heavily consumed with implementation reasoning from prior PCs, causing quality drops in the "last 20% of context" zone. The GSD framework (26K+ GitHub stars) demonstrates that giving each execution unit a fresh context window eliminates this "context rot."

Two design questions required decisions:
1. Who runs the regression suite after each PC — the subagent or the parent?
2. Should PCs be dispatched in parallel or sequentially?

## Decision

Each PC's RED-GREEN-REFACTOR cycle executes in a forked `tdd-executor` subagent with fresh context. The parent orchestrator owns regression verification (runs all prior PC commands after each subagent completes). PCs are dispatched sequentially — parallel execution is deferred to BL-032.

Key design choices:
- **Parent owns regression**: Subagent runs only its own PC command. Parent runs all prior PCs after subagent returns. This keeps the subagent's context minimal and places regression detection at the orchestration level where it belongs.
- **Sequential dispatch only**: TDD progression requires prior PC implementations to exist on disk. Parallelism would break this dependency chain.
- **Hard stop on regression**: Unlike the previous inline auto-fix behavior, the parent now stops and reports. The subagent lacks context of other PCs' code, and auto-fixing in the parent would re-pollute context, defeating the isolation purpose.

## Consequences

### Positive
- Each PC gets a fresh ~200K context window — no quality degradation after PC-010+
- Parent context grows only by subagent summaries (~200 tokens/PC), not full implementation reasoning
- Git history unchanged — same atomic commits (test, implementation, refactor)
- Clean separation of concerns: orchestration (parent) vs execution (subagent)
- BL-032 (parallel waves) can extend this design without modifying the tdd-executor agent

### Negative
- Added latency per PC (subagent spawn + file reads for context brief)
- Regressions that were previously auto-fixed now cause a hard stop requiring user intervention
- Subagent cannot detect cross-PC conflicts during execution — conflicts surface only at parent regression check
- Prompt-level enforcement of restrictions (subagent MUST NOT modify state.json) — not technically enforced
