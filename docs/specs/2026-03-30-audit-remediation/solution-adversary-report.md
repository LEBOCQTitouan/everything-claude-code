# Solution Adversary Report

## Summary
Verdict: CONDITIONAL (avg: 62/100)
Rounds: 1 of 3

## Dimension Results
| # | Dimension | Score | Verdict | Critical Findings |
|---|-----------|-------|---------|-------------------|
| 1 | AC Coverage | 40 | FAIL | 4 ACs completely missing from design (no file changes, no PCs). 2 ACs have file changes but no verifying PCs. |
| 2 | Execution Order | 85 | PASS | Dependency graph is sound. Minor concern with Phase 4/7 overlap on PC-027/PC-028. |
| 3 | Fragility | 70 | PASS | Some grep-based PCs are brittle but functional. |
| 4 | Rollback Adequacy | 30 | FAIL | No Rollback Plan section exists in the design at all. |
| 5 | Architecture Compliance | 85 | PASS | Changes follow hexagonal rules. No SOLID Assessment section but no violations found. |
| 6 | Blast Radius | 70 | PASS | 25 file changes across 5 crates is significant but justified by scope. |
| 7 | Missing Pass Conditions | 75 | PASS | Build (PC-029) and lint (PC-017, PC-024, PC-028) PCs exist. No dedicated clippy PC separate from build. |
| 8 | Doc Plan Completeness | 40 | FAIL | CHANGELOG.md not in doc plan. No SOLID/Robert/Security assessment sections. |

## Uncovered ACs
| AC | Description (from spec) | Suggested PC |
|----|------------------------|--------------|
| AC-003.2 | `.unwrap()` in `flock_lock.rs:149` and `toggle.rs:126` propagated with `?` | PC-034: `grep -c '\.unwrap()' crates/ecc-flock/src/flock_lock.rs crates/ecc-app/src/commands/toggle.rs` expects 0 in production paths |
| AC-003.6 (partial) | `DryRun` enum replacing bool params | PC-035: `cargo test -p ecc-domain dry_run_enum` or `grep -c 'DryRun' crates/ecc-domain/src/...` |
| AC-004.3 | ARCHITECTURE.md regenerated with ecc-flock and correct test count | PC-036: `grep -c 'ecc-flock' docs/ARCHITECTURE.md` expects >= 1 |
| AC-004.4 | Duplicate ADR numbers fixed (three 0030s, two 0031s) | PC-037: `ls docs/adr/ \| grep -c '003[0-1]'` — verify unique numbering, or explicitly acknowledge AC is stale and mark N/A |
| AC-004.6 | MODULE-SUMMARIES subcommand count fixed (17 to 20) | PC-038: `grep -c '20 subcommands\|20)' docs/MODULE-SUMMARIES.md` expects >= 1 |
| AC-005.4 | `ecc-workflow --help` mentions RUST_LOG | PC-039: `cargo run -p ecc-workflow -- --help 2>&1 \| grep -c RUST_LOG` expects >= 1 |

## Detailed Findings

### AC Coverage
- **Finding**: 4 ACs from the spec are completely absent from the design -- no file changes, no pass conditions, no mention whatsoever.
- **Evidence**: AC-003.2 (unwrap in flock_lock.rs/toggle.rs), AC-003.6 partial (DryRun enum -- spec says "ColorMode AND DryRun enums replace bools", design only implements ColorMode), AC-004.4 (ADR renumbering), AC-005.4 (RUST_LOG in --help). The design's Risks section mentions AC-004.4 may be stale but provides no PC to verify that claim.
- **Recommendation**: Either add file changes + PCs for each missing AC, or explicitly mark them as N/A with justification in the design (not just a risk note).

