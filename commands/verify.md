---
description: "Build + lint + test + code review + architecture review"
---

# Verify

Run a single-shot quality gate on the current codebase.

## Arguments

`$ARGUMENTS` supports:
- `quick` — build + lint + test only (skip reviews)
- `full` — all steps (default)
- `--fix` — auto-fix lint/format issues

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

Invoke the **code-reviewer** agent on `git diff HEAD`. Auto-detect language-specific reviewers:
- Go project → also invoke **go-reviewer** agent
- Python project → also invoke **python-reviewer** agent

### 5. Architecture Review

Invoke the **arch-reviewer** agent on the full project structure.

### 6. Drift Check (conditional)

If `.claude/workflow/state.json` exists and workflow artifacts are present (state.json and implement-done.md):
- Invoke the **drift-checker** agent
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
