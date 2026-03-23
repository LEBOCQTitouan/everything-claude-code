# Solution: Interactive Stage-by-Stage Questioning via AskUserQuestion (BL-061)

## Spec Reference
Concern: refactor, Feature: BL-061 Refactor grill-me skill and backlog command to use stage-by-stage interactive questioning via AskUserQuestion with challenge loops and cross-stage mutation

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/grill-me/SKILL.md` | modify | Rewrite: 5 canonical stages, challenge loops, question cap (25), stage reopen limit, skip-all, spec-mode/backlog-mode params, decision log output | US-001, AC-001.1–001.12 |
| 2 | `skills/grill-me-adversary/SKILL.md` | modify | Align stage names, explicit grill-me reference, challenges under current question | US-004, AC-004.1–004.5 |
| 3 | `skills/spec-pipeline-shared/SKILL.md` | modify | Remove "Grill-Me Interview Rules" section, replace with grill-me skill reference. Retain: project detection, adversarial review, spec output schema | US-002, US-005, AC-002.1, AC-005.1–005.3 |
| 4 | `commands/spec-dev.md` | modify | Remove inlined interview rules, reference grill-me skill with spec-mode | US-002, AC-002.2, AC-002.4, AC-002.5 |
| 5 | `commands/spec-fix.md` | modify | Remove inlined interview rules, reference grill-me skill with spec-mode | US-002, AC-002.2, AC-002.5 |
| 6 | `commands/spec-refactor.md` | modify | Remove inlined interview rules, reference grill-me skill with spec-mode | US-002, AC-002.2, AC-002.5 |
| 7 | `commands/backlog.md` | modify | Add allowed-tools frontmatter, delegate challenge to grill-me skill | US-003, US-007, AC-003.1, AC-007.1 |
| 8 | `skills/backlog-management/SKILL.md` | modify | Update Challenge Log format for grill-me output | US-003, AC-003.6 |
| 9 | `agents/backlog-curator.md` | modify | Replace ad-hoc questions with grill-me delegation, add grill-me to skills | US-003, AC-003.4, AC-003.5 |
| 10 | `.claude/hooks/grill-me-gate.sh` | create | Stop hook: content-presence check for grill-me decisions in spec output | US-006, AC-006.1–006.6 |
| 11 | `.claude/settings.json` | modify | Register grill-me-gate.sh as Stop hook | US-006, AC-006.1 |
| 12 | `docs/domain/glossary.md` | modify | Update grill-me entry to "universal questioning protocol" | US-008, AC-008.1 |
| 13 | `CHANGELOG.md` | modify | Add BL-061 refactoring entry | US-008, AC-008.2 |
| 14 | `docs/adr/0017-grill-me-universal-protocol.md` | create | ADR for universal protocol decision | US-008, AC-008.3 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | 5 canonical stage names present | AC-001.1 | `grep -q 'Clarity' skills/grill-me/SKILL.md && grep -q 'Assumptions' skills/grill-me/SKILL.md && grep -q 'Edge Cases' skills/grill-me/SKILL.md && grep -q 'Alternatives' skills/grill-me/SKILL.md && grep -q 'Stress Test' skills/grill-me/SKILL.md` | exit 0 |
| PC-002 | unit | AskUserQuestion one-at-a-time language | AC-001.2 | `grep -q 'AskUserQuestion' skills/grill-me/SKILL.md && grep -qi 'one.*question.*per.*turn\|one.*at.*a.*time' skills/grill-me/SKILL.md` | exit 0 |
| PC-003 | unit | Challenge loop documented | AC-001.3 | `grep -qi 'challenge.*loop\|challenge.*follow-up\|challengeable' skills/grill-me/SKILL.md` | exit 0 |
| PC-004 | unit | Non-adversary exit criterion | AC-001.4 | `grep -q '2 follow-up' skills/grill-me/SKILL.md && grep -qi 'termination.*reason\|state.*termination' skills/grill-me/SKILL.md` | exit 0 |
| PC-005 | unit | Cross-stage mutation | AC-001.5 | `grep -qi 'cross-stage\|add.*questions.*any.*stage' skills/grill-me/SKILL.md` | exit 0 |
| PC-006 | unit | Stage progression rule | AC-001.6 | `grep -qi 'all.*questions.*answered.*next.*stage\|stage.*complete.*proceed' skills/grill-me/SKILL.md` | exit 0 |
| PC-007 | unit | Question list with 4 statuses | AC-001.7 | `grep -q 'pending' skills/grill-me/SKILL.md && grep -q 'open' skills/grill-me/SKILL.md && grep -q 'challenged' skills/grill-me/SKILL.md && grep -q 'answered' skills/grill-me/SKILL.md` | exit 0 |
| PC-008 | unit | Decision log output | AC-001.8 | `grep -qi 'decision.*log\|output.*decision' skills/grill-me/SKILL.md` | exit 0 |
| PC-009 | unit | Question cap of 25 | AC-001.9 | `grep -q '25' skills/grill-me/SKILL.md && grep -qi 'cap\|maximum.*question\|question.*limit' skills/grill-me/SKILL.md` | exit 0 |
| PC-010 | unit | Stage reopen limit once | AC-001.10 | `grep -qi 'reopen.*once\|once.*reopen\|reopens.*exactly.*once' skills/grill-me/SKILL.md` | exit 0 |
| PC-011 | unit | Skip all / done with warning | AC-001.11 | `grep -qi 'skip.*all\|"done"' skills/grill-me/SKILL.md && grep -qi '50%\|degraded' skills/grill-me/SKILL.md` | exit 0 |
| PC-012 | unit | AskUserQuestion language enforcement | AC-001.12 | `grep -qi 'AskUserQuestion' skills/grill-me/SKILL.md` | exit 0 |
| PC-013 | unit | Old stage names absent | AC-001.1 | `! grep -q '### Stage 1: Problem' skills/grill-me/SKILL.md && ! grep -q '### Stage 4: Rollback' skills/grill-me/SKILL.md && ! grep -q '### Stage 5: Success Criteria' skills/grill-me/SKILL.md` | exit 0 |
| PC-014 | unit | Spec-mode parameters | AC-002.3, AC-002.4 | `grep -qi 'spec-mode\|spec mode' skills/grill-me/SKILL.md && grep -q '"spec it"' skills/grill-me/SKILL.md && grep -q '(Recommended)' skills/grill-me/SKILL.md` | exit 0 |
| PC-015 | unit | Backlog-mode parameters | AC-003.2 | `grep -qi 'backlog-mode\|backlog mode' skills/grill-me/SKILL.md && grep -qi 'max 3 stages\|3 stages' skills/grill-me/SKILL.md` | exit 0 |
| PC-016 | unit | Adversary canonical stages | AC-004.5 | `grep -q 'Clarity' skills/grill-me-adversary/SKILL.md && grep -q 'Assumptions' skills/grill-me-adversary/SKILL.md && grep -q 'Stress Test' skills/grill-me-adversary/SKILL.md` | exit 0 |
| PC-017 | unit | Adversary explicit grill-me reference | AC-004.4 | `grep -qi 'grill-me skill\|skills/grill-me' skills/grill-me-adversary/SKILL.md` | exit 0 |
| PC-018 | unit | Adversary challenges under current question | AC-004.3 | `grep -qi 'challenge.*under.*current\|under.*current.*question' skills/grill-me-adversary/SKILL.md` | exit 0 |
| PC-019 | unit | Adversary scoring alongside challenge | AC-004.2 | `grep -qi 'completeness.*specificity\|specificity.*completeness' skills/grill-me-adversary/SKILL.md` | exit 0 |
| PC-020 | unit | Adversary question-gen for stage context | AC-004.1 | `grep -qi 'hardest.*question.*stage\|stage.*hardest' skills/grill-me-adversary/SKILL.md` | exit 0 |
| PC-021 | unit | spec-pipeline-shared no grill-me rules | AC-002.1, AC-005.1 | `! grep -q '## Grill-Me Interview Rules' skills/spec-pipeline-shared/SKILL.md` | exit 0 |
| PC-022 | unit | spec-pipeline-shared references grill-me | AC-002.1 | `grep -qi 'grill-me skill' skills/spec-pipeline-shared/SKILL.md` | exit 0 |
| PC-023 | unit | spec-pipeline-shared retains project detection | AC-005.1 | `grep -q 'Project Detection' skills/spec-pipeline-shared/SKILL.md` | exit 0 |
| PC-024 | unit | spec-pipeline-shared retains adversarial review | AC-005.1 | `grep -qi 'Adversarial Review' skills/spec-pipeline-shared/SKILL.md` | exit 0 |
| PC-025 | unit | spec-pipeline-shared retains spec output schema | AC-005.2 | `grep -qi 'output\|schema\|spec' skills/spec-pipeline-shared/SKILL.md` | exit 0 |
| PC-026 | unit | spec-dev removes inlined rules | AC-002.5 | `! grep -q 'Ask \*\*one question at a time\*\*' commands/spec-dev.md` | exit 0 |
| PC-027 | unit | spec-dev references grill-me | AC-002.2 | `grep -qi 'grill-me skill\|grill-me' commands/spec-dev.md` | exit 0 |
| PC-028 | unit | spec-fix removes inlined rules | AC-002.5 | `! grep -q 'Ask \*\*one question at a time\*\*' commands/spec-fix.md` | exit 0 |
| PC-029 | unit | spec-fix references grill-me | AC-002.2 | `grep -qi 'grill-me' commands/spec-fix.md` | exit 0 |
| PC-030 | unit | spec-refactor removes inlined rules | AC-002.5 | `! grep -q 'Ask \*\*one question at a time\*\*' commands/spec-refactor.md` | exit 0 |
| PC-031 | unit | spec-refactor references grill-me | AC-002.2 | `grep -qi 'grill-me' commands/spec-refactor.md` | exit 0 |
| PC-032 | unit | Campaign persistence preserved | AC-002.6 | `grep -qi 'campaign.md' commands/spec-dev.md && grep -qi 'Grill-Me Decisions' commands/spec-dev.md` | exit 0 |
| PC-033 | unit | backlog delegates to grill-me | AC-003.1 | `grep -qi 'grill-me' commands/backlog.md` | exit 0 |
| PC-034 | unit | backlog-curator delegates to grill-me | AC-003.4, AC-003.5 | `grep -qi 'grill-me' agents/backlog-curator.md && ! grep -qi '1-3 focused questions' agents/backlog-curator.md` | exit 0 |
| PC-035 | unit | backlog-management challenge log updated | AC-003.6 | `grep -qi 'grill-me.*output\|stages.*questions' skills/backlog-management/SKILL.md` | exit 0 |
| PC-036 | unit | Scope-based config | AC-003.2, AC-003.3 | `grep -qi 'HIGH.*EPIC\|scope.*full' commands/backlog.md || grep -qi 'HIGH.*EPIC\|scope.*full' agents/backlog-curator.md` | exit 0 |
| PC-037 | unit | backlog allowed-tools frontmatter | AC-007.1 | `grep -q 'allowed-tools' commands/backlog.md` | exit 0 |
| PC-038 | unit | Hook file exists | AC-006.1 | `test -f .claude/hooks/grill-me-gate.sh` | exit 0 |
| PC-039 | unit | Hook has set -uo pipefail | AC-006.5 | `grep -q 'set -uo pipefail' .claude/hooks/grill-me-gate.sh` | exit 0 |
| PC-040 | unit | Hook has ECC_WORKFLOW_BYPASS | AC-006.4 | `grep -q 'ECC_WORKFLOW_BYPASS' .claude/hooks/grill-me-gate.sh` | exit 0 |
| PC-041 | unit | Hook checks for decision markers | AC-006.6 | `grep -q 'grep' .claude/hooks/grill-me-gate.sh && grep -qi 'Grill-Me Decisions\|decision' .claude/hooks/grill-me-gate.sh` | exit 0 |
| PC-042 | unit | Hook emits WARNING | AC-006.2 | `grep -q 'WARNING' .claude/hooks/grill-me-gate.sh` | exit 0 |
| PC-043 | unit | Hook exits 0 | AC-006.3 | `grep -q 'exit 0' .claude/hooks/grill-me-gate.sh` | exit 0 |
| PC-044 | unit | Hook registered in settings | AC-006.1 | `grep -q 'grill-me-gate' .claude/settings.json` | exit 0 |
| PC-045 | unit | Glossary has universal protocol | AC-008.1 | `grep -qi 'universal.*protocol\|universal.*questioning' docs/domain/glossary.md` | exit 0 |
| PC-046 | unit | CHANGELOG has BL-061 | AC-008.2 | `grep -q 'BL-061' CHANGELOG.md` | exit 0 |
| PC-047 | unit | ADR 0017 exists | AC-008.3 | `test -f docs/adr/0017-grill-me-universal-protocol.md` | exit 0 |
| PC-048 | unit | ADR documents universal protocol | AC-008.3 | `grep -qi 'universal.*protocol' docs/adr/0017-grill-me-universal-protocol.md` | exit 0 |
| PC-049 | unit | Git tag for rollback | AC-008.4 | `git tag -l 'pre-bl-061' | grep -q 'pre-bl-061'` | exit 0 |
| PC-050 | lint | Cargo clippy clean | AC-008.2 | `cargo clippy -- -D warnings` | exit 0 |
| PC-051 | build | Cargo build | AC-008.2 | `cargo build` | exit 0 |
| PC-052 | build | All tests pass | AC-008.2 | `cargo test` | exit 0 |
| PC-053 | unit | All grill-me rules in one place | AC-005.3 | `grep -qi 'spec-mode\|backlog-mode' skills/grill-me/SKILL.md` | exit 0 |
| PC-054 | unit | grill-me output feeds backlog | AC-003.4 | `grep -qi 'grill-me.*output\|output.*feeds\|prompt optimization' commands/backlog.md || grep -qi 'grill-me.*output\|prompt optimization' agents/backlog-curator.md` | exit 0 |

| PC-055 | unit | Campaign persistence in spec-fix | AC-002.6 | `grep -qi 'campaign.md' commands/spec-fix.md && grep -qi 'Grill-Me Decisions' commands/spec-fix.md` | exit 0 |
| PC-056 | unit | Campaign persistence in spec-refactor | AC-002.6 | `grep -qi 'campaign.md' commands/spec-refactor.md && grep -qi 'Grill-Me Decisions' commands/spec-refactor.md` | exit 0 |
| PC-057 | unit | Standalone mode default in grill-me | AC-001.1 | `grep -qi 'standalone\|default.*mode\|without.*mode' skills/grill-me/SKILL.md` | exit 0 |
| PC-058 | unit | grill-me in backlog-curator skills frontmatter | AC-003.5 | `grep -qi 'grill-me' agents/backlog-curator.md` | exit 0 |
| PC-059 | unit | spec-pipeline-shared description updated | AC-005.1 | `! grep -qi 'grill-me interview rules' skills/spec-pipeline-shared/SKILL.md` | exit 0 |

### Coverage Check
All 44 ACs covered by 59 PCs. No uncovered ACs.

### Design Clarifications (from adversarial review)

**Rollback timing**: Before Phase 1 begins, create git tag `pre-bl-061` as the first action. PC-049 verifies this exists. The tag MUST be created before any file modifications.

**Fragile PC-025 replaced**: PC-025 checks `grep -qi 'output\|schema\|spec'` which is too broad. Implementation should verify a specific heading like `## Spec Output` or equivalent.

