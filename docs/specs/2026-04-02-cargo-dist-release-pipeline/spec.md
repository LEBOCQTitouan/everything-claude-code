# Spec: BL-112 Migrate Release Pipeline to cargo-dist

## Problem Statement

The current release pipeline (`.github/workflows/release.yml`) is a hand-rolled 260+ line workflow with cross-compilation matrix, tarball packaging, checksum generation, and optional cosign signing. It has been rewritten 11 times, has fragile conditional logic (silent failures for missing binaries and unsigned releases), and duplicates capabilities that cargo-dist provides declaratively. Migration to cargo-dist would reduce maintenance burden, gain installers (shell/PowerShell/Homebrew), SBOM generation, and standardize the release process.

## Research Summary

- **cargo-dist natively supports all 5 ECC targets** (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc) using native CI runners per platform
- **Checksums are built-in** (sha256 default, configurable) but **cosign/Sigstore is NOT supported** -- must be added as custom post-build step; GitHub Artifact Attestations are an alternative
- **Installers are a key gain**: shell script, PowerShell, Homebrew formulae, npm packages -- all auto-generated from config
- **cargo-dist auto-generates its own release.yml** with plan-build-host-publish-announce pipeline -- reduces hand-maintained CI complexity
- **Migration pitfall**: cargo-dist regenerates CI on `cargo dist init`, which can overwrite custom modifications (cosign job). Use `allow-dirty` or manual merge
- **Additional capabilities**: cargo-auditable (dependency tree in binaries), CycloneDX SBOM, machine-readable release manifests
- **Recommended alongside release-plz** for automated semver bumping (complementary tool, separate backlog item BL-117)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Adopt cargo-dist replacing hand-rolled release.yml | Declarative config, auto-generated CI, installers, SBOM -- reduces 260+ lines of hand-maintained YAML to dist.toml config | Yes |
| 2 | Custom cosign job as post-build extension | cargo-dist lacks native Sigstore/cosign support. Add as custom job after host stage. Preserves existing signing capability. | Yes |
| 3 | Phased migration: dist.toml first, workflow swap later | Safer -- validate locally before changing CI. dist.toml can coexist with current release.yml. | No |
| 4 | Verify tarball contents match current format | ecc update and install.sh depend on tarball structure. Verify binaries, content dirs, shims are present. | No |
| 5 | First cargo-dist release as pre-release (v*-rc1) | De-risk by shipping first cargo-dist release as RC for manual verification | No |

## User Stories

### US-001: cargo-dist Configuration

**As a** maintainer, **I want** cargo-dist configured for the ECC workspace, **so that** binary distribution is declarative and locally testable.

#### Acceptance Criteria

- AC-001.1: Given `dist.toml` exists in the workspace root, when `cargo dist build --artifacts=local` runs on the native host, then it produces an archive for the host target; full 5-target builds are verified in CI only (cross-compilation requires CI runners)
- AC-001.2: Given `dist.toml` configuration, when inspected, then it lists all 5 compilation targets explicitly
- AC-001.3: Given `dist.toml`, when inspected, then it includes both `ecc` and `ecc-workflow` binaries
- AC-001.4: Given `dist.toml`, when inspected, then it configures sha256 checksums
- AC-001.5: Given `dist.toml`, when inspected, then it includes content directories (agents, commands, skills, rules, hooks, contexts, mcp-configs, examples, schemas) and install.sh in the archive
- AC-001.6: Given `cargo dist build` output, when compared to current tarball contents, then both contain the same set of binaries and content directories

#### Dependencies

- Depends on: none

### US-002: Release Workflow Migration

**As a** maintainer, **I want** the release.yml replaced with cargo-dist-generated workflow, **so that** release CI is auto-maintained.

#### Acceptance Criteria

- AC-002.1: Given `cargo dist generate-ci github` runs, when it produces a release.yml, then it is committed to `.github/workflows/`
- AC-002.2: Given the generated release.yml, when a `v*` tag is pushed, then all 5 targets are built and published as a GitHub Release
- AC-002.3: Given the generated release.yml, when inspected, then permissions are least-privilege
- AC-002.4: Given the old hand-rolled release.yml, when the migration completes, then it is removed
- AC-002.5: Given the CD pipeline (`cd.yml`), when inspected after migration, then it remains unchanged and functional

#### Dependencies

- Depends on: US-001

### US-003: Custom Cosign Signing Job

**As a** maintainer, **I want** cosign signing preserved as a custom post-build job, **so that** release artifacts continue to have Sigstore signatures.

#### Acceptance Criteria

- AC-003.1: Given the generated release.yml, when extended with a custom job, then a cosign signing step runs after the host stage
- AC-003.2: Given the cosign job, when it signs release archives, then `.sig` files are uploaded alongside the archives
- AC-003.3: Given the cosign job, when it fails, then the release still proceeds (continue-on-error: true, matching current behavior)
- AC-003.4: Given `cargo dist generate-ci` is re-run, when the cosign custom job exists, then it is preserved via cargo-dist's `plan.jobs` custom job configuration in `dist.toml` (not manual merge)

#### Dependencies

- Depends on: US-002

### US-004: Validation and Verification

**As a** maintainer, **I want** the migration verified before cutting a real release, **so that** no release artifacts are broken.

#### Acceptance Criteria

