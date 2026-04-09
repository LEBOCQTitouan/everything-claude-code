---
name: ecc-component-authoring
description: Frontmatter schemas, templates, and behavioral requirements for authoring ECC components (agents, commands, skills, hooks).
origin: ECC
---

# ECC Component Authoring

Canonical conventions live in `rules/ecc/development.md`. This skill provides templates and behavioral checklists per ADR-0030 (archetype pattern).

## Agent Template

```yaml
---
name: <agent-name>
description: "<one-line purpose>"
model: opus|sonnet|haiku
tools: [Read, Grep, Glob]
skills: []
---
```

Required: `name`, `description`, `tools`, `model`. File: `agents/<name>.md`.

## Command Template

```yaml
---
description: "<one-line purpose>"
allowed-tools: [Bash, Read, Grep, Glob]
---
```

Required: `description`, `allowed-tools`. File: `commands/<name>.md`.

## Skill Template

```yaml
---
name: <skill-name>
description: "<one-line purpose>"
origin: ECC
---
```

Required: `name`, `description`, `origin: ECC`. Directory: `skills/<name>/SKILL.md`. Directory name MUST match `name` field.

## Hook Structure

Hooks are JSON entries in `hooks/hooks.json`, not markdown files:

```json
{
  "PreToolUse": { "<matcher>": [{"type": "command", "command": "..."}] },
  "PostToolUse": { "<matcher>": [{"type": "command", "command": "..."}] },
  "Stop": [{"type": "command", "command": "..."}]
}
```

Event types: `PreToolUse`, `PostToolUse`, `Stop`. Hook object: `{type, command}`.

Script conventions: `set -uo pipefail`, atomic writes via `mktemp` + `mv`.

## Behavioral Requirements

- Agents with 4+ steps: MUST include TodoWrite with graceful degradation
- Agents spawning subagents: MUST specify `allowedTools` on every spawn
- Read-only analysis agents: MUST NOT have `Write` or `Edit` in tools
- Adversary agents: MUST have `skills: ["clean-craft"]` and SHOULD have `memory: project`
- Commands: MUST have `allowed-tools` listing every tool used
- Skills: MUST be under 500 words for v1

## Naming

- Lowercase-hyphens only: `[a-z][a-z0-9-]*`, max 50 chars
- File naming: `agents/<name>.md`, `commands/<name>.md`, `skills/<name>/SKILL.md`

## Validation

Run `ecc validate <type>` after creation. Scaffolded output must pass immediately.
