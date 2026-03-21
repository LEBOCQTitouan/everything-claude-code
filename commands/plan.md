---
description: Plan any change — auto-classifies as dev/fix/refactor and delegates to the appropriate plan command.
allowed-tools: [Read, Grep, Glob, Bash, Skill, AskUserQuestion]
---

# Plan Router Command

> **Thin router**: This command classifies intent and delegates to `/plan-dev`, `/plan-fix`, or `/plan-refactor`. All workflow logic lives in the delegated command.

> **Do NOT enter Plan Mode. Do NOT directly edit `.claude/workflow/state.json`.** This is a thin router.

## Phase 0 — Validate Input

If `$ARGUMENTS` is empty or blank, use AskUserQuestion to ask:

> What would you like to plan? Describe the feature, bug fix, or refactoring you have in mind.

Use the answer as `$ARGUMENTS` for the next phase.

## Phase 1 — Intent Classification

### Check explicit flags first

If `$ARGUMENTS` starts with or contains one of these flags, use it directly:

| Flag | Intent |
|------|--------|
| `--dev` | dev |
| `--fix` | fix |
| `--refactor` | refactor |

Strip the flag from the arguments before delegating.

### Signal-word classification

If no explicit flag, classify by scanning `$ARGUMENTS` for signal words:

| Intent | Signal Words |
|--------|-------------|
| **dev** | add, implement, create, new, feature, support, enable, introduce, build |
| **fix** | fix, bug, broken, error, crash, fails, wrong, regression, not working |
| **refactor** | refactor, clean, restructure, extract, rename, move, simplify, decouple, split, merge |

Match against the **first strong verb or keyword** in the input. If multiple categories match, prioritize the main action verb (the first one that appears).

Default to **dev** if no signal words match.

## Phase 2 — Confirm Classification

Display exactly:

> **Classified as: `<type>`** — delegating to `/plan-<type>`.

If the classification confidence is low (no signal words matched, or multiple categories had equal matches), use AskUserQuestion with options `["dev", "fix", "refactor"]` to ask:

> "I classified this as `<type>` but I'm not confident. Which type fits best?"

If AskUserQuestion is unavailable, display the classification and proceed after a brief pause. For clear-cut cases, proceed directly.

If the user provides a correction, use the corrected value.

## Phase 3 — Delegate

Invoke the Skill tool to delegate to the classified plan command:

```
skill: "plan-<type>", args: "<original $ARGUMENTS>"
```

If the Skill invocation fails, instruct the user:

> Delegation failed. Please run directly: `/plan-<type> <your arguments>`

The delegated command handles everything from there: workflow-init, project detection, research, grill-me interview, adversarial review, and spec generation.
