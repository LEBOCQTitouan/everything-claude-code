# Solution: /commit Slash Command (BL-063)

## Spec Reference
Concern: dev, Feature: BL-063 Create /commit slash command

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `commands/commit.md` | Create | New /commit command with 7 phases | All US (001-007) |
| 2 | `CLAUDE.md` | Modify | Register /commit in slash commands | US-007 |
| 3 | `CHANGELOG.md` | Modify | Add feature entry | All US |
| 4 | `docs/commands-reference.md` | Modify | Add /commit to command reference | US-007 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | ecc validate passes | AC-007.1 | `cargo run --bin ecc -- validate commands` | exit 0 |
| PC-002 | grep | Frontmatter has correct allowed-tools | AC-007.2 | `head -5 commands/commit.md \| grep -c "Bash.*Read\|Read.*Bash"` | 1 |
| PC-003 | grep | Nothing-to-commit detection | AC-001.1, AC-001.2 | `grep -c "Nothing to commit" commands/commit.md` | >= 1 |
| PC-004 | grep | Merge conflict detection | AC-001.3 | `grep -ci "merge conflict\|conflict.*block" commands/commit.md` | >= 1 |
| PC-005 | grep | git status primary staging | AC-002.1, AC-002.3 | `grep -c "git status" commands/commit.md` | >= 1 |
| PC-006 | grep | Session context enrichment | AC-002.2 | `grep -ci "session.*context\|session.*action" commands/commit.md` | >= 1 |
| PC-007 | grep | AskUserQuestion for confirmation | AC-002.4, AC-004.5 | `grep -c "AskUserQuestion" commands/commit.md` | >= 1 |
| PC-008 | grep | Atomic commit warning | AC-003.1, AC-003.2, AC-003.3 | `grep -ci "atomic\|multiple.*concern\|unrelated.*concern" commands/commit.md` | >= 1 |
| PC-009 | grep | Split guidance | AC-003.4 | `grep -ci "split\|unstag" commands/commit.md` | >= 1 |
| PC-010 | grep | Conventional commit types | AC-004.1, AC-004.2 | `grep -c "feat.*fix.*refactor.*docs.*test.*chore.*perf.*ci" commands/commit.md` | >= 1 |
| PC-011 | grep | Scope inference | AC-004.3, AC-004.4 | `grep -ci "scope.*infer\|infer.*scope\|directory.*scope" commands/commit.md` | >= 1 |
| PC-012 | grep | Message confirmation | AC-004.5 | `grep -ci "accept.*edit.*reject\|confirm.*message" commands/commit.md` | >= 1 |
| PC-013 | grep | Toolchain from state.json | AC-005.1 | `grep -c "toolchain" commands/commit.md` | >= 1 |
| PC-014 | grep | Cargo.toml fallback | AC-005.2, AC-005.3 | `grep -c "Cargo.toml" commands/commit.md` | >= 1 |
| PC-015 | grep | Pre-flight blocks on failure | AC-005.4, AC-005.5 | `grep -ci "block.*commit\|commit.*block\|pre-flight.*fail" commands/commit.md` | >= 1 |
| PC-016 | grep | Workflow state warning | AC-006.1, AC-006.2, AC-006.3 | `grep -c "implement.*warn\|warn.*implement\|workflow.*active" commands/commit.md` | >= 1 |
| PC-017 | grep | $ARGUMENTS handling | AC-007.3 | `grep -c "ARGUMENTS\|argument" commands/commit.md` | >= 1 |
| PC-018 | grep | CLAUDE.md mentions /commit | Doc | `grep -c "/commit" CLAUDE.md` | >= 1 |
| PC-019 | build | Rust build passes | All | `cargo build` | exit 0 |
| PC-020 | build | Rust tests pass | All | `cargo test` | exit 0 |
| PC-021 | grep | CHANGELOG mentions BL-063 | Doc | `grep -c "BL-063" CHANGELOG.md` | >= 1 |
| PC-022 | grep | commands-reference mentions /commit | Doc | `grep -c "/commit" docs/commands-reference.md` | >= 1 |

### Coverage Check

All 27 ACs covered by 22 PCs. Zero uncovered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | None | N/A | N/A | No E2E boundaries | — | — |

### E2E Activation Rules
No E2E tests needed.

## Test Strategy

TDD order:
1. **PC-001–017** (Wave 1: create commit.md + verify all content)
2. **PC-018, PC-021, PC-022** (Wave 2: doc updates — CLAUDE.md, CHANGELOG, commands-reference)
3. **PC-019–020** (Wave 3: final build/test gate)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Minor | Add /commit to side commands | Extend parenthetical list | US-007 |
| 2 | CHANGELOG.md | Minor | Add entry | "/commit slash command (BL-063)" | All US |
| 3 | docs/commands-reference.md | Minor | Add /commit entry | New row in command table | US-007 |

No ADRs needed.

## SOLID Assessment
**PASS** — 0 findings.

## Robert's Oath Check
**CLEAN** — 0 warnings.

## Security Notes
**CLEAR** — 0 findings.

## Rollback Plan
1. Revert docs/commands-reference.md
2. Revert CHANGELOG.md
3. Revert CLAUDE.md
4. Delete commands/commit.md

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
| Coverage | PASS (round 2) | Added PC-021 for CHANGELOG, PC-022 for commands-reference |
| Order | PASS | Single wave, trivial ordering |
| Fragility | PASS (advisory) | Grep PCs appropriate for Markdown content |
| Rollback | PASS | Delete file + revert 3 docs |
| Architecture | PASS (round 2) | Clarified standalone, not invoking native skill |
| Blast radius | PASS | 4 files, content only |
| Missing PCs | PASS (round 2) | Added CHANGELOG and commands-reference PCs |
| Doc plan | PASS (round 2) | Added docs/commands-reference.md |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | commands/commit.md | Create | All US |
| 2 | CLAUDE.md | Modify | US-007 |
| 3 | CHANGELOG.md | Modify | All US |
| 4 | docs/commands-reference.md | Modify | US-007 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-commit-slash-command/spec.md | Full spec + phase summary |
| docs/specs/2026-03-27-commit-slash-command/design.md | Full design + phase summary |
