# Solution Adversary Report

## Summary
Verdict: CONDITIONAL (avg: 73/100)
Rounds: 1 of 3

## Dimension Results
| # | Dimension | Score | Verdict | Critical Findings |
|---|-----------|-------|---------|-------------------|
| 1 | AC Coverage | 62 | CONDITIONAL | AC-006.1 through AC-006.4 have no automated PC; "catchup.md (manual)" is not a verifiable pass condition |
| 2 | Execution Order | 85 | PASS | Minor: PC-037 appears in both Phase 2 and Phase 11 |
| 3 | Fragility | 72 | PASS | Test module paths are brittle; HookPorts ~30 site fixup is fragile but acknowledged |
| 4 | Rollback Adequacy | 80 | PASS | Covers all file changes in reverse order; minor gap on test fixup sites |
| 5 | Architecture Compliance | 90 | PASS | All uncle-bob findings addressed; domain stays I/O-free; DIP fixed |
| 6 | Blast Radius | 65 | CONDITIONAL | 32 HookPorts construction sites affected but solution says "~15"; undercounted by 2x |
| 7 | Missing Pass Conditions | 60 | CONDITIONAL | No cargo fmt PC; catchup ACs lack automated PCs; no integration PC despite port wiring |
| 8 | Doc Plan Completeness | 75 | PASS | CHANGELOG present; no ADRs needed; minor gap on MODULE-SUMMARIES |

## Uncovered ACs
| AC | Description (from spec) | Suggested PC |
|----|------------------------|--------------|
| AC-006.1 | Catchup shows four rates as percentages | PC-038: Verify catchup.md contains `ecc metrics summary --session` invocation pattern and percentage formatting instructions |
| AC-006.2 | No events = "No harness metrics recorded" | PC-039: Verify catchup.md contains the exact fallback string "No harness metrics recorded for this session." |
| AC-006.3 | DB unavailable = skip silently | PC-040: Verify catchup.md contains error-handling instruction (skip on failure) |
| AC-006.4 | N/A for metric types with no events | Covered implicitly by PC-026 (--json outputs Option<f64> which serializes to null) but no direct catchup display test |

## Detailed Findings

### 1. AC Coverage
- **Finding**: AC-006.1 through AC-006.4 are mapped to "catchup.md (manual)" in the coverage check table. A markdown command file edit is not a verifiable pass condition. The coverage check claims "Zero uncovered ACs" but 4 ACs have no command that can be run to verify them.
- **Evidence**: Solution line 74: `AC-006.1-006.4 -> catchup.md (manual)`. There is no PC-NNN entry with a Command column for these 4 ACs.
- **Recommendation**: Add at least one structural PC that greps catchup.md for required content (the `ecc metrics summary --session` invocation, percentage formatting, N/A handling, and "No harness metrics" fallback string). Alternatively, add a PC that runs a catchup.md validation check.

### 2. Execution Order
- **Finding**: PC-037 (cargo build --workspace) appears in both Phase 2 and Phase 11. Running it in Phase 2 is premature -- the build will fail until File Change #13 (all HookPorts construction sites) is complete, since adding a new field to HookPorts is a breaking struct change.
- **Evidence**: Solution line 96: `PC-003, PC-037 -- HookPorts field + build fix (no deps)`. But the design also lists File Change #13 ("All make_ports/HookPorts sites in ecc-app tests") as a separate modification, meaning PC-037 cannot pass at Phase 2 unless FC-13 is also done in Phase 2.
- **Recommendation**: Either move PC-037 entirely to Phase 11 (its second appearance), or explicitly note that FC-13 is part of Phase 2 and recount the phase dependencies. The current phrasing is ambiguous about when the 30+ HookPorts sites get fixed.

- **Finding**: Phase 6 is listed as "independent" but PC-019 and PC-020 test `record_if_enabled` which is defined in metrics_mgmt.rs and depends on the MetricsStore port. While the port trait already exists, the tests depend on InMemoryMetricsStore which is already available. This is acceptable but the "independent" label is slightly misleading since it shares the same test infrastructure as Phase 3.
- **Evidence**: Solution line 101: `PC-017, PC-018, PC-019, PC-020, PC-021 -- Commit gate app + kill switch (independent)`

