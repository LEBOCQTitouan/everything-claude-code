# Implementation Complete: BL-112 Migrate Release Pipeline to cargo-dist

## Spec Reference
Concern: refactor, Feature: BL-112 cargo-dist release pipeline migration

## Changes Made
| # | File | Action | Solution Ref | Tests | Status |
|---|------|--------|--------------|-------|--------|
| 1 | dist.toml | create | PC-001 to PC-008 | structural+build | done |
| 2 | .github/workflows/release.yml | replace | PC-012 to PC-020 | YAML+structural | done |

## TDD Log
| PC ID | RED | GREEN | REFACTOR | Notes |
|-------|-----|-------|----------|-------|
| PC-001 to PC-007 | N/A (config) | All pass | N/A | dist.toml structure |
| PC-008 | N/A | SKIP (cargo-dist not installed) | N/A | Local build optional |
| PC-009 to PC-011 | N/A | All pass | N/A | Build/test/clippy gate |
| PC-012 to PC-020 | N/A | All pass | N/A | Workflow structure + cosign |
| PC-021 to PC-023 | N/A | All pass (reuse PC-009-011) | N/A | Final gates |
| PC-024 | N/A | Pass | N/A | No old file backup |

## Pass Condition Results
| PC ID | Command | Expected | Actual | Status |
|-------|---------|----------|--------|--------|
| PC-001 | test -f dist.toml | Exit 0 | Exit 0 | PASS |
| PC-002 | grep all 5 targets | Exit 0 | Exit 0 | PASS |
| PC-003 | grep ecc-cli + ecc-workflow | Exit 0 | Exit 0 | PASS |
| PC-004 | grep checksum sha256 | Exit 0 | Exit 0 | PASS |
| PC-005 | grep content dirs | Exit 0 | Exit 0 | PASS |
| PC-006 | grep shell shims | Exit 0 | Exit 0 | PASS |
| PC-007 | grep scripts dirs | Exit 0 | Exit 0 | PASS |
| PC-008 | cargo dist build | SKIP | SKIP | SKIP |
| PC-009 | cargo build --workspace | Exit 0 | Exit 0 | PASS |
| PC-010 | cargo test --workspace | Exit 0 | Exit 0 | PASS |
| PC-011 | cargo clippy | Exit 0 | Exit 0 | PASS |
| PC-012 | YAML parse release.yml | Exit 0 | Exit 0 | PASS |
| PC-013 | grep v* tags | Exit 0 | Exit 0 | PASS |
| PC-014 | grep permissions | Exit 0 | Exit 0 | PASS |
| PC-015 | grep all 5 targets | Exit 0 | Exit 0 | PASS |
| PC-016 | cd.yml unchanged | Exit 0 | Exit 0 | PASS |
| PC-017 | grep cosign | Exit 0 | Exit 0 | PASS |
| PC-018 | grep continue-on-error | Exit 0 | Exit 0 | PASS |
| PC-019 | grep .sig | Exit 0 | Exit 0 | PASS |
| PC-020 | grep plan.jobs | Exit 0 | Exit 0 | PASS |
| PC-021 | cargo build | Exit 0 | Exit 0 | PASS |
| PC-022 | cargo test | Exit 0 | Exit 0 | PASS |
| PC-023 | cargo clippy | Exit 0 | Exit 0 | PASS |
| PC-024 | no old file backup | Exit 0 | Exit 0 | PASS |

All pass conditions: 23/24 PASS, 1 SKIP (PC-008 cargo-dist not installed)

## E2E Tests
No automated E2E tests — manual RC pre-release verification required post-merge (AC-004.2-004.4).

## Docs Updated
| # | Doc File | Level | What Changed |
|---|----------|-------|--------------|
| 1 | docs/adr/0039-cargo-dist-adoption.md | MEDIUM | New ADR for cargo-dist adoption |
| 2 | docs/adr/0040-cosign-custom-job.md | MEDIUM | New ADR for cosign as custom job |
| 3 | docs/adr/README.md | LOW | Added 0039, 0040 entries |
| 4 | CLAUDE.md | HIGH | Added cargo dist build command |
| 5 | rules/ecc/github-actions.md | MEDIUM | Updated release.yml section |
| 6 | CHANGELOG.md | project | Added BL-112 migration entry |

## ADRs Created
| # | File | Decision |
|---|------|----------|
| 1 | docs/adr/0039-cargo-dist-adoption.md | cargo-dist adoption for binary distribution |
| 2 | docs/adr/0040-cosign-custom-job.md | Cosign signing as custom post-build job |

## Supplemental Docs
No supplemental docs generated — no Rust crates modified, config/CI only.

## Subagent Execution
Inline execution — subagent dispatch not used (config/CI refactoring).

## Code Review
PASS — Config/CI refactoring only. dist.toml validated for all required entries. release.yml restructured to cargo-dist pipeline with custom cosign job. All targets preserved. cd.yml unchanged.

## Suggested Commit
refactor(release): migrate release pipeline to cargo-dist
