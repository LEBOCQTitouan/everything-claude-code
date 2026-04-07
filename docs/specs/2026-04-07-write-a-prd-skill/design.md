# Design: BL-012 Write-a-PRD Skill

## Spec Reference
`docs/specs/2026-04-07-write-a-prd-skill/spec.md`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | `skills/write-a-prd/SKILL.md` | New skill file (<500 words) | Skill | US-001/002/003 |
| 2 | `crates/ecc-workflow/src/commands/phase_gate.rs` | Add `"docs/prds/"` to allowlist (line 24) | Adapter | US-004 |

## Pass Conditions

| PC | Type | Command | Expected | Verifies |
|----|------|---------|----------|----------|
| PC-001 | Validate | `ecc validate skills 2>&1 \| tail -3` | exit 0 | AC-001.1, AC-001.2 |
| PC-002 | Content | `awk '/^---/{n++}n==2{p=1;next}p' skills/write-a-prd/SKILL.md \| wc -w` | <500 | AC-001.3 |
| PC-003 | Content | `grep -c "write a prd\|product requirements\|feature spec\|define what we're building" skills/write-a-prd/SKILL.md` | >=4 | AC-001.4 |
| PC-004 | Content | `grep -cE "problem interview\|codebase exploration\|alternatives\|scope\|module sketch\|write PRD" skills/write-a-prd/SKILL.md` | >=5 | AC-002.1 |
| PC-005 | Content | `grep -cE "Problem Statement\|Target Users\|User Stories\|Non-Goals\|Risks\|Module Sketch\|Success Metrics\|Open Questions" skills/write-a-prd/SKILL.md` | >=7 | AC-003.2 |
| PC-006 | Unit | `cargo test -p ecc-workflow -- phase_gate_allows_prds_dir` | PASS | AC-004.1, AC-004.3 |
| PC-007 | Regression | `cargo test -p ecc-workflow -- phase_gate` | all PASS | AC-004.2 |
| PC-008 | Lint | `cargo clippy -p ecc-workflow -- -D warnings` | exit 0 | -- |
| PC-009 | Build | `cargo build -p ecc-workflow` | exit 0 | -- |
| PC-010 | Content | `grep -c "AskUserQuestion" skills/write-a-prd/SKILL.md` | >=1 | AC-002.2 |
| PC-011 | Content | `grep -cE "Read\|Grep\|Glob" skills/write-a-prd/SKILL.md` | >=1 | AC-002.3 |
| PC-012 | Content | `grep -ci "unavailable\|fall back" skills/write-a-prd/SKILL.md` | >=1 | AC-002.4 |
| PC-013 | Content | `grep -c "docs/prds/" skills/write-a-prd/SKILL.md` | >=1 | AC-003.1 |
| PC-014 | Content | `grep -ci "create.*dir\|automatically" skills/write-a-prd/SKILL.md` | >=1 | AC-003.3 |
| PC-015 | Content | `grep -ci "overwrite\|revision\|already exists" skills/write-a-prd/SKILL.md` | >=1 | AC-003.4 |
| PC-016 | Content | `grep -c "None identified" skills/write-a-prd/SKILL.md` | >=1 | AC-003.5 |

## Coverage Check

All 16 ACs covered by at least 1 PC. 16/16.

## Implementation Strategy

### Wave 1: Skill file (PC-001 through PC-005, PC-010 through PC-016)
Write `skills/write-a-prd/SKILL.md` with frontmatter + 6-step flow + template.

### Wave 2: Phase-gate (PC-006 through PC-009)
Add `"docs/prds/"` to allowlist + new test + regression + lint/build.

## E2E Test Plan
No new E2E tests. Existing phase-gate tests cover the gate mechanism.

## Test Strategy
- **Content verification**: grep-based PCs against skill file (Wave 1)
- **Unit test**: new phase-gate test for docs/prds/ allowlist (Wave 2)
- **Regression**: all existing phase-gate tests (Wave 2)

## Doc Update Plan
| Doc | Action | Spec Ref |
|-----|--------|----------|
| CHANGELOG.md | Add entry | US-001 |
| docs/backlog/BL-012 | Mark implemented | -- |

## SOLID Assessment
N/A for skill file. Allowlist change is additive — no SOLID violation.

## Robert's Oath Check
CLEAN — minimal scope, single concern per file.

## Security Notes
CLEAR — PRD files are non-executable project artifacts. Allowlist addition permits writes to a new docs subdirectory during gated phases. No path traversal risk (existing normalization handles this).

## Rollback Plan
Remove skill file + revert 1-line allowlist change. No data migration needed.

## Bounded Contexts Affected
None — skill files are outside the domain model. Phase-gate is a static allowlist, not a bounded context.
