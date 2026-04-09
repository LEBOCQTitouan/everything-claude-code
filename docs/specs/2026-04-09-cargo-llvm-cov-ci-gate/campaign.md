# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | What is explicitly OUT of scope? | Out of scope: Codecov integration, PR comments, branch coverage, trend tracking | recommended |
| 2 | Skip coverage when no Rust files changed? | Yes, path-filter on **.rs, Cargo.toml, Cargo.lock | recommended |
| 3 | Per-crate or workspace aggregate threshold? | Workspace aggregate 80% only | recommended |
| 4 | CI timeout for coverage job? | 20 minutes, separate cache key cargo-llvm-cov-* | recommended |
| 5 | Security implications? | None - read-only, no secrets, no external uploads | recommended |
| 6 | Blocking required status check? | Yes, coverage job blocks PR merge if below 80% | recommended |
| 7 | New domain terms for glossary? | Add coverage gate definition to CLAUDE.md glossary | user |
| 8 | ADR needed? | No - straightforward CI addition following established patterns | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
