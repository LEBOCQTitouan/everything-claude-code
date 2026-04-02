# Solution: BL-112 Migrate Release Pipeline to cargo-dist

## Spec Reference
Concern: refactor, Feature: BL-112 cargo-dist release pipeline migration

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `dist.toml` | Create | cargo-dist config: 5 targets, both binaries, content dirs, checksums, installers | US-001, AC-001.1-001.6 |
| 2 | `Cargo.toml` | Review (no change expected) | Verify workspace compat with cargo-dist; dist.toml is self-contained in cargo-dist 0.22+ | US-001 |
| 3 | `.github/workflows/release.yml` | Replace | cargo-dist-generated workflow + custom cosign signing job | US-002, AC-002.1-002.4; US-003, AC-003.1-003.4 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | dist.toml exists | AC-001.1 | `test -f dist.toml` | Exit 0 |
| PC-002 | unit | All 5 targets listed | AC-001.2 | `for t in aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-msvc; do grep -q "$t" dist.toml \|\| { echo "FAIL: missing $t"; exit 1; }; done` | Exit 0 |
| PC-003 | unit | ecc-cli + ecc-workflow referenced | AC-001.3 | `grep -q 'ecc-cli' dist.toml && grep -q 'ecc-workflow' dist.toml` | Exit 0 |
| PC-004 | unit | sha256 checksums configured | AC-001.4 | `grep -q 'checksum' dist.toml && grep -q 'sha256' dist.toml` | Exit 0 |
| PC-005 | unit | Content dirs in includes | AC-001.5 | `for d in agents commands skills rules hooks contexts mcp-configs examples schemas install.sh; do grep -q "$d" dist.toml \|\| { echo "FAIL: missing $d"; exit 1; }; done` | Exit 0 |
| PC-006 | unit | Shell shims included | AC-001.5 | `grep -q 'ecc-hook' dist.toml && grep -q 'ecc-shell-hook' dist.toml` | Exit 0 |
| PC-007 | unit | Scripts dirs included | AC-001.5 | `grep -q 'scripts/hooks' dist.toml && grep -q 'scripts/codemaps' dist.toml` | Exit 0 |
| PC-008 | integration | Local cargo dist build | AC-001.1, AC-001.6 | `if command -v cargo-dist >/dev/null 2>&1; then cargo dist build --artifacts=local && echo PASS; else echo "SKIP: cargo-dist not installed"; fi` | Exit 0 or SKIP |
| PC-009 | build | Cargo workspace builds | All | `cargo build --workspace` | Exit 0 |
| PC-010 | integration | All tests pass | All | `cargo test --workspace` | Exit 0 |
| PC-011 | lint | Clippy passes | All | `cargo clippy --workspace -- -D warnings` | Exit 0 |
| PC-012 | lint | release.yml valid YAML | AC-002.1 | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"` | Exit 0 |
| PC-013 | unit | Triggers on v* tags | AC-002.2 | `grep -A2 'push:' .github/workflows/release.yml \| grep -q 'v'` | Exit 0 |
| PC-014 | unit | Least-privilege permissions | AC-002.3 | `grep -q 'permissions:' .github/workflows/release.yml` | Exit 0 |
| PC-015 | unit | All 5 targets in workflow | AC-002.2 | `for t in aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-msvc; do grep -q "$t" .github/workflows/release.yml \|\| { echo "FAIL: missing $t"; exit 1; }; done` | Exit 0 |
| PC-016 | unit | cd.yml unchanged | AC-002.5 | `git diff HEAD -- .github/workflows/cd.yml \| wc -l \| xargs test 0 -eq` | Exit 0 |
| PC-017 | unit | Cosign job present | AC-003.1 | `grep -q 'cosign' .github/workflows/release.yml` | Exit 0 |
| PC-018 | unit | Cosign continue-on-error | AC-003.3 | `grep -q 'continue-on-error' .github/workflows/release.yml` | Exit 0 |
| PC-019 | unit | .sig file upload | AC-003.2 | `grep -q '\.sig' .github/workflows/release.yml` | Exit 0 |
| PC-020 | unit | plan.jobs in dist.toml for cosign | AC-003.4 | `grep -q 'plan\.jobs\|plan-jobs\|\[plan\]' dist.toml` | Exit 0 |
| PC-021 | build | Cargo build post-swap | All | `cargo build --workspace` | Exit 0 |
| PC-022 | integration | Tests pass post-swap | All | `cargo test --workspace` | Exit 0 |
| PC-023 | lint | Clippy post-swap | All | `cargo clippy --workspace -- -D warnings` | Exit 0 |
| PC-024 | unit | Old release.yml backup does not exist | AC-002.4 | `! test -f .github/workflows/release-old.yml && ! test -f .github/workflows/release.yml.bak` | Exit 0 |

