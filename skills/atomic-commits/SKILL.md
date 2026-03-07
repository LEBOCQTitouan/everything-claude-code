---
name: atomic-commits
description: Commit after every logical unit of change to preserve version history. Applies to all agents, commands, and workflows.
origin: ECC
---

# Atomic Commits

Commit after every logical unit of change. A logical unit is the smallest change that leaves the codebase in a valid state (build passes, tests pass).

## When to Commit

- After writing a failing test (RED phase)
- After making the test pass (GREEN phase)
- After refactoring (REFACTOR phase)
- After fixing a bug and verifying the fix
- After updating configuration or dependencies
- After renaming/moving files
- After updating documentation
- Before switching to a different concern or subtask

## How to Commit

1. Verify build passes (`npm run build` or equivalent)
2. Verify tests pass (`npm test` or equivalent)
3. Stage only the files related to this logical change
4. Write a conventional commit message: `<type>: <description>`

## Commit Granularity Examples

```
# TDD cycle for a new function
test: add calculateScore unit tests
feat: implement calculateScore
refactor: extract constants from calculateScore

# Bug fix
test: reproduce off-by-one in pagination
fix: correct pagination offset calculation

# Multi-file refactor
refactor: rename UserService to AccountService
refactor: update imports for AccountService rename

# Config change
chore: add strict mode to tsconfig
fix: resolve strict mode type errors
```

## Anti-Patterns

- Committing once at the end of a session with all changes bundled
- "WIP" commits with no description of what changed
- Mixing unrelated changes (bug fix + feature + refactor) in one commit
- Waiting to commit until "everything works" across multiple features
- Skipping commits during autonomous loops or batch operations

## Agent Integration

All ECC agents that modify code MUST commit after each logical change:
- **tdd-guide**: commit after RED, GREEN, and REFACTOR phases
- **build-error-resolver**: commit after each error is fixed
- **refactor-cleaner**: commit after each batch of deletions
- **code-reviewer**: commit after each review fix applied
- **doc-updater**: commit after documentation updates
- **loop-operator**: commit at each checkpoint
