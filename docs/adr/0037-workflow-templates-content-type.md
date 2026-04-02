# ADR 0037: Workflow Templates Content Type

## Status
Accepted

## Context
ECC ships content as Markdown files (agents, commands, skills, rules) installed to `~/.claude/`. GitHub Actions workflow templates are YAML files that need to be installed to a project's `.github/workflows/` directory — a different file format and install target. No existing content type fits this need.

## Decision
Create a new `workflow-templates/` top-level content directory for installable GitHub Actions YAML files. Each template is a standalone `.yml` file with ECC version header comments. Templates are distributed via a `/scaffold-workflows` slash command rather than through `ecc install` (which targets `~/.claude/`).

## Consequences
- Clean separation between Markdown knowledge (skills) and installable YAML (workflow templates)
- Enables dedicated validation rules for YAML structure, permissions, and action version pinning
- Requires adding `workflow-templates/` to release tarball bundling
- Future: may evolve into Rust-based `ecc validate workflow-templates` and `ecc init --with-workflows`
