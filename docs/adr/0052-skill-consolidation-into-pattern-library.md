# ADR-0052: Skill Consolidation into Pattern Library

## Status

Accepted (2026-04-05)

## Context

ECC had 9 language-specific skills (rust-patterns, python-patterns, golang-patterns, kotlin-patterns, csharp-patterns, cpp-coding-standards, api-design, deployment-patterns, docker-patterns) that overlapped significantly with the pattern library introduced in ADR-0050. Maintaining both systems created duplication, inconsistent recommendations, and confusion about which source agents should consult.

## Decision

Remove all 9 language-specific skills and absorb their content into the pattern library. Agent frontmatter is updated to use `patterns:` field instead of `skills:` references to removed skills. No deprecation period — direct replacement.

### Alternatives Considered

1. **Deprecate then remove** — Keep skills as thin redirects pointing to pattern library for one release cycle. Rejected: adds maintenance burden and delays consolidation.
2. **Keep both systems** — Pattern library as parallel reference, skills unchanged. Rejected: duplication is the problem we're solving.
3. **Remove and replace (chosen)** — Direct removal with content migration checklist. Simplest, cleanest, and the pattern library already contains richer content than the skills.

## Consequences

- 9 skill directories deleted; agents reference `patterns:` categories instead
- Skills validation count decreases; pattern validation count increases
- Any external references to removed skills (bookmarks, docs) will break
- Rollback possible via `git revert` of the separate skill-removal commit (AC-017.7)
