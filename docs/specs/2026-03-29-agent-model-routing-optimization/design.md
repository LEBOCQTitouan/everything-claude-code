# Solution: Agent model routing optimization (BL-094)

## Spec Reference
Concern: refactor, Feature: Agent model routing optimization — downgrade misaligned agents to Sonnet/Haiku (BL-094)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `agents/drift-checker.md` | modify | model: opus → haiku (simple diff detection) | AC-001.1 |
| 2 | `agents/doc-validator.md` | modify | model: opus → sonnet (checklist validation) | AC-001.2 |
| 3 | `agents/web-scout.md` | modify | model: opus → sonnet (dispatch/aggregation) | AC-001.3 |
| 4 | `agents/doc-orchestrator.md` | modify | model: opus → sonnet (dispatch/aggregation) | AC-001.4 |
| 5 | `agents/python-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 6 | `agents/go-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 7 | `agents/rust-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 8 | `agents/typescript-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 9 | `agents/java-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 10 | `agents/kotlin-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 11 | `agents/cpp-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 12 | `agents/csharp-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 13 | `agents/shell-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 14 | `agents/database-reviewer.md` | modify | model: opus → sonnet (code review) | AC-001.5 |
| 15 | `rules/common/performance.md` | modify | Update Model Selection Strategy section | AC-001.6 |
| 16 | `docs/adr/0030-model-routing-policy.md` | create | Document three-tier routing policy | AC-001.7 |
| 17 | `CHANGELOG.md` | modify | Add routing optimization entry | US-001 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | drift-checker model is haiku | AC-001.1 | `grep '^model: haiku' agents/drift-checker.md` | exit 0 |
| PC-002 | unit | doc-validator model is sonnet | AC-001.2 | `grep '^model: sonnet' agents/doc-validator.md` | exit 0 |
| PC-003 | unit | web-scout model is sonnet | AC-001.3 | `grep '^model: sonnet' agents/web-scout.md` | exit 0 |
| PC-004 | unit | doc-orchestrator model is sonnet | AC-001.4 | `grep '^model: sonnet' agents/doc-orchestrator.md` | exit 0 |
| PC-005 | unit | All 10 language reviewers are sonnet | AC-001.5 | `for f in agents/{python,go,rust,typescript,java,kotlin,cpp,csharp,shell,database}-reviewer.md; do grep -q '^model: sonnet' "$f" \|\| exit 1; done` | exit 0 |
| PC-006 | unit | performance.md has three-tier routing | AC-001.6 | `grep -q 'diff-based detection' rules/common/performance.md && grep -q 'Code review' rules/common/performance.md && grep -q 'Architecture decisions' rules/common/performance.md` | exit 0 |
| PC-007 | unit | ADR 0030 exists with Status: Accepted | AC-001.7 | `test -f docs/adr/0030-model-routing-policy.md && grep -q 'Accepted' docs/adr/0030-model-routing-policy.md` | exit 0 |
| PC-008 | integration | ecc validate agents passes | AC-001.8 | `ecc validate agents` | exit 0 |
| PC-009 | unit | 14 justified opus agents unchanged | AC-001.9 | `for f in agents/{code-reviewer,security-reviewer,architect,architect-module,uncle-bob,arch-reviewer,robert,spec-adversary,solution-adversary,planner,requirements-analyst,interviewer,interface-designer,audit-orchestrator}.md; do grep -q '^model: opus' "$f" \|\| exit 1; done` | exit 0 |
| PC-010 | unit | 4 deferred opus agents unchanged | AC-001.10 | `for f in agents/{doc-analyzer,harness-optimizer,evolution-analyst,component-auditor}.md; do grep -q '^model: opus' "$f" \|\| exit 1; done` | exit 0 |
| PC-011 | lint | Zero clippy warnings | all | `cargo clippy -- -D warnings` | exit 0 |
| PC-012 | build | Build succeeds | all | `cargo build` | exit 0 |

### Coverage Check

| AC | Covered by |
|----|------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005 |
| AC-001.6 | PC-006 |
| AC-001.7 | PC-007 |
| AC-001.8 | PC-008 |
| AC-001.9 | PC-009 |
| AC-001.10 | PC-010 |

All ACs covered. Zero uncovered.

### E2E Test Plan

No E2E tests needed — config-only changes to markdown frontmatter.

### E2E Activation Rules

None.

## Test Strategy

TDD order (3 tiers + docs + validation + gate):
1. PC-001: Tier 1 — drift-checker → haiku (single file, lowest risk)
2. PC-002-004: Tier 2 — auditors/orchestrators → sonnet (3 files)
3. PC-005: Tier 3 — 10 language reviewers → sonnet (batch)
4. PC-006-007: Docs — performance.md + ADR
5. PC-008: Validation — ecc validate agents
6. PC-009-010: Guards — verify unchanged agents
7. PC-011-012: Gate — clippy + build

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `rules/common/performance.md` | Rules | Update Model Selection Strategy | Three-tier: Haiku (extraction), Sonnet (review/TDD), Opus (architecture/security) | AC-001.6 |
| 2 | `docs/adr/0030-model-routing-policy.md` | ADR | Create | Status: Accepted. Three-tier model routing per Anthropic guidance. | AC-001.7 |
| 3 | `CHANGELOG.md` | Project | Add entry | Agent model routing optimization: 14 agents re-tiered | US-001 |

## SOLID Assessment

N/A — no code changes. Config-only frontmatter edits. PASS by inspection.

## Robert's Oath Check

CLEAN — follows vendor guidance, small atomic commits per tier, no mess introduced.

## Security Notes

CLEAR — no injection surfaces, no secrets, no auth. Model field is a trusted config value.

## Rollback Plan

Reverse order:
1. Revert CHANGELOG.md
2. Delete docs/adr/0030-model-routing-policy.md
3. Revert rules/common/performance.md
4. Revert 10 language reviewers (agents/*.md model: sonnet → opus)
5. Revert agents/doc-orchestrator.md (model: sonnet → opus)
6. Revert agents/web-scout.md (model: sonnet → opus)
7. Revert agents/doc-validator.md (model: sonnet → opus)
8. Revert agents/drift-checker.md (model: haiku → opus)