### 3. Fragility
- **Finding**: All 35 PC commands use specific Rust test module paths (e.g., `cargo test -p ecc-app -- hook::tests::dispatch_records_hook_success_metric`). If the test module structure changes during implementation (e.g., tests get nested into submodules for organization), every PC command breaks. This is standard for Rust TDD but flagged as a known fragility.
- **Evidence**: PC-004 through PC-035 all use `--` filter paths.
- **Recommendation**: Acceptable -- this is inherent to Rust test filtering. Implementer should treat these as intent, not rigid commands.

- **Finding**: The solution depends on `resolve_session_id()` existing in `metrics_session.rs` with a specific signature accepting `Option<&str>`. If that function's signature changes (e.g., to accept an `Environment` trait reference instead), PC-008 and all session ID resolution paths break.
- **Evidence**: Solution FC-4 says "session_id (via resolve_session_id)" but the function currently takes `Option<&str>` (line 10 of metrics_session.rs), requiring the caller to first read `CLAUDE_SESSION_ID` from the environment and pass it in. The solution's description implies this is handled at the call site but doesn't explicitly describe the two-step pattern.

### 4. Rollback Adequacy
- **Finding**: Step 9 says "Revert crates/ecc-app/src/hook/mod.rs (+ all make_ports sites)" but does not enumerate the 30+ files that contain HookPorts struct construction. If rollback is needed after partial implementation, an implementer would not know which files had already been touched.
- **Evidence**: Solution line 156: `9. Revert crates/ecc-app/src/hook/mod.rs (+ all make_ports sites)`. The actual count is 31 struct literal sites across 28 files in ecc-app alone, plus 1 in ecc-cli.
- **Recommendation**: Add explicit note: "use `git diff --name-only` against the pre-implementation commit to identify all modified files" or list a representative sample.

### 5. Architecture Compliance
- **Finding**: Domain crate changes (targets.rs, trend.rs) are pure computation with zero I/O. Verified: ecc-domain/src/metrics/ currently has no std::fs, std::process, or std::net imports.
- **Evidence**: Existing metrics module (event.rs, aggregate.rs) contains only domain logic. New files follow the same pattern.

- **Finding**: DIP fix correctly places SqliteMetricsStore construction in main.rs (binary entry point), not in transition.rs. This is the textbook hexagonal pattern.
- **Evidence**: FC-8: "Construct SqliteMetricsStore at binary entry point (DIP fix per uncle-bob)". FC-9 accepts `Option<&dyn MetricsStore>`.

- **Finding**: ecc-workflow already has `ecc-infra` as a dependency (Cargo.toml line 15), so no new coupling is introduced.
- **Evidence**: `ecc-infra = { workspace = true }` in crates/ecc-workflow/Cargo.toml.

### 6. Blast Radius
- **Finding**: File Change #13 claims "~15 sites" for HookPorts construction fixup. Actual count from codebase: 31 `HookPorts {` struct literal occurrences across 28 files in ecc-app, plus 1 in ecc-cli/src/commands/hook.rs. The solution undercounts by roughly 2x.
- **Evidence**: `grep -c "HookPorts\s*\{"` yields 32 total occurrences across 28 files. Each `make_ports` helper function returns a HookPorts struct literal, so each must be updated.
- **Recommendation**: Correct FC-13 to say "~30 sites across ~28 files" and acknowledge that this is the highest-risk mechanical change. Consider whether a helper function in ecc-test-support could reduce the blast radius (e.g., a `TestHookPorts::new()` builder that defaults metrics_store to None).

- **Finding**: The solution touches 5 crates (ecc-domain, ecc-app, ecc-cli, ecc-workflow, ecc-infra is untouched but its adapter is used). This is moderate cross-crate reach but justified by the wiring nature of the work.
- **Evidence**: 13 file changes listed, but FC-13 alone represents ~28 additional file touches.

### 7. Missing Pass Conditions
- **Finding**: No `cargo fmt --check` pass condition. The solution has PC-036 (clippy) and PC-037 (build) but omits format checking. The CI pipeline (ci.yml) runs `cargo fmt --check` as a required gate.
- **Evidence**: PC-036 is `cargo clippy -- -D warnings`, PC-037 is `cargo build --workspace`. No PC for `cargo fmt -- --check`.
- **Recommendation**: Add PC-038 (or renumber): `cargo fmt -- --check` with expected exit 0.

