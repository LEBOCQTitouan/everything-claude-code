# ADR 0030: Archetype Pattern for Component Scaffolding

## Status

Accepted (2026-03-29)

## Context

Creating new ECC components (agents, commands, skills, hooks) requires consulting multiple scattered references. The conventions in `rules/ecc/development.md` define required frontmatter fields, naming rules, and behavioral requirements, but there is no scaffolding tool that generates correct files from these conventions. BL-090 identified this gap.

The key design question: should a scaffolding skill duplicate the convention rules inline, or act as a thin pointer to the canonical source? Duplication creates a maintenance burden and risks divergence (the architect review flagged this as a MEDIUM knowledge duplication risk). A pointer pattern avoids duplication but requires the consumer to read two files.

## Decision

Adopt the **archetype pattern** for component scaffolding:

1. **Skills reference `rules/` as the canonical source of conventions** — they do not duplicate convention definitions. The skill provides concrete templates and quick-reference tables, with a pointer to the rules file for the authoritative specification.

2. **One archetype template per component type** — each component type (agent, command, skill, hook) has a concrete template in the skill that demonstrates correct frontmatter and body structure. These templates are the "archetypes" that scaffolding commands use as starting points.

3. **Scaffolding commands consume the skill** — the `/create-component` command loads the `ecc-component-authoring` skill for schema knowledge, then generates files that pass the existing `ecc validate` pipeline. The command does not embed its own schema definitions.

This follows the archetype pattern established by Hugo (content archetypes), moonrepo (code generators), and GitHub Docs (frontmatter templates).

## Consequences

- Single source of truth: `rules/ecc/development.md` remains authoritative for conventions. The skill adds only templates and behavioral checklists that the rules file does not cover.
- When conventions change, only `rules/ecc/development.md` needs updating. Skill templates may need updating if the required frontmatter fields change, but the skill does not need to restate the rules.
- Future meta-tooling skills (convention linting, component migration) should follow the same pointer pattern: reference `rules/`, don't duplicate.
- The skill stays under the 500-word limit because it avoids restating convention definitions.