### Coverage Check

| AC | Covering PCs |
|---|---|
| AC-001.1 | PC-001, PC-008 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005, PC-006, PC-007 |
| AC-001.6 | PC-008 |
| AC-002.1 | PC-012 |
| AC-002.2 | PC-013, PC-015 |
| AC-002.3 | PC-014 |
| AC-002.4 | PC-024 |
| AC-002.5 | PC-016 |
| AC-003.1 | PC-017 |
| AC-003.2 | PC-019 |
| AC-003.3 | PC-018 |
| AC-003.4 | PC-020 |
| AC-004.1 | PC-008 |
| AC-004.2 | Manual: RC pre-release verification |
| AC-004.3 | Manual: RC artifact download + execution |
| AC-004.4 | Manual: ecc update against RC |

19/19 ACs covered. AC-004.2/3/4 are inherently manual (require real CI run + artifact download) — documented as post-merge RC verification.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Release pipeline | CI workflow | N/A | Push v*-rc1 tag, verify 5 archives | ignored | release.yml modified |
| 2 | ecc update | ecc-app | N/A | ecc update --version rc1 | ignored | Tarball format changes |

### E2E Activation Rules
Both tests are manual post-merge verifications via RC pre-release. No automated E2E to un-ignore.

## Test Strategy

TDD order:
1. **PC-001 to PC-007** — dist.toml structural validation (grep-based, no tools needed)
2. **PC-008** — Optional local cargo dist build
3. **PC-009 to PC-011** — Build/test/clippy gate for Phase 1
4. **PC-012 to PC-015** — Workflow YAML structure
5. **PC-016** — cd.yml unchanged guard
6. **PC-017 to PC-020** — Cosign job structure
7. **PC-021 to PC-023** — Final build/test/clippy gate

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0039-cargo-dist-adoption.md` | MEDIUM | Create | ADR for cargo-dist replacing hand-rolled release.yml | Decision #1 |
| 2 | `docs/adr/0040-cosign-custom-job.md` | MEDIUM | Create | ADR for cosign as custom post-build job | Decision #2 |
| 3 | `docs/adr/README.md` | LOW | Modify | Add 0039, 0040 entries | Decision #1, #2 |
| 4 | `CLAUDE.md` | HIGH | Modify | Add cargo dist build, update release docs | US-001 |
| 5 | `rules/ecc/github-actions.md` | MEDIUM | Modify | Update release.yml section for cargo-dist | US-002 |
| 6 | `CHANGELOG.md` | LOW | Modify | Add migration entry | All |

## SOLID Assessment
**PASS** — Config/CI only. No domain, port, or adapter changes. Clean architecture fully respected.

## Robert's Oath Check
**CLEAN** — 1 warning: verify content bundling is complete in dist.toml includes (PCs 005-007 cover this). Phased approach with 23 PCs provides proof. Small releases via dist.toml first, workflow swap later.

## Security Notes
**CLEAR** — 1 advisory: cosign failure policy is explicitly continue-on-error (matching current behavior). Permissions are least-privilege. No secret handling changes (ANTHROPIC_API_KEY is not in release.yml).

## Rollback Plan

Reverse dependency order:
1. `git revert` the release.yml replacement commit — restores old hand-rolled workflow
2. Remove dist.toml (or `git revert` the dist.toml commit)
3. Cargo.toml is unchanged — no revert needed
4. Old release.yml is preserved in git history — always recoverable

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 1 warning (content bundling) |
| Security | CLEAR | 1 advisory (cosign policy) |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `dist.toml` | Create | US-001 |
| 2 | `Cargo.toml` | Review (no change) | US-001 |
| 3 | `.github/workflows/release.yml` | Replace | US-002, US-003 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-cargo-dist-release-pipeline/design.md` | Full design + Phase Summary |
