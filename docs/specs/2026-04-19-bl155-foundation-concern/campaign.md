# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Semver policy for Concern enum extension? | Accept as major-version bump. Add Foundation as normal variant; bump workspace Cargo.toml version accordingly. ecc-domain has no external consumers, impact is internal-only. Simplest path; reflects reality (Concern is genuinely expanding). | recommended |
| 2 | Doc sites to update? | Update all 3: UnknownConcern error text (alphabetical: dev, fix, foundation, or refactor) + assert via test, project-foundation.md line 18 revert to init foundation, catchup.md line 19 concern list. Prevents doc-code drift. | recommended |
| 3 | Phase transitions for Foundation? | Same FSM as dev/fix/refactor (plan->solution->implement->done). Integration test verifies init foundation -> transition each phase -> done exits 0. No TransitionPolicy change. Foundation-specific phase skipping deferred to separate spec per BL-155 out-of-scope. | recommended |
| 4 | Scope boundaries? | concern.rs (enum + Display + FromStr + error msg + tests) + state.rs test array + ecc-workflow tests + 3 doc sites (project-foundation.md, catchup.md, CHANGELOG.md). Out: phase-transition changes, spec-dir slug conventions, backfill state.json. | recommended |
| 5 | Edge cases? | Casing (lowercase-only maintained), empty string still fails, existing dev/fix/refactor state.json deserialize unchanged, UnknownConcern error text includes foundation. | recommended |
| 6 | Performance? | N/A. Enum variants are zero-cost; no runtime cost increase. | recommended |
| 7 | Security? | N/A. Pure domain value object, no input validation surface beyond FromStr which already rejects unknowns. | recommended |
| 8 | Breaking changes? | YES - semver-wise. Addressed by Decision 1 (accept as major bump). No data migration (no existing foundation state.json files). | recommended |
| 9 | Domain glossary additions? | None. Foundation was already defined in BL-143 (/project-foundation command); this just adds the missing domain enum variant. | recommended |
| 10 | ADR needed? | No. Simple value-object extension following existing pattern (Dev/Fix/Refactor). No architectural decision, no alternatives weighed. | recommended |
| 11 | Adversary round 1 verdict? | CONDITIONAL (73/100). 7 fixes: AC-001.5 test count wrong (7 not 6; drop number), add AC-001.6 case-sensitivity, add AC-001.7 explicit UnknownConcern assertion, AC-002.4 tempfile isolation, Decision 1 name version (or downgrade to minor + waiver), add 2 skills/ sites, drop/tighten AC-003.3. | user |
| 12 | Version bump direction? | Reverse Decision 1: minor bump 4.2.0 -> 4.3.0 with cargo-semver-checks waiver via baseline override. Rationale 'no external consumers' is the case FOR minor, not major. Enum-extension shipped as additive minor. | user |
| 13 | Adversary round 2 verdict? | PASS (86/100). All dimensions >=70. No new issues from v2 fixes. Proceed to persist + transition solution. | user |
| 14 | Design adversary round 1? | CONDITIONAL (78/100). 6 fixes: AC-003.3 numbering gap, semver baseline tag v4.2.0 missing in repo, AC-001.5 diff-test needed, rollback plan absent, add PC-semver + PC-fmt, split Step 4 doc commits to 3 atomic ones. Plus line-anchored PC-014 + env!(CARGO_BIN_EXE) pattern. | user |
| 15 | Design adversary round 2 verdict? | PASS (83/100). Fragility 55->72, Rollback 60->74, Missing PCs 75->80. All 6 round-1 fixes applied. 3 non-blocking residuals: PC-014 line-hardcoded, origin/main baseline drift, rollback window. All baked into final design.md as refinements. | user |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
