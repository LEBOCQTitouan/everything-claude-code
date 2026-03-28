---
id: BL-063
title: Create /commit slash command
status: implemented
created: 2026-03-26
tags: [git, workflow, commit, conventional-commits, slash-command]
scope: MEDIUM
target_command: /spec-dev
---

## Optimized Prompt

Use `/spec-dev` to create a `/commit` slash command for the ECC project.

**Context:** Developers using ECC must manually ask Claude to commit after changes, breaking workflow rhythm. A `/commit` command should generate commit messages automatically from the diff, enforce atomic-commit and conventional-commits rules, stage files intelligently, and gate on build/test results before committing.

**Tech stack:** ECC slash commands (Markdown + YAML frontmatter in `commands/`). No Rust changes required. One new command file, potentially one new skill.

**Acceptance criteria:**

1. `/commit` command file created at `commands/commit.md` with proper frontmatter (`allowed-tools`, model, description).
2. Auto-generates a conventional commit message from `git diff --staged` (or the full diff if nothing staged), following the format `<type>: <description>` with optional body.
3. Intelligently stages files: uses Claude's session action history as primary signal; falls back to `git status` with hypothesis-based grouping when session context is unavailable.
4. Enforces atomic commit rules: if the staged diff spans multiple unrelated concerns, surfaces a warning and asks the user to split or confirm.
5. Enforces conventional commits format: type must be one of `feat | fix | refactor | docs | test | chore | perf | ci`.
6. Runs a build + test pre-flight (`cargo test` / `npm test` / detected test command) before committing.
7. If pre-flight fails: blocks the commit, surfaces the failures, and presents the user with two options — fix first, or force-proceed.
8. If nothing to commit (`git status` clean): informs the user and stops without error.
9. Commit message is shown to the user for confirmation before executing `git commit`.

**Scope boundaries — do NOT:**
- Implement push or PR creation (that belongs to a separate `/pr` command)
- Modify the Rust CLI or any `.rs` files
- Change existing command files
- Implement branch management

**Verification steps:**

1. `/commit` with a clean working tree → "Nothing to commit" message, no side effects.
2. `/commit` with staged changes → message generated, pre-flight runs, user confirms, commit executes.
3. `/commit` with a failing test → commit blocked, failure surfaced, user offered fix-first or force-proceed.
4. Generated message follows `<type>: <description>` format with valid type token.
5. Command frontmatter passes `ecc validate commands`.

## Original Input

The friction is having to manually ask Claude to commit after changes. The user wants a `/commit` slash command that auto-generates commit messages from the diff following conventional commits format, enforces atomic commits and git workflow rules, stages files intelligently based on session context, handles nothing-to-commit gracefully, and blocks on pre-flight failures while letting the user choose to fix first or force-proceed.

## Challenge Log

Mode: backlog-mode (3 stages, max 2 questions per stage)

**Stage 1: Clarity**

Q1: What specific friction are you solving?
A: Having to manually ask Claude to commit after changes. Wants a `/commit` command.

Q2: Should it generate the commit message automatically or prompt the user to write one?
A: Auto-generate from the diff, following conventional commits format.

**Stage 2: Assumptions**

Q3: Should it also enforce git workflow rules (conventional commits, atomic commits)?
A: Yes — enforce both conventional commits format and atomic commits.

Q4: How should it decide which files to stage?
A: Auto-stage intelligently — based on Claude's session actions as primary signal, hypothesis-based for unknown files.

**Stage 3: Edge Cases**

Q5: What should happen if there's nothing to commit?
A: Inform the user and stop gracefully.

Q6: What if the build or tests fail before committing?
A: Block the commit, surface failures, then let the user choose: fix first or force-proceed.

## Related Backlog Items

- BL-059: Auto-commit backlog edits at end of /backlog command — adjacent (auto-commit pattern) but scoped to backlog only; this entry is the general-purpose command BL-059 could eventually delegate to.
