---
id: BL-079
title: Conditional rule/skill loading via frontmatter applicability
scope: MEDIUM
target: /spec-dev
status: implemented
created: 2026-03-27
origin: Stripe Minions blog post — conditional rule application pattern
---

# BL-079: Conditional Rule/Skill Loading via Frontmatter

## Problem

ECC loads all rules and skills unconditionally regardless of the target project's stack. This causes prompt bloat and potential misdirection — e.g., Django skills loaded for a Rust project, Spring Boot patterns shown for a Python codebase. Stripe notes that "almost all agent rules at Stripe are conditionally applied based on subdirectories."

## Proposal

Add an `applies-to` field in rule/skill frontmatter that declares applicability conditions. The ECC install/init system and Claude Code's rule loading would then filter based on detected project characteristics:

```yaml
---
name: django-patterns
description: Django architecture patterns
applies-to:
  languages: [python]
  frameworks: [django]
  files: ["manage.py", "settings.py"]
---
```

Detection heuristics:
- **Language**: Check file extensions in project root (*.rs → rust, *.py → python, *.ts → typescript)
- **Framework**: Check for sentinel files (Cargo.toml → rust, package.json → node, manage.py → django)
- **Subdirectory**: Rules can target specific paths (e.g., `applies-to: { paths: ["src/frontend/"] }`)

## Ready-to-Paste Prompt

```
/spec-dev Conditional rule and skill loading based on project stack detection

Add an `applies-to` frontmatter field to rules and skills that declares when they
should be loaded. ECC's install/init commands should detect the project stack and
only install applicable rules.

Requirements:
- New frontmatter field: applies-to with languages, frameworks, files, paths arrays
- Project detection: scan file extensions, sentinel files (Cargo.toml, package.json,
  manage.py), and directory structure
- Rules without applies-to load unconditionally (backwards compatible)
- ecc install and ecc init filter rules based on detected stack
- ecc validate rules warns about rules loaded for non-matching projects
- Self-describing: each rule declares its own conditions, no central manifest

Inspired by Stripe Minions where "almost all agent rules are conditionally applied
based on subdirectories."
```

## Scope Estimate

MEDIUM — frontmatter schema change + detection heuristics + install/init filtering.

## Dependencies

None — backwards compatible since rules without `applies-to` load unconditionally.
