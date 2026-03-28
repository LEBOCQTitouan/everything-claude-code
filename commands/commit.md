---
description: "Create a git commit with auto-generated conventional message, atomic-commit enforcement, and pre-flight gates."
allowed-tools: [Bash, Read, Grep, Glob, AskUserQuestion]
---

# Commit

> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each step, tell the user what is happening and why.

Create a git commit with intelligent staging, conventional commit message generation, atomic-commit enforcement, and build/test pre-flight validation.

## Arguments

`$ARGUMENTS` supports:
- An optional commit message string used as the initial message (user can still edit before committing)
- If no arguments are provided, the message is auto-generated from the diff

## Phase 1: Workflow State Check

Read `.claude/workflow/state.json` if it exists:

1. If state.json exists AND `phase` is `"implement"`:
   - Display a warning: "An /implement workflow is active. Commits during implementation are managed by the workflow's TDD loop. Proceeding may break the commit trail."
   - Use `AskUserQuestion` with options: `["Proceed anyway", "Cancel"]`
   - If "Cancel": stop immediately
2. If state.json does not exist OR `phase` is not `"implement"`: proceed without warning

## Phase 2: Working Tree Analysis

Run `git status --porcelain` to analyze the working tree:

1. **Nothing to commit**: If `git status --porcelain` returns empty output, display "Nothing to commit — working tree clean" and stop without error. Treat trees with only `.gitignore`-covered untracked files as clean.

2. **Merge conflict detection**: If `git status --porcelain` shows any paths with `UU`, `AA`, `DD`, or other merge conflict markers, display the conflicting files and block the commit: "Unresolved merge conflicts detected. Resolve conflicts before committing."

3. **Staging analysis**: Determine what to stage:
   - If files are already staged (`git diff --cached --name-only` is non-empty): respect existing staging — do not re-stage or unstage anything
   - If no files are staged: propose staging using session action history (files edited/created in the current conversation) as the primary signal. Fall back to `git status` analysis when session context is unavailable (e.g., after context compaction). Use `git status` as enrichment to catch untracked files the session may have missed
   - Use `AskUserQuestion` to present the proposed staging for user confirmation. The user can accept, adjust, or cancel

4. After staging is confirmed, run `git add` for the confirmed files

## Phase 3: Atomic Commit Enforcement

Analyze the staged diff (`git diff --cached`) for concern separation:

1. **Single concern**: If all changes relate to a single logical concern (e.g., one feature, one bug fix, one refactor), proceed without warning
2. **Multiple unrelated concerns**: If the diff spans multiple unrelated concerns (e.g., a bug fix mixed with a refactor, or changes to independent modules with no logical connection), warn the user:
   - Identify and list the separate concerns detected
   - Suggest splitting into multiple atomic commits
   - Use `AskUserQuestion` with options: `["Proceed with single commit", "Split into multiple commits"]`
   - If "Split": guide the user through unstaging files for each concern using `git reset HEAD <files>`, then re-run from Phase 2 for each subset

> Note: Concern detection is heuristic-based (Claude's semantic analysis of the diff). It is best-effort, not deterministic.

## Phase 4: Conventional Commit Message Generation

Generate a commit message following the conventional commits format:

1. **Type selection**: Analyze the staged diff to determine the commit type. Type MUST be one of: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`

2. **Scope inference**: If all changed files are within a single directory subtree, infer a scope from that directory (e.g., changes in `crates/ecc-domain/` → scope `domain`). If changes span multiple unrelated directories, omit the scope to avoid ambiguity

3. **Message format**: `<type>[(<scope>)]: <description>` with an optional body for larger changes. The description should be concise (under 72 characters), lowercase, imperative mood

4. **User override via arguments**: If `$ARGUMENTS` contains a commit message string, use it as the initial message instead of auto-generating

5. **Confirmation**: Display the generated (or argument-provided) message and use `AskUserQuestion` with options: `["Accept", "Edit", "Cancel"]`
   - If "Accept": proceed to pre-flight
   - If "Edit": ask the user for their revised message, then confirm again
   - If "Cancel": abort the commit

## Phase 5: Build and Test Pre-Flight

Run build and test commands before committing. Pre-flight failure always blocks the commit — there is no force-proceed option.

### Toolchain Detection

1. Read `.claude/workflow/state.json` — if `toolchain.build` and `toolchain.test` are set (non-null), use those commands
2. If state.json is unavailable or toolchain fields are null, detect from project files:
   - `Cargo.toml` present → `cargo build` + `cargo test`
   - `package.json` present → `npm run build` (if script exists) + `npm test`
   - `go.mod` present → `go build ./...` + `go test ./...`
   - `pyproject.toml` or `setup.py` present → `python -m pytest`
   - No recognized project file → skip pre-flight with warning: "No build/test toolchain detected. Skipping pre-flight."

### Execution

1. Run the build command. If it fails → display errors, block the commit, and tell the user: "Build failed. Fix the errors before committing."
2. Run the test command. If it fails → display failures, block the commit, and tell the user: "Tests failed. Fix the failing tests before committing."
3. If both pass → proceed to commit execution

## Phase 6: Execute Commit

Run `git commit` with the confirmed message:

```bash
git commit -m "<confirmed message>"
```

If the commit includes a body, use a HEREDOC:

```bash
git commit -m "$(cat <<'EOF'
<type>[(<scope>)]: <description>

<body>
EOF
)"
```

Display the commit hash and summary after successful execution.

## Phase 7: Summary

Display a brief summary:

> **Committed:** `<short hash>` — `<commit message>`
> **Files:** `<N>` files changed
> **Pre-flight:** build ✅ tests ✅

Then stop. Do NOT push or create a PR.
