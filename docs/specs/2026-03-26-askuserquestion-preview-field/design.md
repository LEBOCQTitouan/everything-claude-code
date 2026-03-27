# Solution: AskUserQuestion Preview Field for Architecture Comparisons

## Spec Reference
Concern: dev, Feature: Add AskUserQuestion preview field usage to grill-me, /design, /spec-*, configure-ecc, and interviewer for visual architecture comparisons

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/grill-me/SKILL.md` | Modify | Foundation: add preview subsection to AskUserQuestion Enforcement | US-001: AC-001.1-4 |
| 2 | `commands/design.md` | Modify | Add AskUserQuestion to allowed-tools + preview instruction | US-002: AC-002.1, AC-002.3-4 |
| 3 | `agents/interface-designer.md` | Modify | Add preview to Phase 7 User Synthesis | US-002: AC-002.2 |
| 4 | `commands/spec-dev.md` | Modify | Add preview to Phase 6 grill-me | US-003: AC-003.1, AC-003.4 |
| 5 | `commands/spec-fix.md` | Modify | Add preview to Phase 5 Q2 | US-003: AC-003.2, AC-003.4 |
| 6 | `commands/spec-refactor.md` | Modify | Add preview to Phase 5 Q2 | US-003: AC-003.3, AC-003.4 |
| 7 | `skills/configure-ecc/SKILL.md` | Modify | Add preview for single-select + multiSelect exclusion | US-004: AC-004.1, AC-004.3 |
| 8 | `agents/interviewer.md` | Modify | Add preview for visual alternative stages | US-004: AC-004.2 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | grep | grill-me contains "preview" (>=3) | AC-001.1 | `grep -c "preview" skills/grill-me/SKILL.md` | >= 3 |
| PC-002 | grep | grill-me contains "visual alternative" | AC-001.1, AC-001.2 | `grep -c "visual alternative" skills/grill-me/SKILL.md` | >= 1 |
| PC-003 | grep | grill-me contains textual skip rule | AC-001.3 | `grep -ci "MUST NOT.*preview\|skip.*preview" skills/grill-me/SKILL.md` | >= 1 |
| PC-004 | grep | grill-me contains fallback inline rule | AC-001.4 | `grep -ci "fallback.*inline\|inline.*Markdown" skills/grill-me/SKILL.md` | >= 1 |
| PC-005 | grep | design.md frontmatter has AskUserQuestion | AC-002.1 | `head -5 commands/design.md \| grep -c "AskUserQuestion"` | 1 |
| PC-006 | grep | interface-designer contains preview | AC-002.2 | `grep -c "preview" agents/interface-designer.md` | >= 1 |
| PC-007 | grep | design.md body contains preview | AC-002.3 | `grep -c "preview" commands/design.md` | >= 1 |
| PC-008 | grep | design.md single-approach skip | AC-002.4 | `grep -ci "single.*viable\|one.*viable\|only one" commands/design.md` | >= 1 |
| PC-009 | grep | spec-dev contains preview | AC-003.1 | `grep -c "preview" commands/spec-dev.md` | >= 1 |
| PC-010 | grep | spec-fix contains preview | AC-003.2 | `grep -c "preview" commands/spec-fix.md` | >= 1 |
| PC-011 | grep | spec-refactor contains preview | AC-003.3 | `grep -c "preview" commands/spec-refactor.md` | >= 1 |
| PC-012 | grep | spec-dev textual skip instruction | AC-003.4 | `grep -ci "textual\|MUST NOT.*preview" commands/spec-dev.md` | >= 1 |
| PC-013 | grep | configure-ecc contains preview | AC-004.1 | `grep -c "preview" skills/configure-ecc/SKILL.md` | >= 1 |
| PC-014 | grep | interviewer contains preview | AC-004.2 | `grep -c "preview" agents/interviewer.md` | >= 1 |
| PC-015 | grep | configure-ecc multiSelect exclusion | AC-004.3 | `grep -ci "multiSelect.*MUST NOT\|MUST NOT.*preview" skills/configure-ecc/SKILL.md` | >= 1 |
| PC-016 | lint | Markdown lint passes | All | `npm run lint` | exit 0 |
| PC-017 | build | Rust build passes | All | `cargo build` | exit 0 |
| PC-018 | build | Rust tests pass | All | `cargo test` | exit 0 |
| PC-019 | build | ecc validate passes for all file types | All | `cargo run -- validate agents && cargo run -- validate skills && cargo run -- validate commands` | exit 0 |

### Coverage Check

All 15 ACs covered:

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-001, PC-002 |
| AC-001.2 | PC-001, PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-002.1 | PC-005 |
| AC-002.2 | PC-006 |
| AC-002.3 | PC-007 |
| AC-002.4 | PC-008 |
| AC-003.1 | PC-009 |
| AC-003.2 | PC-010 |
| AC-003.3 | PC-011 |
| AC-003.4 | PC-012 |
| AC-004.1 | PC-013 |
| AC-004.2 | PC-014 |
| AC-004.3 | PC-015 |

Uncovered ACs: **none**.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | None | N/A | N/A | No E2E boundaries affected | — | — |

### E2E Activation Rules

No E2E tests need un-ignoring — all changes are content layer.

## Test Strategy

TDD order:
1. **PC-001–004** (Phase 1: grill-me) — foundation skill, must exist before spec-* commands reference it
2. **PC-005–008** (Phase 2: design + interface-designer) — fixes frontmatter gap + preview instruction
3. **PC-009–012** (Phase 3: spec-dev/fix/refactor) — depends on grill-me foundation
4. **PC-013–015** (Phase 4: configure-ecc + interviewer) — independent surfaces
5. **PC-016–019** (Final gate: lint + build + test + validate) — after all content changes

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | Minor | Add entry | "Add AskUserQuestion preview field usage to grill-me, /design, /spec-*, configure-ecc, and interviewer for visual architecture comparisons" | All US |

No ADRs needed (all decisions marked "No").

## SOLID Assessment

**PASS** — 0 findings. All changes are additive, each file retains SRP, dependency direction correct (consumers → skill).

## Robert's Oath Check

**CLEAN** — 0 warnings. All 8 oath dimensions pass.

## Security Notes

**CLEAR** — 0 findings. Pure Markdown content, no security surface.

## Rollback Plan

Reverse dependency order:
1. Revert `agents/interviewer.md`
2. Revert `skills/configure-ecc/SKILL.md`
3. Revert `commands/spec-refactor.md`
4. Revert `commands/spec-fix.md`
5. Revert `commands/spec-dev.md`
6. Revert `agents/interface-designer.md`
7. Revert `commands/design.md`
8. Revert `skills/grill-me/SKILL.md`

Each revert is independent — no cascading failures since changes are additive text.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 0 |
| Robert (oath) | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS | All 15 ACs mapped to PCs, zero gaps |
| Order | PASS | grill-me foundation first, then consumers |
| Fragility | PASS (advisory) | Grep PCs on Markdown are inherently flexible; case-insensitive matching added |
| Rollback | PASS | All changes additive, independent reverts |
| Architecture | PASS | All content layer, zero Rust/hexagon impact |
| Blast radius | PASS | 8 files, all necessary per confirmed scope |
| Missing PCs | PASS (round 2) | PC-019 added for ecc validate after round 1 CONDITIONAL |
| Doc plan | PASS | CHANGELOG entry sufficient, no ADRs needed |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `skills/grill-me/SKILL.md` | Modify | US-001: AC-001.1-4 |
| 2 | `commands/design.md` | Modify | US-002: AC-002.1, AC-002.3-4 |
| 3 | `agents/interface-designer.md` | Modify | US-002: AC-002.2 |
| 4 | `commands/spec-dev.md` | Modify | US-003: AC-003.1, AC-003.4 |
| 5 | `commands/spec-fix.md` | Modify | US-003: AC-003.2, AC-003.4 |
| 6 | `commands/spec-refactor.md` | Modify | US-003: AC-003.3, AC-003.4 |
| 7 | `skills/configure-ecc/SKILL.md` | Modify | US-004: AC-004.1, AC-004.3 |
| 8 | `agents/interviewer.md` | Modify | US-004: AC-004.2 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-askuserquestion-preview-field/spec.md | Full spec + phase summary |
| docs/specs/2026-03-26-askuserquestion-preview-field/design.md | Full design + phase summary |
