---
id: BL-090
title: "ECC component scaffolding — skill + /create-component command for agents, commands, skills, hooks"
scope: HIGH
target: "/spec dev"
status: implemented
tags: [meta-skill, scaffolding, agent, command, skill, hooks, developer-experience]
created: 2026-03-28
related: [BL-006, BL-007, BL-009]
---

# BL-090: ECC Component Scaffolding Skill + Command

## Problem

Creating new ECC components (agents, commands, skills, hooks) requires knowing multiple conventions scattered across `rules/ecc/development.md`, existing examples, and tribal knowledge. There's no scaffolding tool that generates correct frontmatter, file structure, and test stubs. Updating existing components (e.g., adding tools to an agent) is manual and error-prone.

## Proposed Solution

### 1. `ecc-component-authoring` Skill

New skill at `skills/ecc-component-authoring/SKILL.md` providing:

- **Frontmatter schemas** for each component type (agent, command, skill, hook)
- **Templates** with required fields pre-filled and placeholders for custom content
- **Naming conventions** (directory name = frontmatter `name` field, lowercase-hyphens)
- **Allowed-tools rules** per agent type (read-only agents get no Write/Edit)
- **Test requirements** per component type
- **Validation checklist** for quality gates

### 2. `/create-component` Command

New slash command that scaffolds + validates:

```bash
/create-component agent my-agent          # Scaffold new agent
/create-component skill my-skill          # Scaffold new skill
/create-component command my-command       # Scaffold new command
/create-component hook my-hook            # Scaffold new hook
/create-component update agent my-agent   # Update existing agent
```

Features:
- Generates correct file structure with frontmatter
- Validates against schemas (name matches directory, required fields present)
- For updates: reads existing file, presents current state, guides modification
- Runs `skill-stocktake` quality check after creation/update
- Commits the new/updated component

### 3. Update Mode

When updating existing components:
- Reads current frontmatter and content
- Presents what exists vs what conventions require
- Guides targeted modifications (add tool, update description, fix model)
- Validates the result against the schema

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Skill only or skill + command? | Both: skill for knowledge, command for scaffolding | User |
| 2 | Pipeline or direct edit? | /spec pipeline for the full implementation | Recommended |
| 3 | Create only or create + update? | Both creation and update mode | User |

## Ready-to-Paste Prompt

```
/spec dev

Create an ECC component scaffolding system:

1. `ecc-component-authoring` skill: frontmatter schemas, templates, naming
   conventions, allowed-tools rules, test requirements for agents/commands/
   skills/hooks. At skills/ecc-component-authoring/SKILL.md.

2. `/create-component` command: scaffolds new components with correct
   frontmatter + file structure. Update mode reads existing components and
   guides modifications. Validates against schemas. Runs skill-stocktake
   quality check. Commits result.

   Invocation: /create-component <type> <name> [update]
   Types: agent, command, skill, hook

See BL-090 for full analysis.
```
