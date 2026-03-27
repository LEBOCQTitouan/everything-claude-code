---
id: BL-080
title: TDD fix-loop budget cap at 2 rounds
scope: LOW
target: direct edit
status: open
created: 2026-03-27
origin: Stripe Minions blog post — CI round budget pattern
---

# BL-080: TDD Fix-Loop Budget Cap

## Problem

ECC's /implement TDD loop has no cap on test-fix iterations. A failing test can trigger unbounded retry cycles, burning tokens with diminishing returns. Stripe found that "at most two rounds of CI" balances speed and efficiency, noting "diminishing marginal returns for an LLM to run many rounds."

## Proposal

Add a max retry budget to the tdd-executor agent and /implement's TDD loop:

- **Round 1**: Run tests → if fail, analyze error and fix → commit fix
- **Round 2**: Re-run tests → if fail, analyze and fix again → commit fix
- **Round 3 (budget exceeded)**: If still failing, stop and report to the user with diagnostics instead of retrying

Implementation:
- Add `max_fix_rounds: 2` to tdd-executor agent behavior
- Track round count in the TDD loop
- On budget exceeded: output structured failure report (test name, error, attempted fixes, suggested next steps)
- User can override with explicit "keep trying" instruction

## Ready-to-Paste Prompt

```
Direct edit to tdd-executor agent and /implement command.

Add a fix-loop budget cap of 2 rounds to the TDD executor:
- Track fix attempt count per pass condition
- After 2 failed fix attempts, stop retrying and report failure with diagnostics
- Include: test name, error output, what was tried, suggested manual steps
- User can explicitly override ("keep trying") to extend the budget

Edit: agents/tdd-executor.md and commands/implement.md
Inspired by Stripe's "at most two rounds of CI" constraint.
```

## Scope Estimate

LOW — behavioral change in two existing files, no new infrastructure.

## Dependencies

None.
