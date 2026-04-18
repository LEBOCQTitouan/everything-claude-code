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

## Revision 2026-04-18

Amendment after re-review by requirements-analyst + architect + three adversarial rounds. Prior User Stories US-001..US-008 and all ACs remain binding. Adds 7 cross-cutting rules (R-1..R-7), tightens tautological ACs with crate minimums, adds committed classifier scripts with fixtures, and mandates CI integration. Adversary verdict: PASS (82.1/100) on round 3.

Decision source: `docs/specs/2026-04-18-bl132-ascii-diagram-sweep/campaign.md` (15 grill-me + adversary decisions).

### R-1: Clap-derive Deny-list

Inline `text` diagrams and `# Pattern` annotations are forbidden in files containing `#[derive(Parser|Subcommand|Args|ValueEnum)]`, except at `//!` module level or in `impl` block scope. Rationale: clap derive macros promote the first line of `///` above a struct/variant/field into `--help` short text; ASCII diagrams would corrupt user-visible CLI output.

- **AC-R1.1** — Files matching the clap-derive selector have no `///.*```text` line whose enclosing scope (determined by awk-tracked brace depth) is not (a) a `//!` header block, OR (b) inside an `impl` block.
- **AC-R1.2** — Every fenced code block in `///` doc-comments specifies one of: `text`, `rust`, `ignore`, `no_run`, `compile_fail`. ASCII diagrams MUST use `text` specifically.
- **AC-R1.3** — No Unicode box-drawing characters (U+2500–U+257F range) inside `///` doc-comments. ASCII `+-|` and `-->` only.
- **AC-R1.4** — Classifier scripts at `scripts/check-clap-derive-diagrams.sh`, `scripts/check-fence-hints.sh`, `scripts/check-drift-anchors.sh` are version-controlled with fixture files under `scripts/fixtures/bl132/`. Fixtures cover: impl-block diagram (pass), //!-level diagram (pass), ///-on-struct inside clap-derive file (fail). Fixture tests are mandatory gates in the PCs.
- **AC-R1.5** — Attribute-form bypass blocked: `rg -U '#\[doc\s*=.*```text' crates/ --glob '!target/**'` returns zero matches in clap-derive files.

### R-2: Explicit Additional Priority Targets (additive)

Six files are elevated to mandatory per-file targets, **additive** to US-001..US-007 eligibility — NOT a replacement list:

| File | Story |
|------|-------|
| `crates/ecc-domain/src/workflow/phase.rs` | US-001 |
| `crates/ecc-domain/src/workflow/state.rs` | US-001 |
| `crates/ecc-domain/src/backlog/lock_file.rs` | US-001 |
| `crates/ecc-domain/src/config/audit.rs` | US-001 |
| `crates/ecc-app/src/merge/mod.rs` | US-004 |
| `crates/ecc-workflow/src/commands/phase_gate.rs` | US-002 |

- **AC-R2.1** — Each of the 6 files has ≥1 `text` fence whose body contains at least one of `-->`, `+-+`, or `|` (structural diagram marker, not placeholder).
- **AC-R2.2** — R-2 is additive. `eligibility-target` = 115 items (per problem statement). `eligibility-floor` = 54 items (per AC-*.N-strict sum below). A PR passing only the floor is shippable; the ceiling (115) is aspirational. If gap > 30 items at ship, a follow-up issue must be filed referencing BL-132.

### R-3: Drift-anchor Rule (scoped classifier)

Flow/decision diagrams (those containing `--Y-->` or `--N-->` tokens *inside the `text` fence body*) MUST carry an HTML comment on one of the 3 `///` lines immediately preceding the fence opener:

```text
/// <!-- keep in sync with: <test_fn_name> -->
```

State-transition and composition diagrams are exempt — their types self-anchor via rustdoc.

- **AC-R3.1** — Every flow/decision diagram has the drift-anchor comment within 3 preceding `///` lines. `<test_fn_name>` appears in `cargo test --workspace --list` output.
- **AC-R3.2** — `scripts/check-drift-anchors.sh` ships with 2 fixtures: fence with `--Y-->` body missing anchor (fail), same fence with anchor present (pass).

### R-4: REMOVED

Cross-context rule had no automated gate. Deferred to a separate spec.

### R-5: Cap Relaxation (form tightened)

The skill's 20-line diagram cap is relaxed per-module for complex modules (e.g., `config/audit.rs` 18-enum cluster). Diagrams exceeding 20 lines MUST carry an HTML comment on exactly one of the 3 `///` lines immediately preceding the fence opener:

```text
/// <!-- cap-override: <N> lines, reason: <short text> -->
```

- **AC-R5.1** — No diagram exceeds 20 lines without the `cap-override` comment.
- **AC-R5.2** — Comment form is exactly `/// <!-- cap-override: <N> lines, reason: <text> -->`; N is a positive integer matching actual fence body line count; reason is ≥5 chars of non-whitespace.

### R-6: --help Smoke Test (CI-gated)

- **AC-R6.1** — `cargo run -p ecc-cli -- --help` and `cargo run -p ecc-workflow -- --help` both exit 0 AND stdout contains `Usage:`.
- **AC-R6.2** — The smoke test is wired into `.github/workflows/ci.yml` validate job before PR merge. Without CI mandate, this gate is aspirational only.

### R-7: Minimum Shippable Subset

- **AC-R7.1** — A PR referencing BL-132 MUST pass US-001 (ecc-domain) AND US-002 (ecc-workflow) at minimum. Deferral of US-003..US-008 is allowed with a follow-up issue reference. Deferral of US-001 or US-002 requires explicit justification in commit message and approval reference.

### Replacement for tautological AC-*.5 family

Prior ACs that used "cargo doc succeeds" as the verification (AC-001.5, AC-002.3, AC-004.4, AC-007.3, AC-008.3) were tautological: `cargo doc` builds even with zero diagrams. Replaced with crate-level diagram-count floors:

| AC ID | Replaces | Floor |
|-------|----------|-------|
| AC-001.5-strict | AC-001.5 | ≥20 `text` fences in `crates/ecc-domain/src/**/*.rs` |
| AC-002.3-strict | AC-002.3 | ≥5 `text` fences in `crates/ecc-workflow/src/**/*.rs` |
| AC-003.2-strict | (new) | ≥8 `# Pattern` blocks in `crates/ecc-ports/src/**/*.rs` |
| AC-004.4-strict | AC-004.4 | ≥10 `text` fences in `crates/ecc-app/src/**/*.rs` |
| AC-005.2-strict | AC-005.2 | ≥8 `# Pattern` blocks in `crates/ecc-infra/src/**/*.rs` |
| AC-006.2-strict | AC-006.2 | ≥2 diagrams at `//!`/`impl` in `crates/ecc-cli/src/**/*.rs` |
| AC-007.3-strict | AC-007.3 | ≥1 diagram AND ≥1 `# Pattern` in `crates/ecc-flock/src/lib.rs` |

