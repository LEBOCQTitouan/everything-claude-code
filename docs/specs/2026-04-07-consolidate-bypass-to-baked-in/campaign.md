# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Smell triage | Address all smells 1-9 except bus factor (not addressable via code) | user |
| 2 | Target architecture | Full port extraction: add check_token() to BypassStore, wire SqliteBypassStore in prod CLI, HookPorts::test_default() helper, remove ECC_WORKFLOW_BYPASS everywhere | recommended |
| 3 | Step independence | Independent steps — each ships alone, test suite stays green between steps | recommended |
| 4 | Downstream dependencies | HookPorts::test_default() for 28 handler files, manual integration test update, doc sweep last | recommended |
| 5 | Rename vs behavioral change | Pure removal (env var), behavioral additive (port method), behavioral restructure (interceptor), mechanical (test boilerplate) | recommended |
| 6 | Performance budget | Negligible impact — vtable indirection only, no new I/O on hot path | recommended |
| 7 | ADR decisions | Update ADR-0056 status to Completed, no new ADRs needed | recommended |
| 8 | Test safety net | Need more coverage first — add tests for check_token() port method and token-bypass end-to-end path before refactoring | user |
| 9 | Adversary round 1 | CONDITIONAL: added 10 ACs (AC-001.6-10, AC-002.5-7, revised AC-005.1-2, AC-006.4), added dependency US-006 to all US, added Rollback section, added Decisions 8-9 | recommended |
| 10 | Adversary round 2 | CONDITIONAL: added AC-001.11-13, AC-003.3 clarification, AC-004.3 exit codes, Decisions 10-12, expanded Affected Modules, enhanced Rollback Strategy | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
