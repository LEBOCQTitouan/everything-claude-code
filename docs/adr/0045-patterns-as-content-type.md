# ADR-0045: Patterns as Separate Content Type

## Status

Accepted (2026-04-04)

## Context

ECC needs a structured pattern library (~150 patterns across 15+ categories) for agent-assisted code generation and review. Three options were considered:

- **Option A**: Separate `patterns/` directory — new content type with its own validation, richer schema than skills
- **Option B**: Patterns as skills — put patterns under `skills/patterns/<category>/SKILL.md`, reusing existing validation
- **Option C**: Domain-modeled entities — create `ecc-domain/src/pattern/` with aggregate roots and repositories

## Decision

Option A — patterns are a separate content type in a top-level `patterns/` directory.

## Consequences

- (+) Zero domain model changes; no new ports, adapters, or aggregates
- (+) Richer schema than skills: tags, languages, difficulty, related-patterns, related-skills
- (+) 150+ files in nested hierarchy (category/pattern.md) is architecturally distinct from flat skills
- (+) Independent lifecycle — patterns can evolve without affecting skill validation
- (+) New `ecc validate patterns` follows established validation pipeline
- (-) Agents cannot auto-load patterns via Claude Code's native skill loader
- (-) New install step (`merge_patterns`) required alongside existing `merge_skills`
- (-) Agent discovery uses a separate `patterns:` frontmatter field (not `skills:`)
