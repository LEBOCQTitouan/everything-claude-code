# Campaign Manifest

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|

| 1 | Scope? | 7 implementation stories (1 per crate) + 1 verification. ~115 eligible items, ~85 files. ecc-test-support and ecc-integration-tests excluded (zero eligible). | recommended |
| 2 | Test strategy? | cargo test unchanged — doc-comments don't affect tests. Verification via cargo doc --no-deps (compiles docs) and grep assertions for diagram presence. | recommended |
| 3 | Breaking changes? | None — purely additive doc-comments. Zero functional changes. | recommended |
| 4 | ADR? | No — applying existing ascii-doc-diagrams convention, no new decisions. | recommended |
| 5 | Execution order? | Priority: ecc-domain + ecc-workflow first (named targets), then ecc-ports + ecc-app, then ecc-infra + ecc-cli + ecc-flock. All parallel-safe. | recommended |
## Agent Outputs

| Agent | Phase | Summary |
|-------|-------|---------|

## Commit Trail

| SHA | Message |
|-----|---------|
