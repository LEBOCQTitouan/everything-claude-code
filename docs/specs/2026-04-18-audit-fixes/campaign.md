# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Scope? | All actionable HIGH+MEDIUM except bus factor (organizational). 1 HIGH (CONV-001 oversized files) + 15 MEDIUM across 8 domains. Exclude: EVOL-001 bus factor, all INFO/positive, all LOW. | recommended |
| 2 | Fix approach? | Proper structural fixes, not patches. Decompose oversized files. Clock port injection. Swallowed error replacement. deny_unknown_fields addition. | recommended |
| 3 | Phasing? | Group by domain: (1) Convention fixes (oversized files), (2) Error handling (swallowed errors + clock port), (3) Security (deny_unknown_fields), (4) Testing (xtask env test), (5) Documentation (missing_docs), (6) Observability (health check). Each independently shippable. | recommended |
| 4 | ADR needed? | No — these are code quality improvements, not architectural decisions. | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
