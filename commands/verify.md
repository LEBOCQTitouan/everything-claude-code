---
description: "Build + lint + test + code review + architecture review"
allowed-tools: [Bash, Task, Read, Grep, Glob, LS, TodoWrite, TodoRead]
---

# Verify

> Narrate per `skills/narrative-conventions/SKILL.md`.

Single-shot quality gate.

## Arguments

- `quick` — build + lint + test only
- `full` — all steps (default)
- `--fix` — auto-fix lint/format
- `--mutation` — include diff-scoped mutation testing

> **Tracking**: TodoWrite checklist. If unavailable, proceed without tracking.

## Steps

### 1. Build
Auto-detect (Cargo.toml→cargo build, package.json→npm build, go.mod→go build, pyproject→py_compile). Fail → STOP, suggest `/build-fix`.

### 2. Lint
Auto-detect + `--fix` if passed.

### 3. Test Suite
Auto-detect. Report pass/fail. **If `quick` → stop here.**

### 4. Code Review
`code-reviewer` (context: "fork") on `git diff HEAD`. Auto-detect language reviewers (go-reviewer, python-reviewer).

### 5. Architecture Review
`arch-reviewer` (context: "fork") on full project.

### 6. Mutation Testing (if `--mutation`)
Run `cargo xtask mutants --in-diff`. Informational only — does not affect verdict.

### 7. Drift Check (conditional)
If workflow artifacts exist: `drift-checker` compares implementation vs spec. Reports NONE/LOW/MEDIUM/HIGH. Skip if no artifacts.

## Output

```
VERIFY: [PASS/FAIL]
Build/Lint/Tests/Code Review/Architecture/Mutation/Drift: status
Ready for PR: [YES/NO]
```

## Related Agents

- `code-reviewer`, `arch-reviewer`, `go-reviewer`, `python-reviewer`, `drift-checker`
