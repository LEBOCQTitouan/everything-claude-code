# Solution: BL-076 Statusline Unicode Byte-Counting Fix

## Spec Reference
Concern: fix, Feature: BL-076 Statusline Unicode byte-counting bug

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `statusline/statusline-command.sh` | Modify | Add LC_ALL=C.UTF-8 guard, visible_width() function, update 4 width comparisons | US-001, AC-001.1-001.5 |
| 2 | `tests/statusline/unicode-width.bats` | Create | Bats tests for Unicode width handling and rate limit visibility | US-002, AC-002.1-002.3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Script sets LC_ALL=C.UTF-8 after set line | AC-001.1 | `head -5 statusline/statusline-command.sh \| grep -q 'LC_ALL=C.UTF-8'` | Exit 0 |
| PC-002 | unit | visible_width function exists | AC-001.2 | `grep -q 'visible_width()' statusline/statusline-command.sh` | Exit 0 |
| PC-003 | integration | visible_width counts chars correctly for Unicode | AC-001.2 | `bash -c 'LC_ALL=C.UTF-8; source <(grep -A5 "^visible_width()" statusline/statusline-command.sh); [ "$(visible_width "◆ Model")" -eq 7 ]'` | Exit 0 |
| PC-004 | unit | build_output loop uses visible_width | AC-001.4 | `grep -A2 'stripped=.*strip_ansi' statusline/statusline-command.sh \| grep -q 'visible_width'` | Exit 0 |
| PC-005 | unit | Post-build RL check uses visible_width | AC-001.4 | `grep 'SEG_RL_NARROW' statusline/statusline-command.sh \| grep -q 'visible_width'` | Exit 0 |
| PC-006 | unit | Post-build worktree check uses visible_width | AC-001.4 | `grep 'SEG_WORKTREE_NARROW' statusline/statusline-command.sh \| grep -q 'visible_width'` | Exit 0 |
| PC-007 | unit | Post-build context check uses visible_width | AC-001.4 | `grep 'SEG_CTX_NARROW' statusline/statusline-command.sh \| grep -q 'visible_width'` | Exit 0 |
| PC-008 | integration | Rate limits visible at COLUMNS=120 with rate limit data | AC-001.3, AC-002.1 | `echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"rate_limits":{"five_hour":{"used_percentage":26},"seven_day":{"used_percentage":47}},"cost":{"total_cost_usd":0,"total_duration_ms":0,"total_lines_added":0,"total_lines_removed":0}}' \| COLUMNS=120 bash statusline/statusline-command.sh \| sed 's/\x1b\[[0-9;]*m//g' \| grep -q '5h:'` | Exit 0 |
| PC-009 | integration | Graceful degradation at COLUMNS=50 | AC-001.5, AC-002.2 | `echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"rate_limits":{"five_hour":{"used_percentage":26},"seven_day":{"used_percentage":47}},"cost":{"total_cost_usd":0,"total_duration_ms":0,"total_lines_added":0,"total_lines_removed":0}}' \| COLUMNS=50 bash statusline/statusline-command.sh \| sed 's/\x1b\[[0-9;]*m//g' \| grep -q 'Opus'` | Exit 0 |
| PC-010 | unit | Bats test file exists for Unicode width | AC-002.3 | `test -f tests/statusline/unicode-width.bats` | Exit 0 |
| PC-011 | integration | All existing Bats tests pass | - | `bats tests/statusline/` | Exit 0 |
| PC-012 | build | Cargo build passes | All | `cargo build --workspace` | Exit 0 |
| PC-013 | lint | Cargo clippy passes | All | `cargo clippy --workspace -- -D warnings` | Exit 0 |

### Coverage Check

All 8 ACs covered:

| AC | Covering PCs |
|---|---|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002, PC-003 |
| AC-001.3 | PC-008 |
| AC-001.4 | PC-004, PC-005, PC-006, PC-007 |
| AC-001.5 | PC-009 |
| AC-002.1 | PC-008 |
| AC-002.2 | PC-009 |
| AC-002.3 | PC-010, PC-003 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Statusline | Script | N/A | Rate limits visible at adequate width | ignored | statusline-command.sh modified |

### E2E Activation Rules
PC-008 and PC-009 serve as E2E integration tests for the statusline.

## Test Strategy

TDD order:
1. **PC-001** — LC_ALL guard (foundation)
2. **PC-002, PC-003** — visible_width function
3. **PC-004 to PC-007** — Update 4 comparison lines
4. **PC-008, PC-009** — Integration: rate limits + degradation
5. **PC-010, PC-011** — Bats tests
6. **PC-012, PC-013** — Build/clippy gates

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | LOW | Modify | Add BL-076 fix entry | All |

## SOLID Assessment
**PASS** — Shell script fix only. Single function added (SRP). No architecture changes.

## Robert's Oath Check
**CLEAN** — 13 PCs provide proof. Small, focused fix. No mess introduced.

## Security Notes
**CLEAR** — No user input handling changes. LC_ALL=C.UTF-8 is safe (subprocess-scoped).

## Rollback Plan
1. Revert `tests/statusline/unicode-width.bats`
2. Revert `statusline/statusline-command.sh` changes

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `statusline/statusline-command.sh` | Modify | US-001 |
| 2 | `tests/statusline/unicode-width.bats` | Create | US-002 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-statusline-unicode-fix/design.md` | Full design + Phase Summary |