- **Finding**: No integration test PC despite wiring MetricsStore port to 4 new call sites (hook dispatch, subagent logging, quality gate, workflow transition). The E2E test plan says "No E2E tests activated" but the solution adds real MetricsStore wiring in ecc-cli and ecc-workflow. At minimum, a smoke test verifying that a hook dispatch with a real SqliteMetricsStore actually persists an event would catch adapter incompatibilities.
- **Evidence**: E2E Activation Rules: "No E2E tests activated for this implementation". But the solution wires SqliteMetricsStore in FC-8 (ecc-workflow) and FC-10 (ecc-cli), both of which are real I/O paths.
- **Recommendation**: Add at least one integration PC in ecc-integration-tests that verifies round-trip: dispatch hook -> event appears in SQLite -> summary returns it. This catches adapter mismatches that unit tests with InMemoryMetricsStore would miss.

- **Finding**: AC-006.1 through AC-006.4 have no automated pass conditions (discussed in Dimension 1). This is both an AC coverage gap and a missing structural PC.

### 8. Doc Plan Completeness
- **Finding**: CHANGELOG.md is present in Doc Update Plan item #3. All decisions are marked "ADR Needed? No". Doc levels appear appropriate (CLAUDE.md for CLI surface, commands/ for catchup).
- **Evidence**: Solution line 115: `CHANGELOG.md | project | modify | feat: wire harness reliability metrics`

- **Finding**: MODULE-SUMMARIES.md is not in the Doc Update Plan. The solution adds 2 new files to ecc-domain (targets.rs, trend.rs) and modifies 7 existing files. Per the doc hierarchy, MODULE-SUMMARIES.md should be updated to reflect the new domain types.
- **Evidence**: Doc hierarchy from CLAUDE.md: "docs/MODULE-SUMMARIES.md (per-crate reference)". Solution's Doc Update Plan has 4 items, none targeting MODULE-SUMMARIES.
- **Recommendation**: Add a 5th doc plan item: `MODULE-SUMMARIES.md | project | modify | Add ReferenceTargets, TrendComparison to ecc-domain section`.

## Suggested PCs

If CONDITIONAL is to be resolved to PASS, add these pass conditions:

1. **PC-038** (fmt): `cargo fmt -- --check` | Expected: exit 0 | Verifies: all
2. **PC-039** (catchup content): `grep -q "ecc metrics summary --session" commands/catchup.md && grep -q "No harness metrics recorded" commands/catchup.md` | Expected: exit 0 | Verifies: AC-006.1, AC-006.2
3. **PC-040** (integration): `cargo test -p ecc-integration-tests -- metrics::hook_dispatch_records_event` | Expected: PASS | Verifies: AC-001.1 end-to-end through SqliteMetricsStore
4. Fix FC-13 count: update "~15 sites" to "~30 sites across ~28 files" in the solution
5. Add MODULE-SUMMARIES.md to Doc Update Plan
6. Clarify Phase 2 vs Phase 11 for PC-037 (remove duplicate or make FC-13 explicitly part of Phase 2)

## Verdict Rationale

Average score is 73/100 (above the 70 threshold for PASS), but Dimension 7 (Missing Pass Conditions) scores 60 and Dimension 1 (AC Coverage) scores 62 -- both below 70, creating two dimensions with notable concerns. No dimension falls below 50, so this avoids FAIL.

The verdict is CONDITIONAL because:
1. Four ACs (006.1-006.4) have zero automated verification -- they rely on manually inspecting a markdown file edit. This is the single largest coverage gap.
2. The missing `cargo fmt` PC means the CI gate is not replicated in the design's pass conditions.
3. The HookPorts site count is underestimated by 2x, which does not break the design but misrepresents the implementation effort.
4. No integration test despite wiring a port adapter to 4 new call sites in 2 binaries.

Adding the 3 suggested PCs (fmt, catchup content grep, integration smoke test), fixing the FC-13 count, and adding MODULE-SUMMARIES to the doc plan would resolve all findings and move the verdict to PASS.
