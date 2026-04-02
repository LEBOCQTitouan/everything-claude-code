---
description: "Install Claude Code workflow templates into the current project's .github/workflows/ directory."
allowed-tools: [Read, Write, Glob, AskUserQuestion]
---

# Scaffold Workflows

Install selected Claude Code GitHub Actions workflow templates into the current project.

## Arguments

`$ARGUMENTS` may contain `--dry-run` to preview without writing files.

## Phase 1: Template Discovery

1. Read `workflow-templates/` directory to discover available templates
2. Build a list of available templates with descriptions:
   - `claude-pr-review.yml` — AI-assisted PR code review
   - `claude-pr-review-fork-safe.yml` — Fork-safe PR review (two-workflow pattern)
   - `claude-issue-triage.yml` — Auto-label and triage new issues
   - `claude-release-notes.yml` — Generate release notes on tag push
   - `claude-ci-linter.yml` — Enforce project conventions on PRs

## Phase 2: Template Selection

Use AskUserQuestion to present available templates:

> Which workflow templates would you like to install?

Options (multiSelect: true):
- PR Review — AI code review on pull requests
- PR Review (Fork-Safe) — Two-workflow pattern for OSS repos with fork contributors
- Issue Triage — Auto-label new issues with Claude Code
- Release Notes — Generate categorized release notes on tag push
- CI Convention Linter — Enforce naming, changelog, and commit conventions

## Phase 3: Installation

For each selected template:

1. Check if `.github/workflows/` directory exists. Create it if not.
2. Check if the target file already exists at `.github/workflows/<template-name>.yml`
   - If it already exists, warn the user and ask whether to overwrite
3. If `--dry-run` is in arguments:
   - Display what would be written: file path and template name
   - Do NOT write any files
   - Exit after displaying the dry-run summary
4. Otherwise, read the template from `workflow-templates/<template-name>.yml` and write it verbatim to `.github/workflows/<template-name>.yml`

## Phase 4: Post-Install

Display a summary of installed templates and remind the user:

> **Installed templates:**
> - `.github/workflows/<name>.yml`
>
> **Next steps:**
> 1. Add `ANTHROPIC_API_KEY` to your repository secrets (Settings → Secrets → Actions)
> 2. Customize behavior via repository variables (optional): `CLAUDE_MODEL`, `CLAUDE_MAX_TURNS`, etc.
> 3. Commit the workflow files to your repository
