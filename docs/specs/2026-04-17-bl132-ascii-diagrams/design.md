# Solution: ASCII Diagram Sweep Across 9 ECC Crates

## Spec Reference
Concern: `dev` | Feature: BL-132 ASCII diagram sweep across 9 crates

## File Changes (~85 files, doc-comments only)

| # | Crate | Files | Action | Spec Ref |
|---|-------|-------|--------|----------|
| 1 | ecc-domain | ~30 files | modify (doc-comments) | US-001 |
| 2 | ecc-workflow | ~10 files | modify (doc-comments) | US-002 |
| 3 | ecc-ports | ~12 files | modify (doc-comments) | US-003 |
| 4 | ecc-app | ~20 files | modify (doc-comments) | US-004 |
| 5 | ecc-infra | ~15 files | modify (doc-comments) | US-005 |
| 6 | ecc-cli | ~5 files | modify (doc-comments) | US-006 |
| 7 | ecc-flock | 1 file | modify (doc-comments) | US-007 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | Phase enum state-transition diagram | AC-001.1 | `grep -q 'Idle.*Plan.*Solution' crates/ecc-domain/src/workflow/phase.rs && echo PASS` | PASS |
| PC-002 | build | ecc-domain docs build | AC-001.5 | `cargo doc -p ecc-domain --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-003 | lint | transition flow diagram | AC-002.1 | `grep -qi 'lock.*read.*resolve\|acquire.*read.*transition' crates/ecc-workflow/src/commands/transition.rs && echo PASS` | PASS |
| PC-004 | build | ecc-workflow docs build | AC-002.3 | `cargo doc -p ecc-workflow --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-005 | build | ecc-ports docs build | AC-003.2 | `cargo doc -p ecc-ports --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-006 | lint | dispatch flow diagram | AC-004.1 | `grep -qi 'dispatch\|flow\|decision' crates/ecc-app/src/hook/dispatch.rs && echo PASS` | PASS |
| PC-007 | build | ecc-app docs build | AC-004.4 | `cargo doc -p ecc-app --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-008 | build | ecc-infra docs build | AC-005.2 | `cargo doc -p ecc-infra --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-009 | build | ecc-cli docs build | AC-006.2 | `cargo doc -p ecc-cli --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-010 | lint | FlockGuard RAII annotation | AC-007.1 | `grep -q 'RAII\|Pattern' crates/ecc-flock/src/lib.rs && echo PASS` | PASS |
| PC-011 | build | ecc-flock docs build | AC-007.3 | `cargo doc -p ecc-flock --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-012 | build | Full workspace docs | AC-008.3 | `cargo doc --workspace --no-deps 2>&1; test $? -eq 0 && echo PASS` | PASS |
| PC-013 | lint | Workspace clippy | — | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-014 | build | Workspace builds | — | `cargo build --workspace` | exit 0 |

### Coverage Check
All 19 ACs covered. AC-001.2/3/4, AC-002.2, AC-003.1, AC-004.2/3, AC-005.1, AC-006.1, AC-007.2, AC-008.1/2 covered by the per-crate doc build PCs (cargo doc succeeds = doc-comments valid) and by the triage pass within each crate's implementation.

### E2E Test Plan
None — doc-comments don't affect E2E boundaries.

### E2E Activation Rules
None.

## Test Strategy

Per-crate phases, all parallel-safe:
1. US-001: ecc-domain (PC-001, PC-002) — highest priority
2. US-002: ecc-workflow (PC-003, PC-004) — highest priority
3. US-003: ecc-ports (PC-005)
4. US-004: ecc-app (PC-006, PC-007)
5. US-005: ecc-infra (PC-008)
6. US-006: ecc-cli (PC-009)
7. US-007: ecc-flock (PC-010, PC-011)
8. Final: PC-012, PC-013, PC-014

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | project | modify | docs: ASCII diagram sweep (BL-132) | mandatory |

No ADRs needed (no decisions marked "ADR Needed? Yes").

## SOLID Assessment
PASS — doc-comments only, no code changes.

## Robert's Oath Check
CLEAN — additive documentation, no mess.

## Security Notes
CLEAR — no injection surface in `///` doc-comments.

