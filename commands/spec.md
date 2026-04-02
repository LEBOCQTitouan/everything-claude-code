---
description: Spec any change — auto-classifies as dev/fix/refactor and delegates to the appropriate spec command.
allowed-tools: [Read, Grep, Glob, Bash, Skill, AskUserQuestion]
---

# Spec Router Command

> **Thin router**: This command classifies intent and delegates to `/spec-dev`, `/spec-fix`, or `/spec-refactor`. All workflow logic lives in the delegated command.
>
> **Do NOT enter Plan Mode. Do NOT directly edit `.claude/workflow/state.json`.** This is a thin router.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before delegating, tell the user the classification result and which spec command will handle their request.

## Phase 0 — Validate Input

### Handle `--show-all` flag

If `$ARGUMENTS` contains `--show-all`, strip the flag and skip all in-work filtering below (show all open items regardless of worktree or lock status). This is an escape hatch for false-positive filtering.

### Active Session Detection

Before presenting the backlog picker, detect which items are currently being worked on:

1. **Worktree scan**: List directories in `.claude/worktrees/`. For each worktree name, extract all BL-NNN IDs using regex `(?i)bl-?(\d{3})` (case-insensitive, matches `bl119`, `bl-119`, `BL-119`; a name like `feature-bl-042-and-bl-055` matches both BL-042 and BL-055). Collect all matched BL-NNN IDs into a `claimed_by_worktree` set.

2. **Lock file scan**: List files in `docs/backlog/.locks/BL-NNN.lock`. For each lock file:
   - If the lock file's timestamp is > 24 hours old (stale TTL), remove it automatically
   - If the lock file references a worktree name that no longer exists in `.claude/worktrees/` (orphaned), remove it automatically
   - Otherwise, add the BL-NNN to a `claimed_by_lock` set

3. **Merge claims**: `claimed_items` = `claimed_by_worktree` ∪ `claimed_by_lock`

4. **Display active sessions**: Before the picker, display:
   > **Active sessions:**
   > - `<worktree-name>` → BL-NNN: <Title> (or "no BL match" for worktrees without BL-NNN pattern)
   >
   > **(N items in progress, hidden from picker)**

### Backlog Picker

If `$ARGUMENTS` is empty or blank:

1. Check if `docs/backlog/BACKLOG.md` exists
2. If it exists, read it and collect all entries with status `open`
3. **Filter**: Remove entries whose BL-NNN ID is in `claimed_items` (unless `--show-all` was passed)
4. If there are no remaining entries after filtering (all claimed or archived), fall back to the free-text prompt below
5. If there are open entries, present them via AskUserQuestion:
   - Each open backlog item becomes an option with label `"BL-NNN: <Title>"` and description `"Scope: <Scope> | Target: <Target>"`
   - AskUserQuestion automatically provides "Other" for custom input
6. If the user selects a backlog item (BL-NNN):
   - Read its full file at `docs/backlog/BL-NNN-<slug>.md` and extract the optimized prompt from the `## Optimized Prompt` section. Use that prompt as `$ARGUMENTS`
   - **Write lock file**: Create `docs/backlog/.locks/BL-NNN.lock` containing the current worktree name (from `git rev-parse --show-toplevel | xargs basename`) and ISO 8601 timestamp
7. If the user selects "Other" and types custom text, use that as `$ARGUMENTS`

**Fallback** (no backlog or no open entries after filtering): use AskUserQuestion to ask:

> What would you like to spec? Describe the feature, bug fix, or refactoring you have in mind.

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

> **Classified as: `<type>`** — delegating to `/spec-<type>`.

If the classification confidence is low (no signal words matched, or multiple categories had equal matches), use AskUserQuestion with options `["dev", "fix", "refactor"]` to ask:

> "I classified this as `<type>` but I'm not confident. Which type fits best?"

If AskUserQuestion is unavailable, display the classification and proceed after a brief pause. For clear-cut cases, proceed directly.

If the user provides a correction, use the corrected value.

## Phase 3 — Delegate

Invoke the Skill tool to delegate to the classified spec command:

```
skill: "spec-<type>", args: "<original $ARGUMENTS>"
```

If the Skill invocation fails, try the legacy name:

```
skill: "plan-<type>", args: "<original $ARGUMENTS>"
```

If both fail, instruct the user:

> Delegation failed. Please run directly: `/spec-<type> <your arguments>`

The delegated command handles everything from there: workflow-init, project detection, web research, grill-me interview, adversarial review, and spec generation.
