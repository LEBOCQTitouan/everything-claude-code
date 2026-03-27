# Solution: Display Full Artifacts Inline in Terminal (BL-062)

## Spec Reference
Concern: refactor, Feature: BL-062 Display full artifacts inline in terminal

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `commands/spec-dev.md` | Modify | Add inline artifact display to Phase 10 (reference) | US-001: AC-001.1,4,5,6,7; US-004: AC-004.1,2 |
| 2 | `commands/spec-fix.md` | Modify | Add inline artifact display to Phase 9 | US-001: AC-001.2; US-004: AC-004.1,2 |
| 3 | `commands/spec-refactor.md` | Modify | Add inline artifact display to Phase 9 | US-001: AC-001.3; US-004: AC-004.1,2 |
| 4 | `commands/design.md` | Modify | Add inline artifact display to Phase 11 | US-002: AC-002.1,2,3; US-004: AC-004.1,2 |
| 5 | `commands/implement.md` | Modify | Add inline artifact display to Phase 8 | US-003: AC-003.1,2,3; US-004: AC-004.1,2 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | grep | spec-dev has "Full Artifact Display" | AC-001.1, AC-004.1 | `grep -c "Full Artifact Display" commands/spec-dev.md` | >= 1 |
| PC-002 | grep | spec-dev reads spec_path | AC-001.6 | `grep -c "artifacts.spec_path" commands/spec-dev.md` | >= 1 |
| PC-003 | grep | spec-dev has fallback | AC-001.7 | `grep -ci "warning.*skip\|skip.*summary" commands/spec-dev.md` | >= 1 |
| PC-004 | grep | spec-dev preserves tables | AC-001.4 | `grep -c "Grill-Me Decisions" commands/spec-dev.md` | >= 1 |
| PC-005 | grep | spec-dev file path after tables | AC-001.5 | `grep -c "future access" commands/spec-dev.md` | >= 1 |
| PC-006 | grep | spec-fix has "Full Artifact Display" | AC-001.2, AC-004.1 | `grep -c "Full Artifact Display" commands/spec-fix.md` | >= 1 |
| PC-007 | grep | spec-refactor has "Full Artifact Display" | AC-001.3, AC-004.1 | `grep -c "Full Artifact Display" commands/spec-refactor.md` | >= 1 |
| PC-008 | grep | design.md has "Full Artifact Display" | AC-002.1, AC-004.1 | `grep -c "Full Artifact Display" commands/design.md` | >= 1 |
| PC-009 | grep | design.md reads design_path | AC-002.2 | `grep -c "artifacts.design_path" commands/design.md` | >= 1 |
| PC-010 | grep | design.md has fallback | AC-002.3 | `grep -ci "warning.*skip\|skip.*summary" commands/design.md` | >= 1 |
| PC-011 | grep | implement.md has "Full Artifact Display" | AC-003.1, AC-004.1 | `grep -c "Full Artifact Display" commands/implement.md` | >= 1 |
| PC-012 | grep | implement.md reads tasks_path | AC-003.2 | `grep -c "artifacts.tasks_path" commands/implement.md` | >= 1 |
| PC-013 | grep | implement.md has fallback | AC-003.3 | `grep -ci "warning.*skip\|skip.*summary" commands/implement.md` | >= 1 |
| PC-014 | grep | All 5 use consistent template | AC-004.2 | `grep -l "Read the full artifact" commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md \| wc -l` | 5 |
| PC-015 | build | Rust build passes | All | `cargo build` | exit 0 |
| PC-016 | build | Rust tests pass | All | `cargo test` | exit 0 |
| PC-017 | build | ecc validate passes | All | `cargo run -- validate commands` | exit 0 |
| PC-018 | grep | spec-fix file path reference | AC-001.5 | `grep -c "future access" commands/spec-fix.md` | >= 1 |
| PC-019 | grep | spec-refactor file path reference | AC-001.5 | `grep -c "future access" commands/spec-refactor.md` | >= 1 |
| PC-020 | grep | design.md file path reference | AC-001.5 | `grep -c "future access" commands/design.md` | >= 1 |
| PC-021 | grep | implement.md file path reference | AC-001.5 | `grep -c "future access" commands/implement.md` | >= 1 |

### Coverage Check

All 15 ACs covered:

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-006 |
| AC-001.3 | PC-007 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005, PC-018, PC-019, PC-020, PC-021 |
| AC-001.6 | PC-002 |
| AC-001.7 | PC-003 |
| AC-002.1 | PC-008 |
| AC-002.2 | PC-009 |
| AC-002.3 | PC-010 |
| AC-003.1 | PC-011 |
| AC-003.2 | PC-012 |
| AC-003.3 | PC-013 |
| AC-004.1 | PC-001, PC-006, PC-007, PC-008, PC-011 |
| AC-004.2 | PC-014 |

Uncovered ACs: **none**.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | None | N/A | N/A | No E2E boundaries affected | — | — |

### E2E Activation Rules
No E2E tests needed.

## Test Strategy

TDD order:
1. **PC-001–005** (Wave 1: spec-dev — reference implementation)
2. **PC-006, PC-018** (Wave 2: spec-fix)
3. **PC-007, PC-019** (Wave 2: spec-refactor — parallel with spec-fix)
4. **PC-008–010, PC-020** (Wave 3: design.md)
5. **PC-011–013, PC-021** (Wave 4: implement.md)
6. **PC-014–017** (Wave 5: consistency + final gate)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Minor | Add entry | "Display full spec/design/implement artifacts inline in terminal (BL-062)" | All US |

No ADRs needed.

## SOLID Assessment
**PASS** — 0 findings.

## Robert's Oath Check
**CLEAN** — 0 warnings.

## Security Notes
**CLEAR** — 0 findings.

## Rollback Plan
1. Revert `commands/implement.md`
2. Revert `commands/design.md`
3. Revert `commands/spec-refactor.md`
4. Revert `commands/spec-fix.md`
5. Revert `commands/spec-dev.md`

Each revert independent.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS (round 2) | Added PC-018–021 for file path reference |
| Order | PASS | spec-dev first, then replicate pattern |
| Fragility | PASS (advisory) | Use anchor text, not line numbers |
| Rollback | PASS | Independent file reverts |
| Architecture | PASS | Content layer only |
| Blast radius | PASS | 5 files, ~15 lines each, under 800-line limit |
| Missing PCs | PASS (round 2) | Added 4 file path PCs + reconciled count |
| Doc plan | PASS | CHANGELOG entry sufficient |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | commands/spec-dev.md | Modify | US-001, US-004 |
| 2 | commands/spec-fix.md | Modify | US-001, US-004 |
| 3 | commands/spec-refactor.md | Modify | US-001, US-004 |
| 4 | commands/design.md | Modify | US-002, US-004 |
| 5 | commands/implement.md | Modify | US-003, US-004 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-inline-artifact-display/spec.md | Full spec + phase summary |
| docs/specs/2026-03-26-inline-artifact-display/design.md | Full design + phase summary |
