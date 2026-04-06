# ADR-0057: Release Automation Tooling

## Status

Accepted (2026-04-06)

## Context

ECC's release process uses manual version bumping (`scripts/bump-version.sh`, always patches), a custom CD pipeline (`cd.yml` auto-tag with `release` label gate), and manually maintained CHANGELOG.md. This produces incorrect semver (features get patch bumps), stale changelogs, and manual toil. Four options were evaluated: release-plz, cargo-release, knope, and enhanced-current-pipeline.

See: `docs/specs/2026-04-06-bl117-release-plz/evaluation.md` for full analysis.

## Decision

**Adopt release-plz** for automated version bumps, changelog generation, and release PR workflow.

### Alternatives Considered

1. **release-plz (chosen)**: CI-native, conventional commit analysis, auto-generates changelog, opens release PRs, integrates with cargo-dist. Score: 18/20.
2. **Enhanced current pipeline**: Add commitlint + wire `ecc analyze changelog`. Minimal risk but doesn't solve semver-from-commits. Score: 15/20.
3. **knope**: Newer tool, good changelog support, but less ecosystem validation and sparse workspace docs. Score: 14/20.
4. **cargo-release**: Mature workspace support but CLI-driven (not CI-native), no changelog generation, no commit analysis. Score: 12/20.

## Consequences

- **Positive**: Correct semver from conventional commits (feat→minor, fix→patch, breaking→major)
- **Positive**: Automated, structured changelog maintained by CI
- **Positive**: Release PR model provides human review gate before version bump
- **Positive**: Clean cargo-dist integration (release-plz tags → cargo-dist builds)
- **Negative**: cd.yml auto-tag job must be retired (breaking change to release workflow)
- **Negative**: CHANGELOG.md format changes from manual prose to structured format
- **Negative**: New GitHub Action secret/PAT may need `workflow` scope
- **Prerequisite**: Conventional commit linting CI (delivered in this spec)
