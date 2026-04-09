# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Sources consulted | rust-patterns, error-handling, testing — all relevant to regex/error/test fixes | recommended |
| 2 | Root cause vs symptom | Systemic — rapid growth outpaced hygiene. Treat as one velocity-vs-quality remediation. | user |
| 3 | Minimal vs proper fix | Full structural — decompose all 7 oversized files, full clock port, full regex migration, full doc coverage push. | user |
| 4 | Missing tests | Comprehensive — test every change: prune boundary, regex compilation, warn emission, file split re-exports, doc coverage metric. | user |
| 5 | Regression risk | Re-export breakage from file splits is the primary concern. Must preserve public API via re-exports after decomposition. | recommended |
| 6 | Related audit findings | Bundle overlapping MEDIUMs — CORR-002, CORR-003, ERR-002/003/004, CONV-003, DOC-002/003 included since we're already touching those files. | recommended |
| 7 | Reproducibility | Sufficient — all findings reproducible via cargo test, clippy, wc, grep commands. No manual steps needed. | recommended |
| 8 | Data impact | Verify with a test migration — add test proving old and new prune() produce identical results for same inputs. No schema migration. | user |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
