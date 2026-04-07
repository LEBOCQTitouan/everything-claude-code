# Spec: BL-117 Phase 2 — Implement release-plz Integration

## Problem Statement

ADR-0057 (ADOPT release-plz) was accepted in Phase 1 but not yet implemented. The current cd.yml auto-tag pipeline always increments patch version (ignoring semver from conventional commits), CHANGELOG.md is manually maintained, and scripts/bump-version.sh is the only version management tool. release-plz replaces all three with CI-native automation.

## Research Summary

- release-plz + cargo-dist complementary: release-plz owns versioning/changelog/tagging, cargo-dist owns binary builds
- release-plz.toml configures workspace-level versioning with publish=false
- git_release_enable=false prevents release-plz from creating GitHub Releases (cargo-dist does that)
- MarcoIeni/release-plz-action@v0.5 is the GitHub Action
- Rollout: dry-run first, verify, disable cd.yml, enable live mode

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Dry-run mode first | User requested verification before live | No |
| 2 | Ordered retirement | Prevents double-tagging | No |
| 3 | Reuse RELEASE_PAT with workflow scope | Enables downstream triggers | No |
| 4 | cd.yml renamed to .disabled (not deleted) | Easy rollback during transition | No |
| 5 | release-plz-action@v0.5 pinned | Reproducible CI | No |

## User Stories

### US-001: Create release-plz Configuration
**As a** maintainer, **I want** release-plz.toml configured for workspace-level versioning with publish=false.

#### Acceptance Criteria
- AC-001.1: release-plz.toml configures single workspace version
- AC-001.2: publish = false
- AC-001.3: git_release_enable = false
- AC-001.4: semver_check = true
- AC-001.5: changelog_path = "CHANGELOG.md"
- AC-001.6: ecc-integration-tests, ecc-test-support, xtask excluded
- AC-001.7: Given existing CHANGELOG.md content, when release-plz first runs, then historical entries are preserved (release-plz prepends new content)

#### Dependencies
- Depends on: none

### US-002: Add release-plz Workflow (Dry-Run)
**As a** maintainer, **I want** a release-plz workflow in dry-run mode.

#### Acceptance Criteria
- AC-002.1: Workflow runs on push to main
- AC-002.2: Given the workflow, when inspected, then it has: pinned action version (@v0.5), concurrency group, permissions (contents: write, pull-requests: write), timeout-minutes: 10
- AC-002.3: Uses RELEASE_PAT
- AC-002.4: Skips bot actor commits (if: github.actor != 'github-actions[bot]')
- AC-002.6: Given the workflow, when RELEASE_PAT is used, then it has workflow scope (enables downstream workflow triggers)
- AC-002.5: Given the workflow in dry-run, when inspected, then the release-plz command is "release-plz release-pr" (not "release-plz release") and no tag/publish step exists

#### Dependencies
- Depends on: US-001

### US-003: Retire cd.yml Auto-Tag
**As a** maintainer, **I want** cd.yml retired.

#### Acceptance Criteria
- AC-003.1: Auto-tag job removed
- AC-003.2: Given cd.yml is retired, when a push lands on main, then the only active version-management workflow is release-plz.yml (verified by examining .github/workflows/ for auto-tag patterns)
- AC-003.3: Comment documents retirement
- AC-003.4: Separate commit from US-002
- AC-003.5: Given cd.yml file, when retired, then the file is renamed to cd.yml.disabled (not deleted) for easy rollback

#### Dependencies
- Depends on: US-002

### US-004: Deprecate bump-version.sh
#### Acceptance Criteria
- AC-004.1: Deprecation header comment
- AC-004.2: Script still functions

#### Dependencies
- Depends on: US-003

### US-005: Enable Live Mode
#### Acceptance Criteria
- AC-005.1: Given the live-mode workflow, when a release PR is merged, then the release-plz-action creates a git tag matching `v{version}` pattern via RELEASE_PAT
- AC-005.2: Given the tag format matches `v*`, when the tag is pushed, then release.yml triggers (verified by matching the `on.push.tags` pattern in release.yml)
- AC-005.3: Separate commit

#### Dependencies
- Depends on: US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| .github/workflows/release-plz.yml | CI/CD | New |
| .github/workflows/cd.yml | CI/CD | Retire |
| release-plz.toml | Config | New |
| scripts/bump-version.sh | Scripts | Deprecate |

## Constraints
- release.yml and dist.toml MUST NOT be modified
- cd.yml retirement in separate commit
- Live mode in separate commit

## Non-Requirements
- crates.io publishing, per-crate versioning, custom templates, new ADR

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| GitHub Actions | New + Retired | release-plz replaces cd.yml |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Changelog | CHANGELOG | CHANGELOG.md | Add entry |
| CI | CLAUDE.md | CLAUDE.md | Update CD description |

## Open Questions
None.
