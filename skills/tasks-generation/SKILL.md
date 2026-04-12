---
name: tasks-generation
description: tasks.md generation logic — session-independent task tracker format and status trail conventions
origin: ECC
---

# Tasks Generation

Persist a session-independent task tracker alongside the spec and design artifacts in `docs/specs/YYYY-MM-DD-<slug>/tasks.md`.

## Generation

1. Read `artifacts.spec_path` from `state.json`. If `spec_path` is null or the spec directory is not available, emit a warning: "Spec directory not available. tasks.md generation skipped." and skip.
2. Derive the spec directory from `spec_path` (e.g., `docs/specs/2026-03-21-my-feature/`)
3. Write `tasks.md` in that directory using this format:

```markdown
# Tasks: <feature title>

## Pass Conditions

- [ ] PC-001: <description> | `<command>` | pending@<ISO 8601 timestamp>
- [ ] PC-002: <description> | `<command>` | pending@<ISO 8601 timestamp>
...

## Post-TDD

- [ ] E2E tests | pending@<ISO 8601 timestamp>
- [ ] Code review | pending@<ISO 8601 timestamp>
- [ ] Doc updates | pending@<ISO 8601 timestamp>
- [ ] Write implement-done.md | pending@<ISO 8601 timestamp>
```

4. Store `artifacts.tasks_path` in state.json: run `!ecc-workflow transition implement --artifact implement --path <tasks_path>`
5. Commit: `docs: write tasks.md for <feature>`

## Status Trail

Status updates during the TDD loop append to each line's trail:
- Dispatch: append `| red@<ISO 8601 timestamp>`
- Subagent success: append `| green@<ISO 8601 timestamp>`
- Self-evaluation (if triggered): append `| eval@<ISO 8601 timestamp> <AC/REG/ACH verdict>`
- Regression verification passes: append `| done@<ISO 8601 timestamp>` and mark `[x]`
- Failure: append `| failed@<ISO 8601 timestamp> ERROR: <summary>`
