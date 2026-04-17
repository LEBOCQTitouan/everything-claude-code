# Tool Manifest Authoring Guide

This guide explains how to add tools and presets to the ECC tool manifest.

The canonical manifest lives at `manifest/tool-manifest.yaml` in the repository root.
It is the single source of truth for all Claude Code tool identifiers ECC may reference.

## Adding a Tool

To register a new Claude Code tool in ECC:

1. Open `manifest/tool-manifest.yaml`
2. Add the tool name to the `tools:` section (PascalCase, matching the Claude Code identifier exactly):

```yaml
tools:
  - Read
  - Write
  - MyNewTool   # ← add here
```

3. Run `ecc validate agents` (and `commands`, `teams`, `skills`) to confirm no references are broken
4. Optionally add the new tool to one or more presets (see "Adding a Preset")

**Note**: Tool names are case-sensitive and must exactly match Claude Code's tool identifiers.

## Adding a Preset

Presets bundle commonly-used tool combinations under a named key.
Agents, commands, teams, and skills reference presets via `tool-set:` in their YAML frontmatter.

To add a new preset:

1. Open `manifest/tool-manifest.yaml`
2. Add an entry under the `presets:` section. The preset name must be kebab-case
   (`^[a-z][a-z0-9-]*[a-z0-9]$`):

```yaml
presets:
  my-new-preset:    # ← add here
    - Read
    - Write
    - Grep
```

3. All tools referenced in the preset must already be declared in the `tools:` section
4. Run `ecc validate agents` to confirm the new preset is valid
5. Reference the preset in agent/command frontmatter:

```yaml
---
name: my-agent
tool-set: my-new-preset
---
```

At install time, `tool-set: my-new-preset` is expanded to `tools: [Read, Write, Grep]`
so the Claude Code runtime sees a standard inline tool list.

## Preset Naming Rules

- Kebab-case only: `^[a-z][a-z0-9-]*[a-z0-9]$`
- Must start and end with a lowercase letter or digit
- No underscores, dots, or uppercase letters
- Examples: `readonly-analyzer`, `code-writer`, `tdd-executor`

## Inline Extension

An agent may combine a preset with additional inline tools:

```yaml
---
name: my-agent
tool-set: readonly-analyzer
tools: [TodoWrite]
---
```

The effective tool list is the union of preset tools and inline tools.
Any inline tool not in the preset emits a WARN at validation time (exit 0).

## Validation

Run any of the following to validate manifest references:

```bash
ecc validate agents
ecc validate commands
ecc validate teams
ecc validate conventions
```

A parse error, unknown preset reference, or invalid preset name results in exit 1.
