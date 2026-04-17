# Spec: ASCII Diagram Sweep Across 9 ECC Crates

Source: BL-132 | Scope: HIGH | Doc-comments only

## Problem Statement

ECC's Rust codebase has ~950 public items across 9 crates with near-zero ASCII documentation in doc-comments. The `ascii-doc-diagrams` skill convention has only been applied to 1 file. ~115 eligible items need diagrams or pattern annotations for developer onboarding.

## Research Summary

- Existing skill `skills/ascii-doc-diagrams/SKILL.md` defines eligibility, diagram types, format rules
- One existing diagram in `crates/ecc-domain/src/claw/turn.rs` — proves `text` blocks render in rustdoc
- 80-column max, 20-line max, `+--+`/`|`/`-->` characters only

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Per-crate stories, all parallel | Zero file overlap | No |
| 2 | Triage pass before annotating | ~115 of 950 qualify | No |
| 3 | Priority: domain + workflow first | BL-132 named targets | No |

## User Stories

### US-001: ecc-domain (~37 items, ~30 files)
- AC-001.1: Phase enum has state-transition diagram
- AC-001.2: TaskStatus enum has state-transition diagram
- AC-001.3: WorkflowState struct has composition diagram
- AC-001.4: resolve_transition has flow/decision diagram
- AC-001.5: All eligible items annotated; `cargo doc -p ecc-domain --no-deps` succeeds

### US-002: ecc-workflow (~13 items, ~10 files)
- AC-002.1: transition run_with_store has flow/decision diagram
- AC-002.2: phase_gate function has flow/decision diagram
- AC-002.3: All eligible items annotated; `cargo doc -p ecc-workflow --no-deps` succeeds

### US-003: ecc-ports (~10 items, ~12 files)
- AC-003.1: Every pub trait has `# Pattern` section citing Port [Hexagonal Architecture]
- AC-003.2: `cargo doc -p ecc-ports --no-deps` succeeds

### US-004: ecc-app (~25 items, ~20 files)
- AC-004.1: dispatch function has flow/decision diagram
- AC-004.2: state_resolver has flow/decision diagram
- AC-004.3: HookPorts has composition diagram
- AC-004.4: All eligible items annotated; `cargo doc -p ecc-app --no-deps` succeeds

### US-005: ecc-infra (~20 items, ~15 files)
- AC-005.1: Every adapter struct has Pattern annotation
- AC-005.2: `cargo doc -p ecc-infra --no-deps` succeeds

### US-006: ecc-cli (~5 items, ~5 files)
- AC-006.1: Eligible CLI items annotated
- AC-006.2: `cargo doc -p ecc-cli --no-deps` succeeds

### US-007: ecc-flock (~5 items, 1 file)
- AC-007.1: FlockGuard has RAII pattern annotation
- AC-007.2: Lock acquisition flow diagram
- AC-007.3: `cargo doc -p ecc-flock --no-deps` succeeds

### US-008: Verification (0 changes)
- AC-008.1: ecc-test-support confirmed zero eligible items
- AC-008.2: ecc-integration-tests confirmed zero eligible items
- AC-008.3: `cargo doc --workspace --no-deps` succeeds

## Affected Modules

All 9 crates — doc-comments only. Zero functional changes.

## Constraints

Diagrams: `text` fenced blocks, ≤80 cols, ≤20 lines, `+--+`/`|`/`-->` characters. Eligibility per SKILL.md.

## Non-Requirements

No functional code changes. No new tests. Not exhaustive annotation of all 950 items.

## E2E Boundaries Affected

None.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | project | CHANGELOG.md | docs: ASCII diagram sweep (BL-132) |

## Open Questions

None.
