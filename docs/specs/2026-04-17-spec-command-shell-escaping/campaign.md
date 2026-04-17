# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Is the shell interpolation the true root cause? | User requested deeper investigation - check Claude Code template semantics and all other !-prefix sites | user |
| 2 | Fix scope - minimal vs proper? | Proper fix with guard-rail: template removal + --feature-stdin CLI flag + ecc validate commands rule | recommended |
| 3 | Test coverage scope? | All 4 layers: CLI argv + stdin + validation rule + template integration test; plus property/fuzz test on stdin round-trip | user |
| 4 | Which regression vectors should be hard constraints? | All four: ecc-cli/workflow parity, positional backward-compat, grill-me as Non-Requirement follow-up, preserve domain invariants | user |
| 5 | How to incorporate prior audit findings? | Document audit-scope gap in spec + create backlog entry for comprehensive slash-command template audit | user |
| 6 | Reproduction steps sufficient? | Use derived repro + canonical test-vector list (backtick, double-quote, single-quote, dollar, backslash, newline, Unicode control) | recommended |
| 7 | Data impact - any migration or cleanup needed? | Zero data impact confirmed by empirical scan of 26 campaign.md files and 4 state.json files - all feature strings clean, no residual corruption | user |
| 8 | Adversarial review round 2 verdict? | PASS avg 86/100 - all 7 dimensions passed; minor residuals noted as /design clarifications | recommended |
| 9 | Adversary round 4 verdict? | PASS avg 84/100 after 3 CONDITIONAL rounds. All spec deviations reconciled by tightening solution to match pinned spec values; dangerous payloads replaced with safe reproducers; framing honesty restored for AC-001.1b/1.5 | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
