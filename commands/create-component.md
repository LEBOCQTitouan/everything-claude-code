---
description: "Scaffold or update an ECC component (agent, command, skill, hook) with correct frontmatter, structure, and validation."
allowed-tools: [Bash, Read, Write, Edit, Grep, Glob, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Create Component

> Narrate per `skills/narrative-conventions/SKILL.md`.

Scaffold or update an ECC component. Loads `ecc-component-authoring` skill.

## Arguments

`$ARGUMENTS`: `<type> <name> [update]`
- `<type>`: agent, command, skill, hook (reject invalid)
- `<name>`: normalize to lowercase-hyphens, validate `[a-z][a-z0-9-]*`, max 50 chars
- `[update]`: switches to Update Mode

Target paths: Agent → `agents/<name>.md` | Command → `commands/<name>.md` | Skill → `skills/<name>/SKILL.md` | Hook → `hooks/hooks.json` + `hooks/<name>.sh`

## Phase 2A: Create Mode

1. **Existence check**: Warn if exists, AskUserQuestion Overwrite/Cancel.
2. **Interview**: Purpose (free text), Model (agents: opus/sonnet/haiku), Read-only? (agents), Tools needed.
3. **Scaffold**: `EnterPlanMode` to preview. Agent/Command/Skill templates with frontmatter. Hook: atomic JSON update + script boilerplate with `ECC_WORKFLOW_BYPASS` check.
4. **Validate & Commit**: `ExitPlanMode`, write file(s), `ecc validate <type>`. Pass → commit `feat: scaffold <name> <type>`. Fail → report, no commit.

## Phase 2B: Update Mode

1. **Load**: Check exists (fall back to Create if not). Read file. Reject if malformed frontmatter.
2. **Review**: `EnterPlanMode`. Analyze against `ecc-component-authoring` schemas. Present fixes via AskUserQuestion. Preserve custom fields.
3. **Validate & Commit**: Apply changes, `ecc validate <type>`. Pass → commit `refactor: update <name> <type>`. Fail → report, no commit.
