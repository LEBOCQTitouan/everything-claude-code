# Spec: ECC Help Skill

Source: BL-151 | Scope: LOW | Content-only

## Problem Statement

New Claude Code sessions working on ECC lack quick orientation. A dedicated help skill provides a concise, loadable reference for slash commands, pipeline flow, CLI commands, and key concepts.

## Research Summary

- ECC skills follow `name`/`description`/`origin` frontmatter + sections under 500 words
- CLAUDE.md is authoritative; skill complements, not duplicates
- `configure-ecc` skill covers installation; this covers usage

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Single SKILL.md | Matches skill convention | No |
| 2 | Complement CLAUDE.md | Avoid duplication | No |
| 3 | Under 500 words | Skill v1 limit | No |

## User Stories

### US-001: ECC Help Skill File

**As a** Claude Code user, **I want** a help skill explaining ECC tooling, **so that** I orient quickly.

- AC-001.1: `skills/ecc-help/SKILL.md` exists with valid frontmatter
- AC-001.2: Contains sections: Pipeline Overview, Slash Commands, CLI Commands, Key Concepts
- AC-001.3: Pipeline covers /spec → /design → /implement
- AC-001.4: Slash Commands lists major commands with descriptions
- AC-001.5: CLI Commands lists top ecc subcommands
- AC-001.6: Key Concepts covers worktree, workflow state, tool-set, party panel
- AC-001.7: Valid markdown, non-empty, under ~500 words

**Dependencies:** none

## Affected Modules

`skills/ecc-help/SKILL.md` (new) — content only

## Constraints
Under 500 words. Complement CLAUDE.md.

## Non-Requirements
Not a tutorial. Not exhaustive docs. Not interactive.

## E2E Boundaries Affected
None.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | project | CHANGELOG.md | Add entry |

## Open Questions
None.
