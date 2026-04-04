# ADR-0051: Pattern File Schema Design

## Status

Accepted (2026-04-04)

## Context

Pattern files need a machine-parseable, consistent format for agent discovery. The schema must support cross-language comparison, metadata-based search, and automated quality validation.

## Decision

YAML frontmatter with 5 required fields plus 9 mandatory markdown sections, enforced by a full quality gate in `ecc validate patterns`.

### Required Frontmatter Fields

- `name` — kebab-case pattern identifier
- `category` — must match parent directory name
- `tags` — list of descriptive tags
- `languages` — list from canonical set (rust, go, python, typescript, java, kotlin, csharp, cpp, swift, shell) or special value "all"
- `difficulty` — one of: beginner, intermediate, advanced

### Optional Frontmatter Fields

- `related-patterns` — cross-references to other pattern names
- `related-skills` — cross-references to ECC skills
- `unsafe-examples` — boolean, suppresses unsafe code warnings

### Required Sections (9)

Intent, Problem, Solution, Language Implementations, When to Use, When NOT to Use, Anti-Patterns, Related Patterns, References

### Quality Gate Validations

- Frontmatter field presence and value validation
- Category-directory consistency
- Required section presence with non-empty bodies
- Cross-reference resolution (related-patterns must exist)
- Language implementation heading matching (must match languages field)
- Unsafe code pattern scanning in code blocks
- Index coverage (every pattern listed in index.md)

## Consequences

- (+) Rich metadata enables agent discovery by category, language, difficulty, or tag
- (+) Full quality gate catches schema violations pre-merge via CI
- (+) Cross-reference validation ensures link integrity
- (-) Schema is relatively strict — adding patterns requires all 9 sections
- (-) Hand-maintained index.md works for Phase 1 but needs automation at scale