## Rollback Plan
Revert doc-comment additions per crate in reverse order: flock → cli → infra → app → ports → workflow → domain. All purely additive.

## Bounded Contexts Affected
Doc-comments touch all bounded contexts but modify zero domain logic. No bounded context structural changes.

## Revision 2026-04-18

Adds Pass Conditions enforcing the Revision constraints in `spec.md` (R-1..R-7). Companion to spec Revision 2026-04-18.

### Prior PCs Replaced / Strengthened

| ID | Replaces | New command / implementation |
|----|----------|------------------------------|
| PC-001-v3 | PC-001 | `rg -U --glob '!target/**' '```text\s*\n(///\s[^\n]*\n){1,20}' crates/ecc-domain/src/workflow/phase.rs` AND body contains `-->\s*\w+` |
| PC-006-v3 | PC-006 | `rg -U --glob '!target/**' '```text\s*\n(///\s[^\n]*\n){1,25}' crates/ecc-app/src/hook/dispatch.rs` AND body contains `-->` or `--Y-->`/`--N-->`. Closing-fence anchor via PC-007 `cargo doc`. |

### New PCs (Revision)

| ID | Type | Description | Verifies AC | Command / Implementation | Expected |
|----|------|-------------|-------------|--------------------------|----------|
| PC-015-v3 | lint | Clap-derive deny-list | AC-R1.1 | `scripts/check-clap-derive-diagrams.sh` — awk classifier with brace-depth state machine. Runs fixtures first (exit 1 on fixture fail), then scans `rg -l '#\[derive\((Parser\|Subcommand\|Args\|ValueEnum)\)' crates/ecc-cli crates/ecc-workflow --glob '!target/**'`. Exit 0 if zero violations. | exit 0 |
| PC-016-v3 | lint | Drift-anchor presence | AC-R3.1 | `scripts/check-drift-anchors.sh` — awk scans for fences with `--Y-->`/`--N-->` in body; checks 3 preceding `///` lines for `<!-- keep in sync with:` comment; validates `<fn_name>` via `cargo test --workspace --list`. Fixtures mandatory. | exit 0 |
| PC-017 | lint | Priority-target presence | AC-R2.1 | `for f in crates/ecc-domain/src/{workflow/phase.rs,workflow/state.rs,backlog/lock_file.rs,config/audit.rs} crates/ecc-app/src/merge/mod.rs crates/ecc-workflow/src/commands/phase_gate.rs; do rg -qU --glob '!target/**' '```text[\s\S]*?(-->\|\+-\+\|\|)' $f \|\| { echo "MISSING: $f"; exit 1; }; done` | exit 0 |
| PC-018-v3 | lint | Unicode box-drawing ban | AC-R1.3 | `! rg --pcre2 -U '///.*[\x{2500}-\x{257F}]' crates/ --glob '!target/**'` | exit 0 |
| PC-019-v3 | lint | Fence language tag required | AC-R1.2 | `scripts/check-fence-hints.sh` — awk state machine tracking `///.*```` toggle state. Opening fences must match `///\s*```(text\|rust\|ignore\|no_run\|compile_fail)\s*$`. Closing fences ignored. Fixtures: (a) valid `text` fence → pass, (b) bare ` ``` ` opener → fail, (c) closing fence → ignored. | exit 0 |
| PC-020 | lint | Cap-override comment | AC-R5.1, AC-R5.2 | Awk: for each `text` fence with body line-count >20, verify one of the 3 `///` lines above the opener matches `<!-- cap-override: \d+ lines, reason: \S{5,} -->`. | exit 0 |
| PC-021 | smoke | `--help` sanity | AC-R6.1 | `cargo run -p ecc-cli -- --help 2>&1 \| grep -q 'Usage:' && cargo run -p ecc-workflow -- --help 2>&1 \| grep -q 'Usage:'` | exit 0 |
| PC-022 | lint | CI integration | AC-R6.2 | `rg 'PC-021\|cargo run.*--help.*Usage' .github/workflows/ci.yml` | ≥1 match |
| PC-023 | lint | Attribute-form bypass ban | AC-R1.5 | `! rg -U --glob '!target/**' '#\[doc\s*=.*```text' $(rg -l --glob '!target/**' '#\[derive\((Parser\|Subcommand\|Args\|ValueEnum)\)' crates/ecc-cli crates/ecc-workflow)` | exit 0 |
| PC-024 | lint | Crate-floor ecc-domain | AC-001.5-strict | `test $(rg -c --glob '!target/**' '```text' crates/ecc-domain/src/ \| awk -F: '{s+=$2}END{print s}') -ge 20` | exit 0 |
| PC-025 | lint | Crate-floor ecc-workflow | AC-002.3-strict | `test $(rg -c --glob '!target/**' '```text' crates/ecc-workflow/src/ \| awk -F: '{s+=$2}END{print s}') -ge 5` | exit 0 |
| PC-026 | lint | Crate-floor ecc-ports Pattern | AC-003.2-strict | `test $(rg -c --glob '!target/**' '^/// # Pattern' crates/ecc-ports/src/ \| awk -F: '{s+=$2}END{print s}') -ge 8` | exit 0 |
| PC-027 | lint | Crate-floor ecc-app | AC-004.4-strict | `test $(rg -c --glob '!target/**' '```text' crates/ecc-app/src/ \| awk -F: '{s+=$2}END{print s}') -ge 10` | exit 0 |
| PC-028 | lint | Crate-floor ecc-infra Pattern | AC-005.2-strict | `test $(rg -c --glob '!target/**' '^/// # Pattern' crates/ecc-infra/src/ \| awk -F: '{s+=$2}END{print s}') -ge 8` | exit 0 |
| PC-029 | lint | Crate-floor ecc-cli | AC-006.2-strict | `test $(rg -c --glob '!target/**' '```text' crates/ecc-cli/src/ \| awk -F: '{s+=$2}END{print s}') -ge 2` | exit 0 |
| PC-030 | lint | Crate-floor ecc-flock | AC-007.3-strict | `rg -q --glob '!target/**' '```text' crates/ecc-flock/src/lib.rs && rg -q --glob '!target/**' '^/// # Pattern' crates/ecc-flock/src/lib.rs` | exit 0 |
| PC-031 | lint | Minimum shippable subset | AC-R7.1 | Implementation-time guidance only — enforced by commit message discipline and reviewer. No grep gate. | (manual) |

### Coverage Check Update

All 19 prior ACs retained (with AC-*.5 family superseded by AC-*.N-strict). 12 new Revision ACs (AC-R1.1..AC-R7.1) covered:

| AC | PC |
|----|----|
| AC-R1.1 | PC-015-v3 |
| AC-R1.2 | PC-019-v3 |
| AC-R1.3 | PC-018-v3 |
| AC-R1.4 | PC-015-v3 / PC-016-v3 / PC-019-v3 fixture gates |
| AC-R1.5 | PC-023 |
| AC-R2.1 | PC-017 |
| AC-R2.2 | PC-024..PC-030 (floor) + manual ceiling tracking |
| AC-R3.1 | PC-016-v3 |
| AC-R3.2 | PC-016-v3 fixture gate |
| AC-R5.1, AC-R5.2 | PC-020 |
| AC-R6.1 | PC-021 |
| AC-R6.2 | PC-022 |
| AC-R7.1 | PC-031 (manual) |
| AC-001.5-strict..AC-007.3-strict | PC-024..PC-030 |

### Rollback Addendum

- Per-crate commits independent. PC-015-v3 failure → `git revert` only offending crate's commits.
- PC-021 failure (`--help` broken) → emergency revert of last commits touching any clap-derive file.
- CI adds PC-021 + PC-022 gates — main stays green.
- Partial ship allowed per AC-R7.1: US-001 + US-002 mandatory. US-003..US-008 deferrable with follow-up issue reference.

### Wave Ordering

Prior per-crate wave plan retained. New PC-015-v3..PC-031 execute in final verification wave with PC-012..PC-014.

### Supplement 2026-04-18 (planner gap analysis)

Classifier scripts, fixtures, ADR, and CI amendment were referenced by Revision ACs but not committed as File Changes. This supplement closes that gap.

#### Supplementary File Changes

| # | Path | Action | Purpose | Spec Ref |
|---|------|--------|---------|----------|
| 8  | `scripts/check-clap-derive-diagrams.sh` | create | awk classifier w/ brace-depth state machine | AC-R1.1, AC-R1.4 |
| 9  | `scripts/check-fence-hints.sh` | create | awk state machine enforcing language tag on openers | AC-R1.2, AC-R1.4 |
| 10 | `scripts/check-drift-anchors.sh` | create | awk anchor-detection + `cargo test --list` validation | AC-R3.1, AC-R1.4 |
| 11 | `scripts/fixtures/bl132/clap_impl_ok.rs` | create | impl-block diagram in clap-derive file → pass | AC-R1.4 |
| 12 | `scripts/fixtures/bl132/clap_module_ok.rs` | create | `//!` header diagram in clap-derive file → pass | AC-R1.4 |
| 13 | `scripts/fixtures/bl132/clap_struct_fail.rs` | create | diagram on `///` above `#[derive(Parser)]` → fail | AC-R1.4 |
| 14 | `scripts/fixtures/bl132/fence_text_ok.rs` | create | `text`-tagged fence → pass | AC-R1.2 |
| 15 | `scripts/fixtures/bl132/fence_bare_fail.rs` | create | bare ` ``` ` opener → fail | AC-R1.2 |
| 16 | `scripts/fixtures/bl132/fence_closing_ok.rs` | create | closing fence ignored by classifier | AC-R1.2 |
| 17 | `scripts/fixtures/bl132/drift_missing_fail.rs` | create | `--Y-->` fence, no anchor → fail | AC-R3.2 |
| 18 | `scripts/fixtures/bl132/drift_present_ok.rs` | create | same fence with anchor → pass | AC-R3.2 |
| 19 | `docs/adr/ADR-NNNN-clap-derive-deny-list.md` | create | Decision record for R-1 | Decision #4 |
| 20 | `.github/workflows/ci.yml` | modify | add `--help` smoke step (PC-021) in validate job | AC-R6.1, AC-R6.2 |

ADR number: /implement resolves via `ls docs/adr/` + `git log --all -- docs/adr/` scan for the next free integer. Use `ADR-NNNN` placeholder throughout until creation time.

#### Supplementary Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-032 | lint | Classifier scripts exist + executable | AC-R1.4 | `for f in scripts/check-clap-derive-diagrams.sh scripts/check-fence-hints.sh scripts/check-drift-anchors.sh; do test -x $f \|\| exit 1; done` | exit 0 |
| PC-033 | lint | Fixture corpus complete (≥8 files) | AC-R1.4, AC-R3.2 | `test $(ls scripts/fixtures/bl132/*.rs 2>/dev/null \| wc -l) -ge 8` | exit 0 |
| PC-034 | lint | ADR present with required sections | Decision #4 | `f=$(ls docs/adr/ADR-*-clap-derive-deny-list.md 2>/dev/null \| head -1); test -f "$f" && grep -qE '^## (Context\|Decision\|Consequences)' "$f"` | exit 0 |
| PC-035 | lint | CI workflow wires --help smoke | AC-R6.2 | `rg -q 'cargo run -p ecc-cli -- --help' .github/workflows/ci.yml && rg -q 'cargo run -p ecc-workflow -- --help' .github/workflows/ci.yml` | exit 0 |

#### TDD Order (5 steps)

A. **Scaffolding (unblocks gates)** — commit classifier scripts + fixtures together. Write RED fixture tests first, then GREEN awk implementations. Commit: `feat(scripts): add BL-132 diagram classifiers and fixtures`. Satisfies PC-032, PC-033.

B. **CI wire-up** — amend `.github/workflows/ci.yml` validate job. Commit: `ci: wire BL-132 --help smoke test (PC-021)`. Satisfies PC-022, PC-035.

C. **ADR authoring** — create `docs/adr/` + ADR-NNNN for clap-derive deny-list. Commit: `docs(adr): clap-derive diagram deny-list (ADR-NNNN)`. Satisfies PC-034.

D. **Per-crate sweep (Waves 1–3)** — as prior wave plan: Wave 1 US-001 + US-002 (priority per R-7), Wave 2 US-003 + US-004 + US-005, Wave 3 US-006 + US-007. Per-crate commits.

E. **Final verification** — run PC-012..PC-014, PC-015-v3..PC-031-v2, PC-032..PC-038. Update CHANGELOG. Commit: `docs(changelog): BL-132 ASCII diagram sweep`.

### Supplement v2 (post-adversary-round-1 fixes)

Round-1 solution-adversary verdict: FAIL (58/100). All 11 blockers addressed:

#### Fix 1: ADR path corrected

`docs/adrs/` → `docs/adr/` (singular, matching repo convention). ADR number resolved at /implement time via `ls docs/adr/ | grep -E '^[0-9]{4}' | sort -V | tail -1`. /implement must re-verify free number at authoring time to tolerate drift. Spec Revision uses `ADR-NNNN` placeholder (no hardcoding).

#### Fix 2: Step A gate placed BEFORE Wave 1

Explicit topological order: **Step A → Step B → Step C → (PC-032 + PC-033-v2 + PC-034 + PC-035 gate) → Step D → Step E**. PC-032/033-v2 RELOCATED from Step E to run immediately after Step C, before Wave 1 begins. Prevents Wave 1 PC-015-v3/PC-016-v3 returning exit 127 (file not found) instead of actionable violation output.

#### Fix 3: PC-031 converted from manual to mechanical

Replace PC-031 with:

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-031-v2 | lint | Minimum shippable subset (US-001 + US-002) | AC-R7.1 | `BASE="${GITHUB_BASE_REF:-main}"; git fetch --no-tags --depth=200 origin "$BASE" 2>/dev/null \|\| true; BASE_SHA=$(git rev-parse --verify "origin/$BASE" 2>/dev/null \|\| git rev-parse --verify "$BASE"); MB=$(git merge-base HEAD "$BASE_SHA"); git log --format='%B' "$MB"..HEAD \| rg -q 'BL-132' && git diff --name-only "$MB"..HEAD \| rg -q 'crates/ecc-domain/src/' && git diff --name-only "$MB"..HEAD \| rg -q 'crates/ecc-workflow/src/'` | exit 0 |

Mechanical: verifies commit messages contain BL-132 AND diff touches both ecc-domain and ecc-workflow source trees.

#### Fix 4: CHANGELOG gate added

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-037 | lint | CHANGELOG has BL-132 sweep entry (unique sentinel) | Doc plan | `rg -q 'BL-132-sweep-shipped' CHANGELOG.md` | exit 0 |

#### Fix 5: Fixture corpus gate — exact-filename match

Replace PC-033 `≥8` floor with:

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-033-v2 | lint | Fixture corpus exact match | AC-R1.4, AC-R3.2 | `for f in clap_impl_ok clap_module_ok clap_struct_fail fence_text_ok fence_bare_fail fence_closing_ok drift_missing_fail drift_present_ok; do test -f scripts/fixtures/bl132/$f.rs \|\| exit 1; done` | exit 0 |

#### Fix 6: Tool-version guards (gawk + rg --pcre2)

Classifier scripts mandate GNU awk (`gawk`). Shell wrappers invoke `gawk` explicitly. CI installs `gawk` and PCRE2-enabled ripgrep before other PCs run. macOS devs install via `brew install gawk`.

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-038 | lint | Tool prerequisites available | prerequisite | `command -v gawk >/dev/null && rg --pcre2 --version 2>&1 \| rg -q 'PCRE2'` | exit 0 |

#### Fix 7: PC-020 boundary clarified (off-by-one)

R-5 prose: "Diagrams **exceeding** 20 lines require cap-override". PC-020 awk condition: `if (body_count > 20) require_override();` — ≤20 lines allowed without override, ≥21 requires override. Documented explicitly in PC-020 description.

#### Fix 8: CI amendment in a separate PR (process rule)

Step B's CI amendment (`.github/workflows/ci.yml`) MUST land in its own standalone PR BEFORE any Wave 1 content PR. Eliminates chicken-and-egg: the CI file being modified is the same one validating its PR.

New AC:
- **AC-R6.3** — The CI amendment adding PC-021 smoke test lands in a standalone PR merged before Wave 1 content PRs open. Rollback: `git revert` of the CI amendment PR's merge commit; `git push` as hotfix.

**Operational note for ECC worktree auto-merge**: ECC's `session:end:worktree-merge` hook auto-merges a worktree at session end. To honour AC-R6.3, /implement MUST end the session after Step B commits (triggering the hook to merge the CI amendment alone as PR #1), then start a fresh session/worktree for Steps C/D/E (which merge as a second PR). Alternative: use `ecc bypass grant --hook session:end:worktree-merge --reason "multi-step BL-132 PR split"` to suppress auto-merge and manually create 2 PRs from the same worktree. Either path satisfies AC-R6.3.

#### Fix 9: ADR body justifies `scripts/` vs `xtask/`

ADR-NNNN body MUST include a "Why shell + awk, not Rust xtask" section justifying: (a) classifiers are language-agnostic lint policy matching prior repo patterns (`bump-version.sh` etc.); (b) awk classifiers run against raw source text without requiring cargo build — faster feedback on uncompiled branches; (c) migration path: if ECC later standardises lint tooling under `xtask/`, classifiers follow.

#### Fix 10: Doc Update Plan expanded

The Doc Update Plan (prior design table) gains 2 rows:

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 2 | `docs/adr/ADR-NNNN-clap-derive-deny-list.md` | project | create | Decision record with Context/Decision/Consequences/Alternatives + scripts-vs-xtask justification | Decision #4 |
| 3 | `.github/workflows/ci.yml` | project | modify | `--help` smoke step in validate job (PC-021) | AC-R6.2, AC-R6.3 |

#### Fix 11: CI amendment rollback plan

If PC-021 has a bug after Step B lands, ALL subsequent PRs are blocked. Emergency escape: `git revert <Step-B-SHA>` pushed via hotfix PR directly to main. Diagram content shipped between Step B and revert remains unchanged.

#### Updated PC total

After Supplement v2: PC-001..PC-014, PC-015-v3..PC-030, PC-031-v2, PC-032, PC-033-v2, PC-034..PC-038 = **39 Pass Conditions** covering **40 ACs** (39 prior + AC-R6.3 new).

#### Updated TDD Order (post-v2)

```
Step A: commit classifier scripts + fixtures        (separate commit)
Step B: SEPARATE PR — amend ci.yml                  (standalone merge)
Step C: create ADR-NNNN (resolve N at commit time)  (separate commit)
Gate:   PC-032 + PC-033-v2 + PC-034 + PC-035 + PC-038 MUST pass before Step D
Step D Wave 1: US-001 (ecc-domain) + US-002 (ecc-workflow) — AC-R7.1 mandatory
Step D Wave 2: US-003 + US-004 + US-005 (parallel)
Step D Wave 3: US-006 + US-007 (parallel)
Step E: final verification — all remaining PCs + CHANGELOG update → PC-037
```

## Phase Summary (2026-04-18 /design session)

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 0 |
| Robert's Oath | CLEAN | 0 warnings |
| Security | CLEAR | 0 findings |
| AC Coverage | 0 uncovered | 40 ACs → 39 PCs |

### Adversary Findings (3 rounds)

| Round | Verdict | Avg Score | Key Delta |
|-------|---------|-----------|-----------|
| 1 | FAIL | 58/100 | Hard blocker: ADR path typo. 11 blockers total |
| 2 | CONDITIONAL | 72.1/100 | 5 fixes required (PC-017 regex, PC-037 sentinel, PC-031-v2 robustness, line 168, AC-R6.3 operational) |
| 3 | **PASS** | **78.4/100** | Fragility 52→74 (+22); Missing PCs 65→75 (+10) |

### Per-Dimension Final Scores (round 3)

| Dimension | Score | Verdict |
|-----------|-------|---------|
| AC Coverage | 90 | PASS |
| Execution Order | 85 | PASS |
| Fragility | 74 | PASS |
| Rollback | 78 | PASS |
| Architecture | 92 | PASS |
| Blast Radius | 55 | PASS (floor) |
| Missing PCs | 75 | PASS |
| Doc Plan | 78 | PASS |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/**/*.rs` (~30 files) | modify (doc-comments) | US-001 |
| 2 | `crates/ecc-workflow/src/**/*.rs` (~10 files) | modify (doc-comments) | US-002 |
| 3 | `crates/ecc-ports/src/**/*.rs` (~12 files) | modify (doc-comments) | US-003 |
| 4 | `crates/ecc-app/src/**/*.rs` (~20 files) | modify (doc-comments) | US-004 |
| 5 | `crates/ecc-infra/src/**/*.rs` (~15 files) | modify (doc-comments) | US-005 |
| 6 | `crates/ecc-cli/src/**/*.rs` (~5 files) | modify (doc-comments) | US-006 |
| 7 | `crates/ecc-flock/src/lib.rs` | modify (doc-comments) | US-007 |
| 8 | `scripts/check-clap-derive-diagrams.sh` | create | AC-R1.1, AC-R1.4 |
| 9 | `scripts/check-fence-hints.sh` | create | AC-R1.2, AC-R1.4 |
| 10 | `scripts/check-drift-anchors.sh` | create | AC-R3.1, AC-R1.4 |
| 11-18 | `scripts/fixtures/bl132/*.rs` (8 fixtures) | create | AC-R1.4, AC-R3.2 |
| 19 | `docs/adr/ADR-NNNN-clap-derive-deny-list.md` | create | Decision #4 |
| 20 | `.github/workflows/ci.yml` | modify | AC-R6.2, AC-R6.3 |
| — | `CHANGELOG.md` | modify (at Step E) | Doc plan |

### Artifacts Persisted

| File | Section |
|------|---------|
| `docs/specs/2026-04-17-bl132-ascii-diagrams/spec.md` | prior + `## Revision 2026-04-18` + `## Phase Summary` |
| `docs/specs/2026-04-17-bl132-ascii-diagrams/design.md` | prior + `## Revision 2026-04-18` + `### Supplement 2026-04-18` + `### Supplement v2` + `## Phase Summary` |
| `docs/specs/2026-04-18-bl132-ascii-diagram-sweep/campaign.md` | 18 grill-me + adversary decisions across /spec-dev + /design |
| `.git/ecc-workflow/state.json` (worktree-scoped) | Phase: implement; artifact.design_path = 2026-04-17 design |
