# 0010. Skill Frontmatter Validation

Date: 2026-03-22

## Status

Accepted

## Context

The `ecc validate skills` command only checked that SKILL.md files existed and were non-empty. It did not validate that skills had the required YAML frontmatter fields (`name`, `description`, `origin`). This allowed malformed skills to pass validation and ship undetected — 4 existing skills were found with missing fields (3 missing `origin`, 1 missing `name`).

Skills should not have `model` or `tools` fields in their frontmatter, as these belong to agents (behavioral orchestrators), not skills (passive knowledge). No mechanism existed to warn about this.

## Decision

Enhance `validate_skills` in `crates/ecc-app/src/validate.rs` to:
1. Parse YAML frontmatter using the existing `extract_frontmatter` function from `ecc-domain`
2. Require `name`, `description`, and `origin` fields — report errors for missing or empty values
3. Report errors when no frontmatter block is found at all
4. Warn (non-blocking) when `model` or `tools` fields are present in skill frontmatter

## Consequences

- **Positive**: Malformed skills are caught at validation time, preventing silent quality degradation
- **Positive**: Clear distinction between skills (passive knowledge) and agents (behavioral orchestrators) is enforced
- **Negative**: Third-party or custom skills without required frontmatter fields will fail validation after this change
- **Negative**: The existing `skills_valid_dir` test fixture had to be updated to include valid frontmatter