- **Finding**: 2 ACs have file changes but no verifying PCs.
- **Evidence**: AC-004.3 (ARCHITECTURE.md regeneration -- File Change #24 exists, but no PC checks that ecc-flock appears or test count is correct). AC-004.6 (MODULE-SUMMARIES -- File Change #25 exists, but no PC verifies the count was actually fixed).
- **Recommendation**: Add PC-036 and PC-038 as specified in the Uncovered ACs table.

- **Finding**: AC-003.7 specifies "13 glob re-exports" but the design only accounts for 9 (4 in handlers/mod.rs + 5 in tier1_simple/mod.rs). The remaining 4 glob re-exports are unaddressed.
- **Evidence**: Spec line 78: "Given 13 glob re-exports in module files". Design File Changes #18 and #19 only cover 2 files with 9 re-exports total.
- **Recommendation**: Identify and add file changes for the remaining 4 glob re-exports, or update the spec if the count was wrong.

### Execution Order
- **Finding**: PC-027 and PC-028 appear in both Phase 4 and Phase 7. Phase 7 description says "Already partially handled in Phase 4." This creates ambiguity about which phase is responsible for satisfying these PCs.
- **Evidence**: Phase 4 Pass list includes PC-027, PC-028. Phase 7 Pass list includes PC-027, PC-028. If Phase 4 already satisfies them, Phase 7 has no unique PCs beyond PC-030.
- **Recommendation**: Assign PC-027 and PC-028 to exactly one phase. If Phase 7 adds "remaining" log::info! calls, make that a separate PC.

### Fragility
- **Finding**: PC-008 uses `wc -l` with piped `tail -1` to verify file sizes. This is brittle -- the output format of `wc -l` varies across platforms (macOS vs Linux padding).
- **Evidence**: PC-008 Command: `wc -l crates/ecc-domain/src/config/merge/*.rs | tail -1` with Expected "total under 868 lines, each file < 400". This is not a single verifiable assertion -- it requires human interpretation of the output.
- **Recommendation**: Replace with a script that parses `wc -l` output and exits non-zero if any file exceeds 400 lines.

- **Finding**: PC-017, PC-024, PC-028 use `grep -c` which is fragile to comment patterns, string literals, and false positives.
- **Evidence**: PC-017: `grep -c 'Regex::new.*\.unwrap()'` would miss `.unwrap( )` with whitespace or `unwrap_or`. PC-028: the grep pattern uses `\|` alternation which behaves differently in grep vs egrep.
- **Recommendation**: These are acceptable for a remediation pass but should be noted as CONDITIONAL. A `cargo clippy` custom lint would be more robust long-term.

### Rollback Adequacy
- **Finding**: The design has zero rollback planning. No Rollback Plan section exists anywhere in the document.
- **Evidence**: Searched for "Rollback", "rollback", "revert" -- no matches. The Risks section mentions "Single atomic commit -- revert if anything breaks" for Phase 3 only, but this is not a rollback plan.
- **Recommendation**: Add a Rollback Plan section covering: (1) Phase 3 merge.rs decomposition revert procedure, (2) Phase 4 install error propagation revert (callers need signature rollback), (3) Phase 6d ColorMode enum revert (touches many callers). At minimum, specify that each phase is independently revertible via `git revert` on its commit.

### Architecture Compliance
- **Finding**: No SOLID Assessment, Robert CLEAN review, or Security CLEAR sections exist in the design despite the task description claiming "Reviews: SOLID PASS, Robert CLEAN, Security CLEAR."
- **Evidence**: Grepped for "SOLID", "Robert", "Security", "CLEAR" -- zero matches in design.md.
- **Recommendation**: These sections should be present in the design document if they were performed. If not performed, the task description is misleading.

- **Finding**: Domain crate changes are pure (merge.rs, ansi.rs). No I/O imports introduced. `ecc-workflow` changes stay within that crate's boundary. Architecture rules are not violated.
- **Recommendation**: None needed -- architecture compliance is sound despite missing assessment sections.

### Blast Radius
- **Finding**: 25 file changes across 5 crates (ecc-domain, ecc-app, ecc-cli, ecc-workflow, docs). Phase 6d (ColorMode) is flagged by the design itself as high blast radius.
- **Evidence**: Design Risks table: "ColorMode enum changes signature of 6 ansi functions, touching many callers" with mitigation "consider From<bool> impl or defer."
- **Recommendation**: The design correctly identifies this risk. The `From<bool>` mitigation should be mandatory, not optional, to avoid a single-commit blast touching dozens of call sites.

- **Finding**: Phase 4 touches 4 files across 3 crates (domain, app, CLI) in a single logical change. This is the riskiest cross-crate modification.
- **Evidence**: File Changes #7, #8, #9, #10 all participate in Phase 4.
- **Recommendation**: PC-015 (integration test) partially mitigates this. Acceptable.

### Missing Pass Conditions
- **Finding**: PC-029 serves as both the clippy lint PC and the build PC. This is acceptable -- `cargo clippy` implies compilation.
- **Evidence**: PC-029: `cargo clippy --workspace -- -D warnings`. PC-030: `cargo test --workspace`.
- **Recommendation**: No action needed. Both structural PCs exist.

- **Finding**: No CLI behavior PC exists for the changed install error output (AC-002.4 says "user sees which steps failed"). PC-013 tests accumulation but not the CLI rendering of errors.
- **Evidence**: AC-002.4: "Given install completes with errors, when summary is displayed, then user sees which steps failed." PC-013 tests the accumulation logic but the CLI display path (install.rs) only gets a "verify no changes needed" note (File Change #10).
- **Recommendation**: Add a PC that verifies the CLI renders accumulated errors to the user -- either a unit test on the CLI rendering function or an integration test that captures stdout.

### Doc Plan Completeness
- **Finding**: CHANGELOG.md exists in the project root but is not mentioned anywhere in the design's documentation plan.
- **Evidence**: `CHANGELOG.md` found via glob. Design Phase 8 only covers CLAUDE.md, ARCHITECTURE.md, MODULE-SUMMARIES.md. No CHANGELOG entry planned.
- **Recommendation**: Add CHANGELOG.md update to Phase 8 documenting the install error propagation behavioral change (AC-002.1..AC-002.7) and any public API signature changes.

- **Finding**: The spec's Doc Impact Assessment mentions ADR renumbering (docs/adr/) but the design drops this entirely without explanation beyond a risk note.
- **Evidence**: Spec line 170: "ADR renumber | docs/adr/ | 0030, 0031 duplicates | Assign 0032, 0033". Design Risks: "ADR renumbering (AC-004.4) may be stale... Skip this AC if confirmed stale." No file change, no PC, no explicit decision to skip.
- **Recommendation**: Either add a file change + PC for ADR renumbering, or add an explicit "Deferred/N/A" entry in the design with verification that no duplicates exist.

## Suggested PCs

To reach PASS, add the following:

1. **PC-034** (AC-003.2): `grep -rn '\.unwrap()' crates/ecc-flock/src/flock_lock.rs crates/ecc-app/src/commands/toggle.rs | grep -v '#\[cfg(test)\]' | grep -v '// test'` -- verify unwraps removed from production paths. Add file changes for both files.

2. **PC-035** (AC-003.6 partial): `grep -c 'DryRun' crates/ecc-domain/src/` or `cargo test -p ecc-domain dry_run` -- verify DryRun enum exists and replaces bool params. Add file changes for DryRun implementation, or explicitly scope AC-003.6 to ColorMode only with spec amendment.

3. **PC-036** (AC-004.3): `grep -c 'ecc-flock' docs/ARCHITECTURE.md` expects >= 1.

4. **PC-037** (AC-004.4): `ls docs/adr/ | sort | uniq -d` expects empty (no duplicate numbers). If no duplicates exist on disk, mark AC-004.4 as N/A with evidence.

5. **PC-038** (AC-004.6): `grep '20' docs/MODULE-SUMMARIES.md` -- verify subcommand count corrected.

6. **PC-039** (AC-005.4): `cargo run -p ecc-workflow -- --help 2>&1 | grep -ic 'RUST_LOG'` expects >= 1. Add file change for ecc-workflow help text.

7. **PC-040** (AC-002.4 CLI rendering): Add a test or verification that `ecc install` CLI output includes error details when `InstallSummary.success == false`.

8. **CHANGELOG.md**: Add to Phase 8 file changes list.

## Verdict Rationale

The design is CONDITIONAL, not PASS, for three reasons:

1. **AC Coverage is FAIL (score 40)**: Four acceptance criteria from the spec are completely absent from the design with no file changes and no pass conditions. AC-003.2 (unwrap removal in flock_lock/toggle), AC-003.6 partial (DryRun enum), AC-004.4 (ADR renumbering), and AC-005.4 (RUST_LOG documentation) will ship untested and unimplemented. Two additional ACs have file changes but no verifying PCs.

2. **Rollback Adequacy is FAIL (score 30)**: The design contains zero rollback planning. For a 25-file, 5-crate remediation touching public API signatures, this is unacceptable. The single mention of "revert if anything breaks" in the Risks section for Phase 3 is not a rollback plan.

3. **Doc Plan is FAIL (score 40)**: CHANGELOG.md is missing from the documentation plan entirely. ADR renumbering is dropped without formal disposition. No SOLID/Robert/Security assessment sections exist despite being referenced.

The remaining dimensions (Execution Order, Fragility, Architecture Compliance, Blast Radius, Missing Pass Conditions) are acceptable. The issues are addressable without redesign -- adding the suggested PCs, a Rollback Plan section, and CHANGELOG entry would bring this to PASS.
