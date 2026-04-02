# Solution: BL-116 cargo-mutants Mutation Testing Integration

## Spec Reference
Concern: dev, Feature: Integrate cargo-mutants into the ECC development workflow

## File Changes Table

| # | File | Action | US | ACs | Layer |
|---|------|--------|----|-----|-------|
| 1 | `mutants.toml` | Create | US-001 | AC-001.1 | Configuration |
| 2 | `.gitignore` | Modify | US-001 | AC-001.2 | Configuration |
| 3 | `xtask/Cargo.toml` | Modify | US-004 | AC-004.4 | Developer tooling |
| 4 | `xtask/src/mutants.rs` | Create | US-004 | AC-004.1, AC-004.2, AC-004.3, AC-004.4 | Developer tooling |
| 5 | `xtask/src/main.rs` | Modify | US-004 | AC-004.1 | Developer tooling |
| 6 | `commands/mutants.md` | Create | US-005 | AC-005.1, AC-005.2, AC-005.3, AC-005.4 | Claude Code commands (flat file, overrides spec's `commands/mutants/COMMAND.md` — matches existing pattern: `verify.md`, `backlog.md`) |
| 7 | `commands/verify.md` | Modify | US-006 | AC-006.1, AC-006.2, AC-006.3, AC-006.4 | Claude Code commands |
| 8 | `.github/workflows/ci.yml` | Modify | US-007 | AC-007.1, AC-007.2, AC-007.3, AC-007.4, AC-007.5, AC-007.6 | CI |
| 9 | `CLAUDE.md` | Modify | US-001 | AC-001.3 | Documentation |
| 10 | `docs/sources.md` | Modify | US-001 | AC-001.4 | Documentation |
| 11 | `docs/audits/mutation-baseline-ecc-domain.md` | Create | US-002 | AC-002.1, AC-002.2, AC-002.3, AC-002.4 | Documentation |
| 12 | `docs/audits/mutation-baseline-ecc-app.md` | Create | US-003 | AC-003.1, AC-003.2, AC-003.3, AC-003.4 | Documentation |
| 13 | `docs/audits/mutation-scores.md` | Create | US-008 | AC-008.1, AC-008.2, AC-008.3, AC-008.4 | Documentation |

## Pass Conditions Table

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | `mutants.toml` exists with required fields | AC-001.1 | `grep -q 'test_tool = "nextest"' mutants.toml && grep -q 'timeout = 120' mutants.toml && grep -q 'ecc-domain' mutants.toml && grep -q 'ecc-app' mutants.toml && echo PASS` | PASS |
| PC-002 | build | `.gitignore` excludes `mutants.out/` | AC-001.2 | `grep -q 'mutants.out/' .gitignore && echo PASS` | PASS |
| PC-003 | unit | `xtask/src/mutants.rs` defines Mutants struct with all flags | AC-004.1 | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo test -p xtask -- mutants && echo PASS` | PASS |
| PC-004 | unit | xtask mutants builds command args with defaults from config | AC-004.2 | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo test -p xtask -- mutants::tests::builds_default_args && echo PASS` | PASS |
| PC-005 | unit | xtask mutants --in-diff passes origin/main diff base | AC-004.3 | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo test -p xtask -- mutants::tests::in_diff_uses_origin_main && echo PASS` | PASS |
| PC-006 | unit | xtask mutants detects missing cargo-mutants and errors | AC-004.4 | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo test -p xtask -- mutants::tests::errors_when_not_installed && echo PASS` | PASS |
| PC-007 | build | xtask registers Mutants subcommand and compiles | AC-004.1 | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo build -p xtask && echo PASS` | PASS |
| PC-008 | build | `/mutants` command file exists with required structure | AC-005.1 | `test -f commands/mutants.md && grep -q 'description:' commands/mutants.md && grep -q 'package' commands/mutants.md && grep -q 'diff' commands/mutants.md && echo PASS` | PASS |
| PC-009 | build | `/mutants` command references xtask for targeted run | AC-005.2, AC-005.3 | `grep -q 'cargo xtask mutants' commands/mutants.md && grep -q '\-\-in-diff' commands/mutants.md && echo PASS` | PASS |
| PC-010 | build | `/mutants` command includes result summary format | AC-005.4 | `grep -q 'killed' commands/mutants.md && grep -q 'survived' commands/mutants.md && grep -q 'timeout' commands/mutants.md && echo PASS` | PASS |
| PC-011 | build | `/verify` updated with `--mutation` flag support | AC-006.1, AC-006.2 | `grep -q '\-\-mutation' commands/verify.md && grep -q 'Mutation' commands/verify.md && echo PASS` | PASS |
| PC-012 | build | `/verify` mutation step is non-blocking | AC-006.3, AC-006.4 | `grep -q 'non-blocking\|does NOT block\|do not block' commands/verify.md && echo PASS` | PASS |
| PC-013 | build | CI mutation job exists with continue-on-error | AC-007.1, AC-007.2 | `grep -A 20 'name: Mutation Testing' .github/workflows/ci.yml \| grep -q 'continue-on-error: true' && echo PASS` | PASS |
| PC-014 | build | CI mutation job uploads artifact | AC-007.3 | `grep -q 'upload-artifact' .github/workflows/ci.yml && grep -q 'mutants' .github/workflows/ci.yml && echo PASS` | PASS |
| PC-015 | build | CI mutation job has 30min timeout and separate cache key | AC-007.4 | `grep -q 'timeout-minutes: 30' .github/workflows/ci.yml && grep -q 'cargo-mutants' .github/workflows/ci.yml && echo PASS` | PASS |
| PC-016 | build | CI mutation job installs pinned cargo-mutants version | AC-007.5 | `grep -q 'cargo install cargo-mutants' .github/workflows/ci.yml && echo PASS` | PASS |
| PC-017 | build | CI mutation job uses full mutation with fetch-depth 0 | AC-007.6 | `grep -q 'fetch-depth: 0' .github/workflows/ci.yml && echo PASS` | PASS |
| PC-018 | build | CLAUDE.md documents cargo mutants command | AC-001.3 | `grep -q 'cargo mutants' CLAUDE.md && echo PASS` | PASS |
| PC-019 | build | docs/sources.md lists cargo-mutants | AC-001.4 | `grep -q 'cargo-mutants' docs/sources.md && echo PASS` | PASS |
| PC-020 | build | ecc-domain baseline report exists with required sections | AC-002.2, AC-002.3, AC-002.4 | `test -f docs/audits/mutation-baseline-ecc-domain.md && grep -q 'killed' docs/audits/mutation-baseline-ecc-domain.md && grep -q 'survived' docs/audits/mutation-baseline-ecc-domain.md && grep -q 'timed.out\|timeout' docs/audits/mutation-baseline-ecc-domain.md && echo PASS` | PASS |
| PC-021 | build | ecc-app baseline report exists with required sections | AC-003.2, AC-003.3, AC-003.4 | `test -f docs/audits/mutation-baseline-ecc-app.md && grep -q 'killed' docs/audits/mutation-baseline-ecc-app.md && grep -q 'survived' docs/audits/mutation-baseline-ecc-app.md && grep -q 'timed.out\|timeout' docs/audits/mutation-baseline-ecc-app.md && echo PASS` | PASS |
| PC-022 | build | Mutation scores dashboard exists with per-crate table | AC-008.1, AC-008.2, AC-008.3, AC-008.4 | `test -f docs/audits/mutation-scores.md && grep -q 'ecc-domain' docs/audits/mutation-scores.md && grep -q 'ecc-app' docs/audits/mutation-scores.md && grep -q 'Module' docs/audits/mutation-scores.md && echo PASS` | PASS |
| PC-023 | build | xtask Cargo.toml includes which dependency | AC-004.4 | `grep -q 'which' xtask/Cargo.toml && echo PASS` | PASS |
| PC-024 | lint | Clippy passes with zero warnings | all | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo clippy -- -D warnings && echo PASS` | PASS |
| PC-025 | build | Full workspace builds | all | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo build && echo PASS` | PASS |
| PC-026 | build | ECC validates commands (including new /mutants) | AC-005.1 | `cd /Users/titouanlebocq/code/everything-claude-code/.claude/worktrees/ecc-session-20260331-bl116-cargo-mutants && cargo build --release && ./target/release/ecc validate commands && echo PASS` | PASS |

## TDD Order Rationale

The phases are ordered by dependency chain and TDD feasibility:

**Phase 1 (Configuration -- files 1-2):** `mutants.toml` and `.gitignore` are pure config with no code dependencies. They are prerequisites for everything else. Verified by content checks (PC-001, PC-002).

**Phase 2 (xtask subcommand -- files 3-4):** The xtask `mutants.rs` module is the only Rust code in this feature. It follows the existing `deploy.rs` pattern. Tests are written first for command argument building, diff-mode flag handling, and missing-binary detection. The main.rs registration wires it in. This is the core testable unit. Verified by unit tests (PC-003 through PC-006) and build (PC-007).

**Phase 3 (Slash commands -- files 5-6):** `/mutants` command and `/verify` modification depend on xtask existing (they reference `cargo xtask mutants`). These are markdown files validated by content checks and `ecc validate commands` (PC-008 through PC-012, PC-025).

**Phase 4 (CI -- file 7):** The CI mutation job depends on `mutants.toml` existing (Phase 1). It is independent of xtask (CI runs cargo-mutants directly). Verified by YAML content checks (PC-013 through PC-017).

**Phase 5 (Documentation -- files 8-12):** CLAUDE.md, sources.md, baseline reports, and dashboard depend on all prior phases being complete (baseline reports reference actual mutation runs from Phase 1 config). Verified by content checks (PC-018 through PC-022).

**Phase 6 (Final gates -- no new files):** `which` dependency check (PC-023), clippy (PC-024), full build (PC-025), and ECC command validation (PC-026).

### Phase-Layer mapping

- **Phase 1:** Configuration only
- **Phase 2:** Adapter (xtask is developer tooling, wraps external binary)
- **Phase 3:** Framework (Claude Code command definitions)
- **Phase 4:** Framework (CI pipeline)
- **Phase 5:** Documentation only
- **Phase 6:** Verification gates

No phase crosses more than 2 layers. Phase 2 is the only phase with Rust unit tests; all other phases use content/build verification.

### Commit cadence

- Phase 1: `chore: add mutants.toml and gitignore exclusion`
- Phase 2: `test: add xtask mutants unit tests` -> `feat: add xtask mutants subcommand` -> `refactor: improve xtask mutants` (if needed)
- Phase 3: `feat: add /mutants command and /verify --mutation flag`
- Phase 4: `ci: add non-blocking mutation testing job`
- Phase 5: `docs: add mutation testing baseline reports and dashboard`
- Phase 6: verification only, no commit needed

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| cargo-mutants not installed on dev machine | Phase 2 tests use arg-building logic, not actual binary invocation | Test command construction, not execution |
| CI mutation job too slow | Non-blocking, 30min cap | continue-on-error: true already specified |
| Baseline reports are point-in-time snapshots | Reports become stale | Dashboard (US-008) tracks scores over time |
| ECC validate commands rejects new /mutants format | Build gate fails | Check existing command format before writing |

### Coverage Check

All 34 ACs covered by 25 PCs. Zero uncovered ACs.

| AC Range | Covering PCs |
|----------|-------------|
| AC-001.1-4 | PC-001, PC-002, PC-018, PC-019 |
| AC-002.1-4 | PC-020 |
| AC-003.1-4 | PC-021 |
| AC-004.1-4 | PC-003, PC-004, PC-005, PC-006, PC-007 |
| AC-005.1-4 | PC-008, PC-009, PC-010, PC-025 |
| AC-006.1-4 | PC-011, PC-012 |
| AC-007.1-6 | PC-013, PC-014, PC-015, PC-016, PC-017 |
| AC-008.1-4 | PC-022 |

### E2E Test Plan

No E2E tests needed — pure tooling integration. No hexagonal boundaries crossed, no ports or adapters touched.

### E2E Activation Rules

None — no E2E tests to activate.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Project | Modify | Add cargo mutants and cargo xtask mutants to test commands | US-001, AC-001.3 |
| 2 | docs/sources.md | Project | Modify | Add cargo-mutants under Trial/testing | US-001, AC-001.4 |
| 3 | docs/audits/mutation-baseline-ecc-domain.md | Report | Create | Baseline mutation scores for ecc-domain | US-002, AC-002.2-4 |
| 4 | docs/audits/mutation-baseline-ecc-app.md | Report | Create | Baseline mutation scores for ecc-app | US-003, AC-003.2-4 |
| 5 | docs/audits/mutation-scores.md | Report | Create | Living dashboard with per-crate scores | US-008, AC-008.1-4 |
| 6 | docs/adr/ADR-NNNN-mutation-testing-tool-choice.md | ADR | Create | Tool choice + crate scoping | Decision 1, 2 |
| 7 | docs/adr/ADR-NNNN-mutation-testing-thresholds.md | ADR | Create | Threshold strategy (aspirational, TBD) | Decision 3 |
| 8 | CHANGELOG.md | Project | Modify | Add mutation testing integration entry | All |

## SOLID Assessment

**PASS** — 1 MEDIUM, 2 LOW informational findings (none blocking).

- F-001 (LOW): `mutants.rs::run` should stay under 50 lines; extract helpers if needed.
- F-002 (LOW): DIP trade-off acceptable for tooling layer. Document rationale in code comment.
- F-003 (MEDIUM): Use `which::which` for binary detection instead of `Command::new().status()`. Mandated for testability.

## Robert's Oath Check

**CLEAN** — No harmful code, no mess, adequate proof (25 PCs for 34 ACs), small releases (6 phases with atomic commits), continuous improvement (addresses TEST-008 audit finding). Path inconsistency (mutants.md vs mutants/COMMAND.md) to resolve at implementation.

## Security Notes

**CLEAR** — No injection vectors (std::process::Command with typed args, no shell), least-privilege CI (contents: read), no secrets in config, safe artifact upload (mutants.out/ contains only test results). Pinned cargo-mutants version for supply chain hygiene.

## Rollback Plan

Reverse dependency order — if implementation fails, undo in this order:

1. Remove `docs/audits/mutation-scores.md`
2. Remove `docs/audits/mutation-baseline-ecc-app.md`
3. Remove `docs/audits/mutation-baseline-ecc-domain.md`
4. Revert `docs/sources.md` changes
5. Revert `CLAUDE.md` changes
6. Revert `.github/workflows/ci.yml` changes
7. Revert `commands/verify.md` changes
8. Remove `commands/mutants.md`
9. Revert `xtask/src/main.rs` changes
10. Remove `xtask/src/mutants.rs`
11. Revert `xtask/Cargo.toml` changes (remove `which` dependency)
12. Revert `.gitignore` changes
13. Remove `mutants.toml`

## CI Trigger Clarification

The mutation CI job runs on `pull_request` trigger only (matching existing ci.yml behavior). This provides PR-scoped mutation feedback. Merge-time tracking via `push` trigger on `main` is explicitly deferred — it would require a separate workflow or trigger expansion that is outside this spec's scope. Baseline tracking is manual via US-002/US-003 reports.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 3 (1 MEDIUM, 2 LOW) |
| Robert | CLEAN | 1 (path inconsistency — resolved) |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Score (R1→R2) | Verdict | Key Rationale |
|-----------|---------------|---------|---------------|
| Completeness | 78→92 | PASS | xtask/Cargo.toml added, which crate PC added |
| Correctness | n/a→88 | PASS | PC-013 rewritten for mutation job context |
| Consistency | n/a→88 | PASS | Command path override stated explicitly |
| Feasibility | n/a→93 | PASS | All implementable with current tooling |
| Testability | n/a→90 | PASS | 26 PCs executable and concrete |
| Security | n/a→95 | PASS | No injection, least-privilege, no secrets |
| Maintainability | n/a→91 | PASS | 6-phase atomic commits, rollback plan complete |
| Traceability | n/a→88 | PASS | File→AC, PC→AC, AC→PC all linked |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | mutants.toml | create | US-001, AC-001.1 |
| 2 | .gitignore | modify | US-001, AC-001.2 |
| 3 | xtask/Cargo.toml | modify | US-004, AC-004.4 |
| 4 | xtask/src/mutants.rs | create | US-004, AC-004.1-4 |
| 5 | xtask/src/main.rs | modify | US-004, AC-004.1 |
| 6 | commands/mutants.md | create | US-005, AC-005.1-4 |
| 7 | commands/verify.md | modify | US-006, AC-006.1-4 |
| 8 | .github/workflows/ci.yml | modify | US-007, AC-007.1-6 |
| 9 | CLAUDE.md | modify | US-001, AC-001.3 |
| 10 | docs/sources.md | modify | US-001, AC-001.4 |
| 11 | docs/audits/mutation-baseline-ecc-domain.md | create | US-002, AC-002.1-4 |
| 12 | docs/audits/mutation-baseline-ecc-app.md | create | US-003, AC-003.1-4 |
| 13 | docs/audits/mutation-scores.md | create | US-008, AC-008.1-4 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-31-cargo-mutants/spec.md | Full spec + Phase Summary |
| docs/specs/2026-03-31-cargo-mutants/design.md | Full design + Phase Summary |