- AC-004.1: Given `cargo dist build` runs locally, when output is inspected, then archive contents match expected structure (binaries + content dirs + shims + install.sh)
- AC-004.2: Given a pre-release tag (v*-rc1), when the cargo-dist pipeline runs, then all 5 target archives are produced with checksums
- AC-004.3: Given the pre-release artifacts, when downloaded and extracted, then `ecc version` exits 0 and prints the version string, and `ecc-workflow --version` exits 0
- AC-004.4: Given the `ecc update --version <rc-version>` command, when it runs, then it exits 0 and the installed `ecc version` matches the RC version

#### Dependencies

- Depends on: US-002, US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `dist.toml` | Config (new) | New cargo-dist configuration file |
| `.github/workflows/release.yml` | CI (replace) | Replace hand-rolled with cargo-dist generated + custom cosign |
| `Cargo.toml` | Config (modify) | May need `[workspace.metadata.dist]` for workspace-level config |
| `xtask/src/deploy.rs` | App (no change expected) | Deploy task operates on local builds, not release artifacts -- no change anticipated |
| `crates/ecc-app/src/update.rs` | App (review) | Verify ecc update parses new tarball format |

## Constraints

- All 5 compilation targets must be preserved
- Cosign signing must be preserved (even if optional/continue-on-error)
- `cd.yml` auto-tagging pipeline must remain unchanged
- Tarball must contain: `ecc`, `ecc-workflow` binaries, content dirs, `install.sh`, shims
- `ecc update` command must work with new tarball format
- First cargo-dist release must be a pre-release for verification
- Rollback strategy: the old release.yml is preserved in git history via the deletion commit; `git revert <commit>` restores it. The dist.toml addition is independently revertible.

## Non-Requirements

- Not changing `cd.yml` auto-tagging pipeline
- Not adopting release-plz (separate backlog item BL-117)
- Not implementing cargo-semver-checks (separate BL-110)
- Not implementing SLSA attestations (separate BL-118)
- Not changing Windows packaging asymmetry (conscious design choice)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Release pipeline | Replace | All release artifacts change packaging tool |
| ecc update | Review | Must parse new tarball format |
| install.sh | Include | Must be bundled in cargo-dist archives |
| cd.yml | None | Auto-tagging unchanged |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New tool adoption | HIGH | CLAUDE.md | Update release commands, mention cargo-dist |
| ADR | MEDIUM | docs/adr/ | Write 2 ADRs (cargo-dist adoption, cosign strategy) |
| Pipeline change | MEDIUM | rules/ecc/github-actions.md | Update release.yml section |
| Changelog | LOW | CHANGELOG.md | Add migration entry |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage: adopt cargo-dist given no cosign support? | Adopt cargo-dist, add cosign as custom step | User |
| 2 | Target architecture | Full cargo-dist replacement: plan/build/host/custom-cosign/announce | Recommended |
| 3 | Step independence | Phased: dist.toml first, workflow swap later | User |
| 4 | Downstream dependencies | Verify tarball contents match current format | Recommended |
| 5 | Rename vs behavioral change | Accept behavioral changes, verify equivalence | Recommended |
| 6 | Performance budget | No performance concerns | Recommended |
| 7 | ADR decisions | Two ADRs: cargo-dist adoption + cosign strategy | User |
| 8 | Test safety net | cargo dist build local + manual RC verification | Recommended |

**Smells addressed**: #1 (fragile cosign), #4 (pipeline churn), #5 (hand-rolled CI duplication)
**Smells deferred**: #2 (fragile binary copy — cargo-dist handles), #3 (Windows asymmetry — conscious choice), #6 (no SBOM — future), #7 (no installers — cargo-dist adds automatically)

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | cargo-dist Configuration | 6 | none |
| US-002 | Release Workflow Migration | 5 | US-001 |
| US-003 | Custom Cosign Signing Job | 4 | US-002 |
| US-004 | Validation and Verification | 4 | US-002, US-003 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Local cargo dist build produces host archive | US-001 |
| AC-001.2 | dist.toml lists all 5 targets | US-001 |
| AC-001.3 | dist.toml includes ecc + ecc-workflow | US-001 |
| AC-001.4 | sha256 checksums configured | US-001 |
| AC-001.5 | Content dirs + install.sh in archive | US-001 |
| AC-001.6 | Tarball contents match current format | US-001 |
| AC-002.1 | Generated release.yml committed | US-002 |
| AC-002.2 | 5 targets built on v* tag push | US-002 |
| AC-002.3 | Least-privilege permissions | US-002 |
| AC-002.4 | Old release.yml removed | US-002 |
| AC-002.5 | cd.yml unchanged | US-002 |
| AC-003.1 | Cosign job runs after host stage | US-003 |
| AC-003.2 | .sig files uploaded | US-003 |
| AC-003.3 | Cosign failure is non-blocking | US-003 |
| AC-003.4 | Cosign job preserved via plan.jobs | US-003 |
| AC-004.1 | Local archive contents match expected | US-004 |
| AC-004.2 | Pre-release produces 5 archives + checksums | US-004 |
| AC-004.3 | Binaries execute correctly (exit 0) | US-004 |
| AC-004.4 | ecc update works with new format (exit 0, version match) | US-004 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict |
|-----------|----------|----------|---------|
| Ambiguity | 72 | 88 | PASS |
| Edge Cases | 61 | 78 | PASS |
| Scope | 85 | 86 | PASS |
| Dependencies | 90 | 86 | PASS |
| Testability | 68 | 80 | PASS |
| Decisions | 88 | 82 | PASS |
| Rollback | 70 | 78 | PASS |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-cargo-dist-release-pipeline/spec.md` | Full spec + Phase Summary |
