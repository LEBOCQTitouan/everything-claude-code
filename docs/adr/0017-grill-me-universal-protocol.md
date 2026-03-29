# 0017. Grill-Me as Universal Questioning Protocol

Date: 2026-03-23

## Status

Partially superseded by ADR-0033 (2026-03-29). The 25-question cap is removed and depth profiles are introduced. The 5-stage structure and AskUserQuestion enforcement remain in effect.

## Context

The ECC codebase had three independent questioning systems serving the same purpose (challenge an idea before proceeding):

1. **Standalone grill-me skill**: 5-stage adversarial interview with branch tracking, vocabulary detection, and transcript output.
2. **Spec-pipeline-shared inline rules**: A different fixed-question clarification model with recommended answers and "spec it" shortcut. Named "Grill-Me Interview" but with different semantics.
3. **Backlog-curator ad-hoc questions**: 1-3 batched questions with no formal protocol.

The "grill-me" name was overloaded to mean two fundamentally different things. Interview rules were duplicated across 4 files (spec-pipeline-shared + 3 spec commands). The backlog challenge was completely disconnected from the formal protocol.

## Decision

Unify all three systems into a single universal grill-me protocol. The grill-me skill becomes the canonical questioning mechanism for all contexts:

- **Standalone mode** (default): All 5 stages, no recommended answers, no shortcuts
- **Spec-mode**: Recommended answers as first option with "(Recommended)", "spec it" shortcut
- **Backlog-mode**: Max 3 stages, max 2 questions per stage for LOW/MEDIUM scope; escalates to full 5 stages for HIGH/EPIC

The protocol uses stage-by-stage AskUserQuestion with challenge loops, cross-stage mutation, a 25-question cap, and stage-reopen-exactly-once limit.

Enforcement is via a `grill-me-gate.sh` Stop hook that checks for grill-me decision markers in the spec output.

## Consequences

**Positive:**

- Single source of truth for all questioning behavior
- No naming collision — "grill-me" means one thing everywhere
- Interview rules maintained in one place (grill-me skill), not duplicated across 4 files
- Backlog gets formal protocol instead of ad-hoc questions
- Hook enforcement prevents skipping the interview
- Spec-pipeline-shared decomposed from 7-concern grab-bag to focused utility

**Negative:**

- Grill-me skill is now larger (handles 3 modes)
- Backlog flow may feel heavier for simple ideas (mitigated by backlog-mode's lighter config)
- Any grill-me bug affects all three contexts simultaneously (mitigated by git tag `pre-bl-061` rollback)
