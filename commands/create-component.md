---
description: "Scaffold or update an ECC component (agent, command, skill, hook) with correct frontmatter, structure, and validation."
allowed-tools: [Bash, Read, Write, Edit, Grep, Glob, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Create Component

> **Narrative**: See narrative-conventions skill.

Scaffold a new ECC component or update an existing one with guided validation. Loads the `ecc-component-authoring` skill for schemas and templates.

## Arguments

`$ARGUMENTS` must match: `<type> <name> [update]`

- `<type>`: one of `agent, command, skill, hook`. If an unrecognized type is provided, reject with: "Invalid component type '<type>'. Valid types: agent, command, skill, hook."
- `<name>`: the component name. Normalize to lowercase-hyphens (replace uppercase with lowercase, underscores with hyphens). After normalization, validate against `[a-z][a-z0-9-]*` with max 50 characters. If invalid, reject with: "Invalid component name. Names must match [a-z][a-z0-9-]*, max 50 characters."
- `[update]`: if present, switch to Update Mode (Phase 2B)

## Phase 1: Input Validation

1. Parse `<type>` and `<name>` from arguments
2. Validate type is one of: agent, command, skill, hook
3. Normalize name to lowercase-hyphens
4. Validate normalized name against `[a-z][a-z0-9-]*`, max 50 chars
5. Determine target file path:
   - Agent: `agents/<name>.md`
   - Command: `commands/<name>.md`
   - Skill: `skills/<name>/SKILL.md`
   - Hook: `hooks/hooks.json` (entry) + `hooks/<name>.sh` (script)

## Phase 2A: Create Mode

If `[update]` is NOT specified:

### Existence Check

Check if the target file already exists. If it does, warn: "File `<path>` already exists. Overwrite?" Use `AskUserQuestion` with options `["Overwrite", "Cancel"]`. If "Cancel", stop.

### Grill-Me Interview

Conduct a brief interview to customize the scaffolded component. Ask about:

1. **Purpose**: "What is the purpose of this component?" (free text)
2. **Model choice**: For agents: "Which model should this agent use?" with options `["opus", "sonnet", "haiku"]`
3. **Read-only vs read-write**: For agents: "Is this a read-only analysis agent (no Write/Edit tools)?" with options `["Read-only", "Read-write"]`
4. **Tools needed**: For agents/commands: "Which tools does this component need?" (suggest defaults based on read-only choice)

### Scaffold

Use `EnterPlanMode` to preview the scaffolded file before writing:

#### Agent Scaffold

```yaml
---
name: <name>
description: "<purpose from interview>"
model: <model from interview>
tools: [<tools from interview>]
skills: []
---

# <Title from name>

<purpose from interview>

## When to Use

- <describe activation conditions>

## Workflow

1. <step 1>
2. <step 2>
```

#### Skill Scaffold

Create directory `skills/<name>/` and file `skills/<name>/SKILL.md`:

```yaml
---
name: <name>
description: "<purpose from interview>"
origin: ECC
---

# <Title>

## When to Activate

- <conditions>

## Reference

<content>
```

#### Command Scaffold

```yaml
---
description: "<purpose from interview>"
allowed-tools: [<tools from interview>]
---

# <Title>

<purpose>

## Arguments

`$ARGUMENTS` supports: <describe>

## Steps

1. <step>
```

#### Hook Scaffold

For hooks, guide the user through adding a JSON entry to `hooks/hooks.json`. Use atomic write: read the file, parse JSON, add the new entry to the appropriate event type array (`PreToolUse`, `PostToolUse`, or `Stop`), validate the JSON, write to a temp file via `mktemp`, then atomic rename via `mv`.

Also create the script file at `hooks/<name>.sh` with boilerplate:

```bash
#!/usr/bin/env bash
set -uo pipefail

# <purpose>
```

### Validate and Commit

1. Call `ExitPlanMode` — wait for user approval
2. Write the scaffolded file(s)
3. Run `ecc validate <type>` to verify the output passes validation
4. If validation fails, report the errors and do NOT commit. Leave the file on disk for manual correction.
5. If validation passes, you MUST commit immediately: `feat: scaffold <name> <type>`

## Phase 2B: Update Mode

If `[update]` IS specified:

### Load Existing

1. Check if the component exists. If it does not exist, inform the user and fall back to Create Mode (Phase 2A): "Component `<name>` does not exist. Falling back to create mode."
2. Read the existing file using the Read tool
3. If the file has malformed frontmatter (invalid YAML, missing `---` delimiters), report a parse error and exit: "Cannot update: frontmatter is malformed (invalid YAML). Fix manually before using update mode."

### Review and Edit

1. Use `EnterPlanMode` to present the current state vs convention requirements
2. Analyze the existing component against the `ecc-component-authoring` skill schemas
3. Identify frontmatter issues (missing fields, invalid values) and body improvements
4. Present findings and use `AskUserQuestion` to confirm each fix:
   - Frontmatter corrections (missing `allowed-tools`, invalid `model`, etc.)
   - Body content: tools list, model selection, body sections, and any custom fields
   - For hook update mode, read the existing hook entry from hooks.json and present it for modification via `AskUserQuestion`
5. Preserve all custom fields beyond the schema — do not remove fields the user has added
6. Call `ExitPlanMode` — wait for user approval

### Validate and Commit

1. Apply the approved changes
2. Run `ecc validate <type>` to verify
3. If validation fails, report errors and do NOT commit
4. If validation passes, you MUST commit immediately: `refactor: update <name> <type>`