Sum: 54 (floor). Target per problem statement: 115.

`cargo doc` success gates (PC-002..PC-011) are RETAINED — still useful, but not the sole verification.

### Decisions Backfill

Appended to prior Decisions Made table:

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 4 | Clap-derive deny-list | Prevents `--help` corruption from diagram prose | ADR-TBD (to be created in /design) |
| 5 | Drift-anchor on flow diagrams | Binds narrative docs to canary test | No |
| 6 | Cap-override with exact form | Prevents silent scope creep via oversized diagrams | No |
| 7 | Crate minimums floor 54 / target 115 | Floor is shippable, ceiling aspirational; gap > 30 triggers follow-up | No |
| 8 | Minimum shippable subset US-001 + US-002 | Priority targets are non-optional | No |

## Phase Summary

### Grill-Me Decisions (2026-04-18 session)

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Prior spec 2026-04-17 unimplemented — how to relate? | Hybrid: amend prior with Revision adding clap-derive gate + other accepted findings | User |
| 2 | Clap-derive deny-list scope? | Any file containing clap derive — diagrams only at //!/impl | Recommended |
| 3 | Additional priority targets handling? | Add 6 files to Revision as additive mandatory targets | Recommended |
| 4 | Drift-anchor rule for narrative diagrams? | Required for flow/decision; state/composition exempt | Recommended |
| 5 | Cross-context diagram rule? | Yes, declared-edges whitelist (later REMOVED in R-4 due to no gate) | Recommended |
| 6 | 20-line cap enforcement? | Allow exceeding for complex modules (with v2 override form) | User |
| 7 | Verification PC for clap deny-list? | Grep/awk-based PC | Recommended |
| 8..12 | Test strategy / performance / security / breaking changes / glossary / ADR | Batch-accepted defaults for doc-only sweep | Recommended |
| 13 | Adversary round 1 verdict? | CONDITIONAL (60/100); apply fixes | User |
| 14 | Adversary round 2 verdict? | CONDITIONAL (72.9/100); apply v3 fixes | User |
| 15 | Adversary round 3 verdict? | PASS (82.1/100) | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | ecc-domain sweep | 5 + Revision ACs | None |
| US-002 | ecc-workflow sweep | 3 + Revision ACs | None |
| US-003 | ecc-ports sweep | 2 + AC-003.2-strict | None |
| US-004 | ecc-app sweep | 4 + Revision ACs | None |
| US-005 | ecc-infra sweep | 2 + AC-005.2-strict | None |
| US-006 | ecc-cli sweep | 2 + AC-006.2-strict | None |
| US-007 | ecc-flock sweep | 3 + AC-007.3-strict | None |
| US-008 | Verification | 3 + Revision ACs | US-001..US-007 |

