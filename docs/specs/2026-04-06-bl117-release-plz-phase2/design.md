# Solution: BL-117 Phase 2 — release-plz Integration

## Spec Reference
Concern: dev, Feature: bl117-release-plz-phase2

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | release-plz.toml | create | Workspace config: publish=false, git_release_enable=false, semver_check=true | US-001 |
| 2 | .github/workflows/release-plz.yml | create | Dry-run workflow using release-plz-action@v0.5 | US-002 |
| 3 | .github/workflows/cd.yml → cd.yml.disabled | rename | Retire auto-tag (release-plz subsumes) | US-003 |
| 4 | scripts/bump-version.sh | modify | Add deprecation header | US-004 |
| 5 | CHANGELOG.md | modify | Add BL-117 Phase 2 entry | Doc plan |

## SOLID Assessment
N/A — no Rust code.

## Robert's Oath Check
CLEAN — ordered rollout (dry-run first), separate commits, easy rollback via cd.yml.disabled.

## Security Notes
CLEAR — RELEASE_PAT reused (same scope), no new secrets, workflow scope needed for downstream triggers.

## Rollback Plan
1. Revert CHANGELOG entry
2. Restore bump-version.sh (remove deprecation header)
3. Rename cd.yml.disabled back to cd.yml
4. Delete .github/workflows/release-plz.yml
5. Delete release-plz.toml

## Bounded Contexts Affected
No bounded contexts affected — no domain files modified.
