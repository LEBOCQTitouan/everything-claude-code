---
description: "Install Claude Code workflow templates into the current project's .github/workflows/ directory."
allowed-tools: [Read, Write, Glob, AskUserQuestion]
---

# Scaffold Workflows

Install Claude Code GitHub Actions templates.

## Arguments

`--dry-run` — preview without writing.

## Phase 1: Discovery

Read `workflow-templates/` for available templates: PR Review, PR Review (Fork-Safe), Issue Triage, Release Notes, CI Convention Linter.

## Phase 2: Selection

AskUserQuestion (multiSelect: true) with template options.

## Phase 3: Installation

For each selected: create `.github/workflows/` if needed, check for existing (warn/overwrite), `--dry-run` shows what would be written, else copy verbatim.

## Phase 4: Post-Install

Summary: installed files + next steps (add ANTHROPIC_API_KEY secret, customize variables, commit).