### Acceptance Criteria (summary)

- Prior (unchanged): 19 ACs across US-001..US-008.
- Revision (new): AC-R1.1..R1.5, AC-R2.1, AC-R2.2, AC-R3.1, AC-R3.2, AC-R5.1, AC-R5.2, AC-R6.1, AC-R6.2, AC-R7.1 (12 ACs).
- Replaced: AC-*.5 family superseded by AC-*.N-strict (7 ACs).
- **Total effective**: 19 prior + 12 Revision − 7 replaced + 7 strict = **31 ACs**.

### Adversary Findings (3 rounds)

| Round | Verdict | Avg Score | Key Issues / Fixes |
|-------|---------|-----------|--------------------|
| 1 | CONDITIONAL | 60/100 | Testability=40 (tautological AC-*.5, fragile PC-015/016, R-4 has no gate) |
| 2 | CONDITIONAL | 72.9/100 | 3 blockers: PC-019 unsatisfiable regex, PC-015 classifier under-specified, 54-vs-115 coverage unclear |
| 3 | PASS | 82.1/100 | All dimensions ≥70; fixtures + CI mandate + floor/ceiling semantic close all blockers |

### Per-Dimension Scores (final, round 3)

| Dimension | Score | Verdict |
|-----------|-------|---------|
| Ambiguity | 80 | PASS |
| Edge Cases | 78 | PASS |
| Scope Creep | 88 | PASS |
| Dependency Gaps | 85 | PASS |
| Testability | 86 | PASS |
| Decision Completeness | 82 | PASS |
| Rollback & Failure | 76 | PASS |

### Artifacts Persisted

| File | Action |
|------|--------|
| `docs/specs/2026-04-17-bl132-ascii-diagrams/spec.md` | Appended `## Revision 2026-04-18` + `## Phase Summary` |
| `docs/specs/2026-04-17-bl132-ascii-diagrams/design.md` | Appended `## Revision 2026-04-18` with 17 new/strengthened PCs |
| `docs/specs/2026-04-18-bl132-ascii-diagram-sweep/campaign.md` | 15 grill-me + adversary decisions |
| `docs/backlog/.locks/BL-132.lock` | Session claim (main repo path) |
| `.git/ecc-workflow/state.json` (worktree-scoped) | Phase: solution; artifact.spec_path = 2026-04-17 spec |
