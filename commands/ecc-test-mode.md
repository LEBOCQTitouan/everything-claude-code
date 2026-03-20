---
description: "Test the ECC hook pipeline in the current repo or an isolated worktree"
---

# ECC Test Mode

Test the full ECC hook pipeline. By default, hooks are bypassed in the ECC repo via `.envrc` (`ECC_WORKFLOW_BYPASS=1`). This command provides two ways to re-enable them for testing.

## Quick Test (same repo)

Launch Claude Code with hooks active in the current repo:

```bash
ECC_WORKFLOW_BYPASS=0 claude
```

All 12 hooks will fire normally. Exit Claude Code to return to bypassed mode.

## Isolated Test (worktree)

Create an isolated worktree where hooks fire by default (no `.envrc`):

```bash
git worktree add ../ecc-test -b ecc-config-dev
```

If the branch already exists:

```bash
git worktree add ../ecc-test ecc-config-dev
```

Then test in isolation:

```bash
cd ../ecc-test && claude
```

The worktree has no `.envrc`, so `ECC_WORKFLOW_BYPASS` is unset and hooks fire normally.

## Cleanup

```bash
git worktree remove ../ecc-test
git branch -d ecc-config-dev
```
