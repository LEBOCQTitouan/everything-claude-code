---
description: "Build + lint + test + code review + architecture review"
allowed-tools: [Bash, Task, Read, Grep, Glob, LS, TodoWrite, TodoRead]
---

# Verify

> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each agent delegation, gate check, and phase transition, tell the user what is happening and why.

Run a single-shot quality gate on the current codebase.

## Arguments

`$ARGUMENTS` supports:
- `quick` — build + lint + test only (skip reviews)
- `full` — all steps (default)
- `--fix` — auto-fix lint/format issues

> **Tracking**: Create a TodoWrite checklist for this command's steps. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Step 1: Build"
- "Step 2: Lint"
- "Step 3: Test Suite"
- "Step 4: Code Review"
- "Step 5: Architecture Review"
- "Step 6: Drift Check"

Mark each item complete as the step finishes.

## Steps

### 1. Build

Auto-detect and run the project build:
- `Cargo.toml` → `cargo build`
- `package.json` → `npm run build` / `pnpm build`
- `go.mod` → `go build ./...`
- `pyproject.toml` → `python -m py_compile`

If build fails → **STOP**, suggest `/build-fix`.

### 2. Lint

Auto-detect and run the linter (append `--fix` if `--fix` was passed):
- `Cargo.toml` → `cargo clippy -- -D warnings`
- `package.json` → `npx eslint .`
- `go.mod` → `golangci-lint run`
- `pyproject.toml` → `ruff check .`

### 3. Test Suite

Auto-detect and run all tests:
- `Cargo.toml` → `cargo test`
- `package.json` → `npm test`
- `go.mod` → `go test ./...`
- `pyproject.toml` → `pytest`

Report pass/fail count.

**If `quick` → stop here.**

### 4. Code Review

> Before dispatching, tell the user which reviewer agents are being launched and why both code-reviewer and arch-reviewer are needed: one checks code quality, the other checks structural integrity.

Invoke the **code-reviewer** agent (allowedTools: [Read, Grep, Glob, Bash]) with `context: "fork"` on `git diff HEAD`. Auto-detect language-specific reviewers:
- Go project → also invoke **go-reviewer** agent (allowedTools: [Read, Grep, Glob, Bash])
- Python project → also invoke **python-reviewer** agent (allowedTools: [Read, Grep, Glob, Bash])

### 5. Architecture Review

Invoke the **arch-reviewer** agent (allowedTools: [Read, Grep, Glob, Bash]) with `context: "fork"` on the full project structure.

### 6. Drift Check (conditional)

If `.claude/workflow/state.json` exists and workflow artifacts are present (state.json and implement-done.md):
- Invoke the **drift-checker** agent (allowedTools: [Read, Grep, Glob, Bash])
- Agent compares implementation against spec — finds unimplemented ACs and scope creep
- Agent writes `.claude/workflow/drift-report.md`
- Report drift level: NONE, LOW, MEDIUM, or HIGH

If no workflow artifacts exist, skip this step silently.

**If `quick` → stop at step 3 (skip reviews, architecture, and drift check).**

## Output

```
VERIFY: [PASS/FAIL]

Build:        [OK/FAIL]
Lint:         [OK/X issues]
Tests:        [X/Y passed]
Code Review:  [PASS/X issues]
Architecture: [PASS/X issues]
Drift:        [NONE/LOW/MEDIUM/HIGH or SKIPPED]

Ready for PR: [YES/NO]
```

## Related Agents

- `code-reviewer` — code quality and security review
- `arch-reviewer` — architecture audit
- `go-reviewer` — Go-specific review (auto-detected)
- `python-reviewer` — Python-specific review (auto-detected)
- `drift-checker` — spec vs implementation drift detection (conditional on workflow artifacts)
