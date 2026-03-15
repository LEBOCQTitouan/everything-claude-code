# Git Workflow

> **MANDATORY WORKFLOW**: The workflow described in this file is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

## Commit Message Format
```
<type>: <description>

<optional body>
```

Types: feat, fix, refactor, docs, test, chore, perf, ci

Note: Attribution disabled globally via ~/.claude/settings.json.

## Commit Cadence — Atomic Commits

Commit after every logical unit of change. Do NOT batch multiple logical changes into a single commit. A logical unit is the smallest change that leaves the codebase in a valid state.

### When to Commit

| Trigger | Example |
|---------|---------|
| Test passes (RED→GREEN) | `test: add calculateScore unit tests` then `feat: implement calculateScore` |
| Refactor complete (GREEN→REFACTOR) | `refactor: extract constants from calculateScore` |
| Bug fix verified | `fix: handle null input in parseConfig` |
| Config/infra change applied | `chore: add eslint rule for no-console` |
| Documentation updated | `docs: update API reference for /markets` |
| Dependency added/removed | `chore: add zod for schema validation` |
| File renamed/moved | `refactor: move utils to lib/` |

### Rules

1. **One concern per commit** — A commit touches one logical change. Mixing a bug fix with a refactor is two commits.
2. **Build must pass** — Never commit code that breaks the build. Run the build/type-check before committing.
3. **Tests must pass** — All existing tests must still pass after the commit.
4. **Commit message matches the change** — Use the conventional commit type that accurately describes what happened.
5. **Commit before context switches** — Before starting a new task, subtask, or switching files for a different concern, commit current work.

### Anti-Patterns

- Accumulating changes across multiple files and committing once at the end of a session
- "WIP" commits with no meaningful message
- Committing generated files alongside source changes
- Waiting for "a good stopping point" — every passing test IS a good stopping point

## Pull Request Workflow

When creating PRs:
1. Analyze full commit history (not just latest commit)
2. Use `git diff [base-branch]...HEAD` to see all changes
3. Draft comprehensive PR summary
4. Include test plan with TODOs
5. Push with `-u` flag if new branch

> For the full development process (planning, TDD, code review) before git operations,
> see [development-workflow.md](./development-workflow.md).