**settings.local.json rollback**: On rollback, selectively remove the `grill-me-gate` hook entry rather than full-file revert (other hooks may have been added).

**Test count**: CLAUDE.md currently says 1224 tests (updated by BL-058). This refactoring adds no Rust tests — count stays at 1224. No CLAUDE.md test count update needed.

**commands-reference.md**: Does not document individual backlog challenge behavior — only lists slash commands. No update needed.

### E2E Test Plan
No E2E boundaries affected — pure skill/command/hook changes.

### E2E Activation Rules
No E2E tests to activate.

## Test Strategy
TDD order:
1. **Phase 1 (PC-001–015)**: Grill-me core rewrite — canonical stages, challenge loops, caps, modes
2. **Phase 2 (PC-016–020)**: Grill-me-adversary alignment — canonical stages, explicit reference
3. **Phase 3 (PC-021–032)**: Spec pipeline — remove inline rules, add skill reference, decompose shared
4. **Phase 4 (PC-033–037)**: Backlog integration — delegate to grill-me, frontmatter
5. **Phase 5 (PC-038–044)**: Hook enforcement — create grill-me-gate.sh, register
6. **Phase 6 (PC-045–049)**: Documentation — glossary, CHANGELOG, ADR 0017, git tag
7. **Phase 7 (PC-050–054)**: Final gate — clippy, build, tests, cross-checks

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/domain/glossary.md` | Domain | Update | Grill-me → "universal questioning protocol" | AC-008.1 |
| 2 | `CHANGELOG.md` | Project | Add entry | BL-061 refactoring: unified grill-me, hook enforcement, decomposed spec-pipeline-shared | AC-008.2 |
| 3 | `docs/adr/0017-grill-me-universal-protocol.md` | Architecture | Create | Universal protocol decision: 3 systems → 1, hook enforcement, stage-by-stage model | AC-008.3 |

## SOLID Assessment
PASS. Grill-me is stable abstraction (SRP). Consumers delegate via reference (DIP). spec-pipeline-shared decomposed to remove grab-bag (SRP). Adversary decorates without modifying base (OCP).

## Robert's Oath Check
CLEAN. Eliminates documented mess (3→1 systems). Proof via 54 grep PCs. 7 atomic phases. Rollback via git tag. Behavior preservation constrained.

## Security Notes
CLEAR. Read-only hook with content-presence grep. No user input handling. ECC_WORKFLOW_BYPASS guard. Trusted paths only.

## Rollback Plan
Reverse dependency order:
1. Remove `docs/adr/0017-grill-me-universal-protocol.md`
2. Revert `CHANGELOG.md`
3. Revert `docs/domain/glossary.md`
4. Revert `.claude/settings.json` (remove hook registration)
5. Delete `.claude/hooks/grill-me-gate.sh`
6. Revert `agents/backlog-curator.md`
7. Revert `skills/backlog-management/SKILL.md`
8. Revert `commands/backlog.md`
9. Revert `commands/spec-refactor.md`
10. Revert `commands/spec-fix.md`
11. Revert `commands/spec-dev.md`
12. Revert `skills/spec-pipeline-shared/SKILL.md`
13. Revert `skills/grill-me-adversary/SKILL.md`
14. Revert `skills/grill-me/SKILL.md`

Or: `git checkout pre-bl-061 -- <all files above>`
