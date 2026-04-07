# Solution: BL-125 Token Optimization Wave 2

## Spec Reference
Concern: refactor, Feature: token-boilerplate-cleanup

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `agents/.templates/todowrite-block.md` | create | Canonical TodoWrite guard block | US-001, AC-001.0 |
| 2 | `crates/ecc-app/src/install/global/steps.rs` | modify | Add expand_tracking_field() + step_expand_tracking(), wire into step_merge_artifacts() | US-001, AC-001.2, AC-001.4 |
| 3 | 25 `agents/*.md` files | modify | Add tracking: todowrite frontmatter, remove inline boilerplate | US-001, AC-001.1 |
| 4 | `CLAUDE.md` | modify | Trim CLI Commands section to top-10 + pointer | US-002, AC-002.1 |
| 5 | `rules/common/performance.md` | modify | Trim to model routing table + thinking effort tiers only | US-004, AC-004.1 |
| 6 | `rules/common/agents.md` | modify | Trim to 10-15 line command mapping summary | US-005, AC-005.1 |
| 7 | `rules/ecc/development.md` | modify | Document tracking: todowrite convention | doc impact |
| 8 | `CHANGELOG.md` | modify | Add BL-125 entry | doc impact |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | expand_tracking_field inserts block before TodoWrite items line | AC-001.4 | `cargo test -p ecc-app -- install::global::steps::tests::expand_tracking_inserts_before_items` | PASS |
| PC-002 | unit | expand_tracking_field appends at end when no items line | AC-001.4 | `cargo test -p ecc-app -- install::global::steps::tests::expand_tracking_appends_at_end` | PASS |
| PC-003 | unit | expand_tracking_field no-ops without tracking frontmatter | AC-001.4 | `cargo test -p ecc-app -- install::global::steps::tests::expand_tracking_noop_no_frontmatter` | PASS |
| PC-004 | unit | expand_tracking_field no-ops when template missing | AC-001.4 | `cargo test -p ecc-app -- install::global::steps::tests::expand_tracking_noop_missing_template` | PASS |
| PC-005 | unit | expand_tracking_field is idempotent (skip if block already present) | AC-001.2 | `cargo test -p ecc-app -- install::global::steps::tests::expand_tracking_idempotent` | PASS |
| PC-006 | validate | ecc validate agents passes on 25 modified agents | AC-001.3 | `cargo run --bin ecc -- validate agents` | exit 0 |
| PC-007 | validate | ecc validate rules passes after trims | AC-004.3, AC-005.2 | `cargo run --bin ecc -- validate rules` | exit 0 |
| PC-008 | content | CLAUDE.md CLI section ≤ 15 lines | AC-002.1 | `awk '/## CLI Commands/,/^## [^C]/' CLAUDE.md \| wc -l` | ≤ 15 |
| PC-009 | content | docs/commands-reference.md exists and non-empty | AC-002.2 | `test -s docs/commands-reference.md && echo ok` | ok |
| PC-010 | content | performance.md ≤ 30 lines | AC-004.1 | `wc -l < rules/common/performance.md` | ≤ 30 |
| PC-011 | content | agents.md ≤ 15 lines | AC-005.1 | `wc -l < rules/common/agents.md` | ≤ 15 |
| PC-012 | lint | cargo clippy clean | all | `cargo clippy -- -D warnings` | exit 0 |
| PC-013 | build | cargo build passes | all | `cargo build --workspace` | exit 0 |
| PC-014 | test | full test suite passes | all | `cargo test --workspace` | exit 0 |

### Coverage Check

- AC-001.0 → PC-001 (template content verified in test assertion)
- AC-001.1 → PC-006 (validate agents catches missing frontmatter)
- AC-001.2 → PC-001, PC-005
- AC-001.3 → PC-006
- AC-001.4 → PC-001 through PC-005
- AC-002.0 → PC-008 (top-10 enumerated in spec, verified by line count)
- AC-002.1 → PC-008
- AC-002.2 → PC-009
- AC-003.1 → manual verification during implementation
- AC-003.2 → manual (backlog item if needed)
- AC-003.3 → documented in implement-done.md
- AC-004.1 → PC-010
- AC-004.2 → manual (verify removed content is redundant)
- AC-004.3 → PC-007
- AC-005.1 → PC-011
- AC-005.2 → PC-007

All ACs covered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Install pipeline | FileSystem | FileSystem::write | Verify agent expansion post-install | ignored | Install pipeline modified |

### E2E Activation Rules

No E2E tests activated — PC-005 integration test covers the install expansion path with InMemoryFileSystem.

## Test Strategy

TDD order:
1. PC-001 through PC-005 — Rust expand_tracking_field (RED → GREEN → REFACTOR)
2. PC-006 — validate agents after 25 agent edits
3. PC-008, PC-009 — CLAUDE.md trim verification
4. PC-010, PC-007 — performance.md trim + validate
5. PC-011, PC-007 — agents.md trim + validate
6. PC-012, PC-013, PC-014 — final gates

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | project | modify | Trim CLI section to top-10 | US-002 |
| 2 | rules/ecc/development.md | rule | modify | Document tracking: todowrite convention | US-001 |
| 3 | CHANGELOG.md | project | modify | Add BL-125 entry | all |

## SOLID Assessment

PASS — One additive function in install pipeline, follows existing step_merge_artifacts pattern. No new types or traits needed.

## Robert's Oath Check

CLEAN — Pure cleanup + one additive function. Boy Scout Rule applied (no mess left behind). Test coverage via InMemoryFileSystem.

## Security Notes

CLEAR — Local template file reads only. No user input, no network, no secrets. expand_tracking_field reads from known ecc_root path.

## Rollback Plan

1. Revert rules/ecc/development.md
2. Revert rules/common/agents.md
3. Revert rules/common/performance.md
4. Revert CLAUDE.md
5. Revert 25 agents/*.md (restore inline boilerplate)
6. Revert crates/ecc-app/src/install/global/steps.rs
7. Delete agents/.templates/todowrite-block.md

## Bounded Contexts Affected

No bounded contexts affected — install pipeline is infrastructure, not domain. Agent files are instruction content, not code.

Other modules:
- ecc-app/install: steps.rs (install orchestration)
