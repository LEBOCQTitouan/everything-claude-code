---
name: pc-evaluation
description: Post-PC self-evaluation rubric for /implement TDD loop. Defines 3 dimensions (AC satisfaction, regression heuristics, spec achievability) with PASS/WARN/FAIL verdicts and conditional trigger rules.
origin: ECC
---

# PC Evaluation Rubric

Evaluates whether a completed Pass Condition truly advances the spec, beyond just tests passing.

## Trigger Rules

Evaluation is **conditional** — it runs only when:
1. `fix_round_count > 0` — the PC needed fix rounds (higher risk)
2. PC type is `integration` or `e2e` — crosses boundaries
3. Last PC in a wave — wave boundary checkpoint

Clean unit PCs that pass first try are **skipped** (logged as "SKIPPED (clean unit)").

## Dimensions

### 1. AC Satisfaction

Does the PC output actually satisfy the Acceptance Criteria it claims to verify?

| Verdict | Criteria |
|---------|----------|
| **PASS** | Files changed match the AC's expected behavior; test names cover the AC's scenario; green_result confirms the AC's "then" clause |
| **WARN** | Tests pass but files changed are minimal/empty or test assertions are trivially satisfied (e.g., assert!(true)) |
| **FAIL** | No files changed despite a non-lint PC; test names don't relate to the AC; or green_result contradicts the AC's intent |

### 2. Regression Heuristics

Did the PC introduce structural damage not caught by tests?

| Verdict | Criteria |
|---------|----------|
| **PASS** | Files modified are within the PC's "Files to Modify" list; no public API signatures changed unexpectedly |
| **WARN** | Files modified outside the "Files to Modify" list (check if legitimate — e.g., Cargo.toml for deps) |
| **FAIL** | Public exports removed or renamed in files not listed in the PC's scope; trait impl removed |

### 3. Spec Achievability

Is the spec still achievable given what was learned from this PC?

| Verdict | Criteria |
|---------|----------|
| **PASS** | Remaining PCs appear achievable; no fundamental constraints discovered |
| **WARN** | A PC assumption was invalidated but a workaround exists; or a later PC's file overlap increased |
| **FAIL** | A dependency is missing or behaves differently than designed; remaining PCs structurally depend on something that failed |

## Escalation Rules

- **WARN**: Logged to tasks.md with `eval@<timestamp>` status. Pipeline continues.
- **FAIL**: Triggers `AskUserQuestion` with options: "Re-dispatch PC with guidance", "Accept as-is", "Pause and revise spec", "Abort"
- **3 consecutive WARNs** across PCs: Auto-escalate to FAIL behavior (cumulative signal of drift)

## Integration

Evaluation runs **after** the fix-round budget resolves (PC passes or is skipped). The pc-evaluator agent receives: PC result, AC text from spec, files changed, prior PC results table.

Output is structured: `{ac_satisfied: PASS|WARN|FAIL, regressions: PASS|WARN|FAIL, achievability: PASS|WARN|FAIL, rationale: string}`
