# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | CLI shape: subcommands vs flag stacking? | Subcommand structure: counts/markers/all, with --counts aliased for one release with deprecation warning | recommended |
| 2 | Default severity polarity: WARN vs ERROR by default? | WARN by default, --strict upgrades to error. Matches raw request; safe for day-1 adoption. | recommended |
| 3 | Scope: CLAUDE.md only or CLAUDE.md + AGENTS.md? | CLAUDE.md + AGENTS.md from v1. Forward-compat with ADR 0062. Zero AGENTS.md files today so identical behavior in practice. | user |
| 4 | Archived status: resolved or unresolved? | Presence-only: archived = resolved. File on disk satisfies. Simpler logic, no frontmatter parsing. | recommended |
| 5 | Markdown parser: hand-rolled or pulldown-cmark? | Hand-rolled fence skip mirroring extract_claims. Zero new deps. Nested fence risk accepted. | recommended |
| 6 | BL-150 stale marker fix: included in spec or separate PR? | Include in spec. Ship lint + fix known drift atomically. CI green on merge. | recommended |
| 7 | CI wiring: --strict from day 1, WARN then promote, or CLI-only? | Wire with --strict from day 1 in ci.yml validate job. BL-150 removed first so first CI run is green. | recommended |
| 8 | ADR: file one for docs↔backlog ACL pattern? | No ADR. Capture in commit messages + CHANGELOG. Code + tests visible enough. | recommended |
| 9 | Scope boundary: out-of-scope items? | All four excluded from v1: (a) orphaned-entry inverse lint, (b) arbitrary markdown paths, (c) --fix auto-removal, (d) doc-validator integration. Candidates for future backlog items. | recommended |
| 10 | Adversary Round 1 verdict? | CONDITIONAL (74/100 avg). 11 remediations applied in Round 2: widened regex to \d{1,6}, added baseline/missing-dir/binary-file/deprecation ACs, rollback story (ci atomic commit + ECC_CLAUDE_MD_MARKERS_DISABLED kill switch), refactor regression guard, governance-loophole counterweight backlog entry commitment, audit-report permanence decision, --counts removal semver anchor. | recommended |
| 11 | Adversary Round 2 verdict? | PASS (87/100 avg, all dimensions >= 80). Residual nits are implementation-level. Spec ready to persist and transition to solution phase. | recommended |
| 12 | Solution adversary Round 1 + 2 verdict? | R1: CONDITIONAL 82/100, 1 dim at 62 (missing PCs). Remediations: PC-038 domain I/O purity grep, PC-039 pre-fix regression anchor grep, PC-040 ANSI sanitizer test, PC-041 clap smoke, tightened PC-024 message composition, tightened PC-015 UTF-8 vs I/O, removed step 10 empty commit, clarified CHANGELOG Unreleased target. R2: PASS 84/100, 39/39 AC coverage, all dimensions >=70. Transition to implement. | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
