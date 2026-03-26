# Solution: Offload Web Research to Task Subagents (BL-049)

## Spec Reference
Concern: refactor, Feature: BL-049 Offload web research phase to Task subagents

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `commands/spec-dev.md` | modify | Replace Phase 3 inline WebSearch with Task subagent invocation | US-001, AC-001.1 |
| 2 | `commands/spec-fix.md` | modify | Replace Phase 3 inline WebSearch with Task subagent invocation | US-001, AC-001.2 |
| 3 | `commands/spec-refactor.md` | modify | Replace Phase 3 inline WebSearch with Task subagent invocation | US-001, AC-001.3 |
| 4 | `CHANGELOG.md` | modify | Add BL-049 entry | US-003, AC-003.1 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | spec-dev no inline WebSearch instructions | AC-001.1, AC-001.7 | `! grep -q 'Run each query using .WebSearch.' commands/spec-dev.md` | exit 0 |
| PC-002 | unit | spec-dev has Task subagent | AC-001.1 | `grep -qi 'Launch.*Task\|Task.*subagent' commands/spec-dev.md` | exit 0 |
| PC-003 | unit | spec-dev has allowedTools with WebSearch | AC-001.9 | `grep -qi 'allowedTools.*WebSearch' commands/spec-dev.md` | exit 0 |
| PC-004 | unit | spec-fix no inline WebSearch | AC-001.2, AC-001.7 | `! grep -q 'Run each query using .WebSearch.' commands/spec-fix.md` | exit 0 |
| PC-005 | unit | spec-fix has Task subagent | AC-001.2 | `grep -qi 'Launch.*Task\|Task.*subagent' commands/spec-fix.md` | exit 0 |
| PC-006 | unit | spec-refactor no inline WebSearch | AC-001.3, AC-001.7 | `! grep -q 'Run each query using .WebSearch.' commands/spec-refactor.md` | exit 0 |
| PC-007 | unit | spec-refactor has Task subagent | AC-001.3 | `grep -qi 'Launch.*Task\|Task.*subagent' commands/spec-refactor.md` | exit 0 |
| PC-008 | unit | Subagent receives intent + description + tech stack | AC-001.4 | `grep -qi 'intent.*type\|user.*input\|tech.*stack\|detected.*tech' commands/spec-dev.md` | exit 0 |
| PC-009 | unit | Subagent returns Research Summary | AC-001.5 | `grep -qi 'Research Summary\|3-7 bullet' commands/spec-dev.md` | exit 0 |
| PC-010 | unit | Subagent failure graceful degradation | AC-001.6, AC-001.11 | `grep -qi 'Web research skipped\|subagent failed' commands/spec-dev.md` | exit 0 |
| PC-011 | unit | exa-web-search fallback in subagent | AC-001.10 | `grep -qi 'exa-web-search' commands/spec-dev.md` | exit 0 |
| PC-012 | unit | Phase 4 grill-me unchanged | AC-001.8 | `grep -qi 'Grill-Me Interview\|grill-me' commands/spec-dev.md` | exit 0 |
| PC-013 | unit | spec-dev domain framing preserved | AC-002.1 | `grep -qi 'best practices.*libraries\|libraries.*patterns\|prior art' commands/spec-dev.md` | exit 0 |
| PC-014 | unit | spec-fix domain framing preserved | AC-002.2 | `grep -qi 'known issues\|fix patterns' commands/spec-fix.md` | exit 0 |
| PC-015 | unit | spec-refactor domain framing preserved | AC-002.3 | `grep -qi 'refactoring patterns\|migration guides' commands/spec-refactor.md` | exit 0 |
| PC-016 | unit | Domain-specific framing in subagent prompt | AC-002.4 | `grep -qi 'focus.*on\|search.*for' commands/spec-dev.md` | exit 0 |
| PC-017 | unit | CHANGELOG has BL-049 | AC-003.1 | `grep -q 'BL-049' CHANGELOG.md` | exit 0 |
| PC-018 | build | cargo test passes | AC-003.2 | `cargo test` | pass |
| PC-019 | lint | cargo clippy passes | AC-003.3 | `cargo clippy -- -D warnings` | exit 0 |

| PC-020 | unit | spec-fix has allowedTools with WebSearch | AC-001.9 | `grep -qi 'allowedTools.*WebSearch' commands/spec-fix.md` | exit 0 |
| PC-021 | unit | spec-refactor has allowedTools with WebSearch | AC-001.9 | `grep -qi 'allowedTools.*WebSearch' commands/spec-refactor.md` | exit 0 |
| PC-022 | unit | spec-fix failure graceful degradation | AC-001.6, AC-001.11 | `grep -qi 'Web research skipped\|subagent failed' commands/spec-fix.md` | exit 0 |
| PC-023 | unit | spec-refactor failure graceful degradation | AC-001.6, AC-001.11 | `grep -qi 'Web research skipped\|subagent failed' commands/spec-refactor.md` | exit 0 |

### Coverage Check
All 18 ACs covered by 23 PCs. No uncovered ACs.

### E2E Test Plan
No E2E boundaries affected.

### E2E Activation Rules
No E2E tests to activate.

## Test Strategy
TDD order:
1. **Phase 1 (PC-001–003)**: Modify spec-dev.md — replace inline WebSearch with Task subagent
2. **Phase 2 (PC-004–005)**: Modify spec-fix.md — same change
3. **Phase 3 (PC-006–007)**: Modify spec-refactor.md — same change
4. **Phase 4 (PC-008–016)**: Verify all cross-cutting PCs (domain framing, fallback, failure handling)
5. **Phase 5 (PC-017–019)**: CHANGELOG + quality gate

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | Project | Add entry | BL-049: Phase 3 delegated to Task subagent | AC-003.1 |

## SOLID Assessment
PASS. Pure markdown modification. No abstractions changed. SRP maintained — each command file owns its own Phase 3 invocation structure.

## Robert's Oath Check
CLEAN. Mechanical refactoring removing throwaway tokens from main context. 19 PCs verify correctness.

## Security Notes
CLEAR. No code, no input handling, no secrets. Pure markdown command file modification.

## Rollback Plan
1. Revert `CHANGELOG.md`
2. Revert `commands/spec-refactor.md`
3. Revert `commands/spec-fix.md`
4. Revert `commands/spec-dev.md`
