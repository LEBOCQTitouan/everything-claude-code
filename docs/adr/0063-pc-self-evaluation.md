# ADR-0063: Post-PC Self-Evaluation Architecture

## Status

Accepted

## Context

ECC's /implement TDD loop dispatches tdd-executor subagents per Pass Condition, checks test pass/fail, and has a fix-round budget. But there's no explicit evaluation of whether each iteration advances the spec. Tests can pass while the AC isn't truly satisfied. Goose's agent loop includes an explicit evaluate step. Research shows feedback-driven loops with empirical verification yield 17-53% performance gains.

## Decision

Add a **conditional post-PC self-evaluation** step to /implement Phase 3, executed by a new `pc-evaluator` read-only subagent (tools: [Read, Grep, Glob]).

**Architecture:**
- Evaluation is parent-orchestrator-owned, not tdd-executor-owned
- The pc-evaluator agent is dispatched after fix-round budget resolves
- Evaluation rubric defined in `skills/pc-evaluation/SKILL.md` (reusable)
- 3 dimensions: AC satisfaction, regression heuristics, spec achievability

**Conditional triggers** (to control token cost):
- PC fix_round_count > 0 (needed fixes)
- PC type is integration or e2e (crosses boundaries)
- Last PC in a wave (wave boundary checkpoint)
- Clean unit PCs that pass first try are skipped

**Escalation:**
- WARN: logged to tasks.md, pipeline continues
- FAIL: AskUserQuestion (Re-dispatch, Accept, Pause/revise spec, Abort)
- 3 consecutive WARNs auto-escalate to FAIL

## Consequences

**Positive**: Catches AC drift early (before code review). Structured evaluation grounded in observable evidence. Conditional triggers keep token cost manageable. Reusable rubric via skill.

**Negative**: Adds latency per triggered PC (~30s subagent). 3-WARN auto-escalation may produce false positives in early pipeline stages.

**Risk**: Self-evaluation without external ground truth risks confabulation. Mitigated by read-only agent constrained to grep/read evidence, not speculation.
