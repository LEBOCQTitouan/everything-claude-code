# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Prior spec 2026-04-17 exists but unimplemented — how should this session relate? | Hybrid: amend prior 2026-04-17/spec.md with ONLY a ## Revision block adding the clap-derive deny-list. Keep prior 7-story per-crate structure and 14 PCs. Accept remaining architect findings (cross-context rule, drift anchors, missed targets) as implementation-time guidance, not formal ACs. | user |
| 2 | Clap-derive deny-list scope? | Deny inline diagrams on ANY item inside files containing #[derive(Parser\|Subcommand\|Args\|ValueEnum)]. Diagrams permitted only at module-level //! comments and impl blocks. Applies to ~25 files across ecc-cli + ecc-workflow. | recommended |
| 3 | Additional priority targets handling? | Add to Revision block as explicit mandatory targets: phase.rs, merge/mod.rs, phase_gate.rs, config/audit.rs (18-enum cluster), backlog/lock_file.rs, workflow/state.rs. Stays within per-crate stories; just names them to prevent triage undersight. | recommended |
| 4 | Cross-context diagram rule? | Each diagram in ecc-domain scoped to its bounded context. Cross-context type references permitted only along 3 declared edges (claw->session, config->detection, hook_runtime->config). Violations reduced to intra-context portion or flagged as arch finding. | recommended |
| 5 | 20-line diagram cap enforcement? | Allow exceeding the 20-line cap for complex modules (e.g., config/audit.rs 18-enum cluster). Pragmatic override of skill's 20-line rule on a per-module basis; no arch-finding trigger. | user |
| 6 | Verification PC for clap-derive deny-list? | Add grep-based PC to design.md Revision that detects inline diagrams inside files containing clap derives. CI-friendly, low maintenance. Specific command wording left to /design phase. | recommended |
| 7 | Test strategy? | cargo test workspace unchanged (doc-comments don't affect tests). Per-crate 'cargo doc --no-deps' validates compilation. New grep PC detects clap-derive deny-list violations. No new insta snapshot tests. | recommended |
| 8 | Performance constraints? | None. Doc-comments have zero runtime cost. cargo doc build time may increase marginally. | recommended |
| 9 | Security implications? | None. /// doc-comments have no injection surface. No secrets, user input, or external APIs touched. | recommended |
| 10 | Breaking changes? | None when deny-list honored. Clap --help output would change if deny-list violated; grep PC prevents. | recommended |
| 11 | Domain concepts for glossary? | None new. Reuses existing terms: bounded context, port, adapter, RAII, state machine, drift anchor. | recommended |
| 12 | ADR decisions? | None. Applying existing ascii-doc-diagrams skill convention. Drift-anchor rule and clap deny-list are Revision policies, not foundational architecture. | recommended |
| 13 | Adversary round 1 verdict? | CONDITIONAL (60/100). Testability=40 (below floor) but all findings addressable. Round 1 fixes: remove R-4 (no automated gate), add baseline-count AC, strengthen PC regexes, add --help smoke test, add cap-override AC, add text-fence-hint + Unicode-ban ACs. Re-dispatch adversary. | user |
| 14 | Adversary round 2 verdict? | CONDITIONAL (72.9/100). Jumped from 60 (v1). Three blockers: PC-019 regex unsatisfiable, PC-015 classifier under-specified, 54-vs-115 coverage target unclear. 7 minor fixes needed. Round 3 will apply all fixes. | user |
| 15 | Adversary round 3 verdict? | PASS (82.1/100). All 7 dimensions above 70; no showstoppers. v3 fixed PC-019 unsatisfiable regex, PC-015 classifier under-specification, 54-vs-115 coverage, CI mandate, minimum shippable subset. | user |
| 16 | Design adversary round 1? | FAIL (58/100). 11 blockers: ADR path typo (docs/adrs vs docs/adr), hardcoded ADR-0067, missing CHANGELOG PC, PC-031 manual, no Step-A gate before Wave 1, awk+rg tool-version fragility, PC-033 floor vs exact, PC-020 off-by-one, CI chicken-and-egg, Doc Plan incomplete, scripts/ vs xtask/ unjustified. Fixes applied as Supplement v2. | user |
| 17 | Design adversary round 2? | CONDITIONAL (72.1/100). 4 mechanical fixes + 1 operational note: PC-017 regex (bare +), PC-037 CHANGELOG false-positive, PC-031-v2 origin/main fragility, line 168 stale 0067, AC-R6.3 vs worktree-merge-hook. All applied. | user |
| 18 | Design adversary round 3? | PASS (78.4/100). Fragility 52->74 (+22), Missing PCs 65->75 (+10). All 5 mechanical fixes verified landing. Proceed to /implement. | user |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
