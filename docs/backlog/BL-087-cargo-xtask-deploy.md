---
id: BL-087
title: Cargo xtask deploy — full local machine setup
status: implemented
created: 2026-03-28
tags: [deploy, xtask, install, shell-integration, completions]
scope: HIGH
target_command: /spec-dev
---

## Optimized Prompt

Use `/spec-dev` to create a `cargo xtask deploy` command for the ECC project.

**Context:** Deploying ECC to the local machine currently requires multiple manual steps: `cargo install --path crates/ecc-cli`, `cargo install --path crates/ecc-workflow`, `ecc install`, then manual shell RC edits for PATH, completions, and statusline. A single `cargo xtask deploy` should automate the entire process.

**Tech stack:** Cargo xtask pattern (new `xtask` crate in workspace with `[[bin]]` target). Rust for the orchestration logic. Shell RC detection and editing.

**Acceptance criteria:**

1. New `xtask` crate added to workspace with `cargo xtask deploy` entry point.
2. Builds both `ecc` and `ecc-workflow` in release mode (`cargo build --release`).
3. Installs both binaries to `~/.cargo/bin/` (or detected cargo install path).
4. Runs `ecc install` to sync ECC config (agents, skills, hooks, rules) to `~/.claude/`.
5. Generates shell completions for the detected shell (zsh/bash/fish) via `ecc completion <shell>`.
6. Installs completions to the correct platform-specific directory (e.g., `~/.zfunc/` for zsh).
7. Detects the user's shell RC file (`.zshrc`, `.bashrc`, `.bash_profile`) and ensures:
   - `~/.cargo/bin` is in PATH (add if missing, skip if present)
   - Completion source line is present (add if missing, skip if present)
   - Statusline prompt integration is present (add if missing, skip if present)
8. All RC edits are idempotent — running deploy twice produces no duplicates.
9. Dry-run mode: `cargo xtask deploy --dry-run` shows what would be done without modifying anything.
10. Summary output: lists all actions taken (installed, skipped, added to RC).

**Scope boundaries — do NOT:**
- Deploy to remote machines (local only)
- Modify the Rust CLI or existing crate code
- Add new dependencies to ecc-cli or ecc-workflow
- Support Windows (POSIX shells only for v1)
- Auto-update (this is a one-shot deploy, not a daemon)

**Verification steps:**

1. `cargo xtask deploy --dry-run` → shows all steps without side effects.
2. `cargo xtask deploy` on a clean machine → both binaries installed, ecc install run, completions installed, RC updated.
3. `cargo xtask deploy` run again → all steps report "already present" (idempotent).
4. `which ecc && which ecc-workflow` → both resolve to `~/.cargo/bin/`.
5. New shell session → completions work, statusline shows.

## Original Input

"I want a 'cargo deploy' command that automatically deploys ECC to the current machine" — full setup including shell integration (RC files, completions, statusline). Cargo xtask approach (Rust-native). HIGH scope due to shell RC editing complexity.

## Challenge Log

Mode: backlog-mode (5 stages, HIGH scope)

**Stage 1: Clarity**
Q1: What does "deploy to current machine" mean?
A1: Full setup including shell integration (RC files, completions, statusline config).

**Stage 2: Assumptions**
Q2: Shell script or cargo xtask?
A2: Cargo xtask — Rust-native, integrates with build system.

**Stage 3: Scope**
Q3: Scope estimate?
A3: HIGH — shell RC editing is tricky (detect zsh/bash, avoid duplicates, PATH management).
