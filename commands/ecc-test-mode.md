---
description: "Set up an isolated worktree for testing ECC config changes"
---

# ECC Test Mode

Create an isolated git worktree for safely testing ECC configuration changes before applying them to your real setup.

## What This Command Does

1. Creates a git worktree at `../ecc-test` on branch `ecc-config-dev`
2. Prints usage instructions for the three test modes

## Setup

```bash
git worktree add ../ecc-test -b ecc-config-dev
```

If the branch already exists, attach to it:

```bash
git worktree add ../ecc-test ecc-config-dev
```

## Modes

| Mode | Env Var | Behavior |
|------|---------|----------|
| `use` | `ECC_MODE=use` | Normal — uses installed ECC config from `~/.claude/` |
| `test` | `ECC_MODE=test` | Dev — uses config from the worktree at `../ecc-test` |
| `test-safe` | `ECC_MODE=test-safe` | Rollback-safe — uses worktree config but reverts on any hook/agent error |

## Usage

After setup, switch modes by setting the environment variable:

```bash
# Normal mode (default)
export ECC_MODE=use

# Test your config changes
export ECC_MODE=test

# Test with automatic rollback on errors
export ECC_MODE=test-safe
```

Edit files in `../ecc-test`, then run Claude Code with `ECC_MODE=test` to validate changes before merging back.

## Cleanup

```bash
git worktree remove ../ecc-test
git branch -d ecc-config-dev
```
