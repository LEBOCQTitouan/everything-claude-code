# Release Automation Evaluation: BL-117

## Executive Summary

Evaluated 4 options for automating semver version bumps and changelog generation in the ECC Rust workspace. **Verdict: ADOPT release-plz** — it provides the best combination of CI-native automation, conventional commit awareness, cargo-dist compatibility, and workspace support.

## Current State

- **Versioning**: Manual. `scripts/bump-version.sh` uses `sed` to increment patch in root `Cargo.toml`. Always patches — ignores semver semantics from commit types.
- **Tagging**: `cd.yml` auto-tags on main push when PR has `release` label. Computes next patch, commits with `[skip cd]`, tags `vX.Y.Z`.
- **Changelog**: Manually maintained. `ecc analyze changelog` CLI exists for local use but isn't wired into CI.
- **Release builds**: `release.yml` (cargo-dist) triggers on `v*` tags. Cross-compiles 5 targets, packages, creates GitHub Release.
- **Commit format**: Conventional commits by convention, not enforced in CI (addressed by US-002 commit-lint workflow).

## Evaluation Dimensions

### 1. Conventional Commit Integration

| Tool | Score (1-5) | Justification |
|------|:-----------:|---------------|
| **release-plz** | 5 | Native conventional commit parsing. Maps feat→minor, fix→patch, breaking→major automatically. |
| **cargo-release** | 2 | No commit analysis. Version must be specified manually or via flag. |
| **knope** | 4 | Parses conventional commits. Supports custom commit-to-section mapping. |
| **Enhanced current** | 3 | Would need `ecc analyze changelog` wired to determine bump. Partial — doesn't auto-determine version level. |

### 2. Workspace-Aware Version Management

| Tool | Score (1-5) | Justification |
|------|:-----------:|---------------|
| **release-plz** | 4 | Supports workspace versioning via `release-plz.toml` config. Dry-run confirmed: correctly reads `version.workspace = true` and proposes single version bump. Slightly less tested than independent versioning mode. |
| **cargo-release** | 5 | Mature workspace support. `cargo release --workspace` bumps all crates together. Battle-tested. |
| **knope** | 3 | Workspace support exists but documentation is sparse. Less ecosystem validation. |
| **Enhanced current** | 4 | Current `scripts/bump-version.sh` already handles workspace-level bump. No change needed for this dimension. |

### 3. CD Pipeline Compatibility (cargo-dist)

| Tool | Score (1-5) | Justification |
|------|:-----------:|---------------|
| **release-plz** | 5 | Complementary to cargo-dist by design. release-plz owns versioning+tagging, cargo-dist owns binary builds. Set `git_release_enable = false`, `publish = false`. Tags in `vX.Y.Z` format matching release.yml trigger. |
| **cargo-release** | 4 | Compatible but requires manual trigger. Not CI-native — needs wrapper workflow. |
| **knope** | 3 | Can create tags but less documented integration with cargo-dist. |
| **Enhanced current** | 5 | No change to existing pipeline. cd.yml and release.yml remain as-is. |

### 4. Changelog Generation Quality

| Tool | Score (1-5) | Justification |
|------|:-----------:|---------------|
| **release-plz** | 4 | Auto-generates keep-a-changelog format from commits. Supports custom templates. Preserves existing content. |
| **cargo-release** | 1 | No changelog generation. Separate tool needed. |
| **knope** | 4 | Good changelog generation with custom section mapping. |
| **Enhanced current** | 3 | Would use existing `ecc analyze changelog` Rust code. Output format already matches project style. But requires CI wiring that doesn't exist yet. |

## Comparison Matrix

| Dimension | release-plz | cargo-release | knope | Enhanced current |
|-----------|:-----------:|:------------:|:-----:|:----------------:|
| Conventional commits | 5 | 2 | 4 | 3 |
| Workspace versioning | 4 | 5 | 3 | 4 |
| CD pipeline compat | 5 | 4 | 3 | 5 |
| Changelog quality | 4 | 1 | 4 | 3 |
| **Total** | **18** | **12** | **14** | **15** |

## Breaking Changes to Existing Pipeline

### release-plz adoption
- **cd.yml**: Must retire auto-tag job entirely. release-plz subsumes version bump + tag. **CRITICAL conflict** if both run — double-tagging.
- **release.yml**: No changes. Triggered by `v*` tags created by release-plz.
- **scripts/bump-version.sh**: Deprecated for CI use. Kept for manual emergency bumps.
- **CHANGELOG.md**: Format changes from manual prose to structured keep-a-changelog.
- **`release` label convention**: Replaced by release-plz's release PR model.

### cargo-release adoption
- **cd.yml**: Would need rewrite to invoke `cargo release` instead of custom script.
- Changelog would need a separate tool.

### knope adoption
- **cd.yml**: Would need rewrite.
- Less documented integration path.

### Enhanced current
- **No breaking changes**. Additive only (commit-lint + changelog automation).
- But doesn't solve semver-from-commits (always patches).

## Workspace Versioning Dry-Run Results

### release-plz
Configured with `release-plz.toml`:
```toml
[workspace]
changelog_path = "CHANGELOG.md"
semver_check = true
publish = false
git_release_enable = false
```
Dry-run correctly:
- Reads `version.workspace = true` across all 9 crates
- Proposes single version bump based on commit history
- Generates unified changelog at root

### cargo-release
```bash
cargo release --dry-run --no-publish patch
```
Correctly bumps workspace version. Mature and reliable. But no conventional commit analysis — must specify bump level manually.

## Verdict

**ADOPT release-plz.** It scores highest overall (18/20) and uniquely combines CI-native PR workflow with conventional commit analysis, workspace support, and cargo-dist compatibility. The main risk (workspace versioning being less tested) is mitigated by the dry-run validation and the fact that ECC doesn't publish to crates.io.

Implementation plan: Phase 2 spec to cover cd.yml retirement, release-plz.toml configuration, release-plz GitHub Action workflow, and CHANGELOG.md format migration. The commit-lint workflow (already delivered in this phase) is a prerequisite.

Rollout sequence:
1. Add release-plz workflow in dry-run mode (observe-only)
2. Verify release PRs and changelog output
3. Disable cd.yml auto-tag job
4. Enable release-plz in live mode
