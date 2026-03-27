---
id: BL-058
title: Symlink-based instant config switching for ecc dev
status: implemented
created: 2026-03-22
promoted_to: ""
tags: [ecc-dev, symlink, install, cli, config-switching]
scope: MEDIUM
target_command: /spec-dev
---

## Optimized Prompt

Extend the `ecc dev` subcommand with a new `switch` action that replaces the current
full clean + reinstall cycle with zero-copy, zero-file-edit instant switching via Unix
symlinks.

**Project context:** Rust workspace with hexagonal architecture (`ecc-domain` → `ecc-ports`
← `ecc-infra` → `ecc-app` → `ecc-cli`). All I/O is abstracted behind port traits in
`ecc-ports`. The `ecc dev on|off|status` implementation lives in `crates/ecc-app/src/dev.rs`
and `crates/ecc-cli/src/commands/dev.rs`. Install infrastructure is in `crates/ecc-app/src/install.rs`.

**New command surface:**

```
ecc dev switch default   # point ~/.claude/ assets → release-installed copies
ecc dev switch dev       # point ~/.claude/ assets → local ECC_ROOT working copy (symlinks)
ecc dev status           # update to show active profile + symlink indicator
```

**What changes:**

1. **Domain** (`ecc-domain`): add `DevProfile` enum (`Default | Dev`) and a
   `SymlinkPlan` value type (list of `(target_path, link_path)` pairs). Pure logic,
   no I/O.

2. **Port** (`ecc-ports`): extend `FileSystem` trait with `create_symlink(target, link)`
   and `read_symlink(link) -> Option<PathBuf>`. Implement in `ecc-infra/src/os_fs.rs`.
   Add in-memory stub to `ecc-test-support`.

3. **App** (`ecc-app`): add `dev_switch(profile, ecc_root, claude_dir, dry_run)` use case
   that builds a `SymlinkPlan` and applies it atomically (remove old link or file, create
   new symlink, rollback on error). Reuse existing `read_manifest` to discover which paths
   to manage.

4. **CLI** (`ecc-cli`): add `DevAction::Switch { profile: DevProfile, dry_run: bool }` to
   the `DevArgs` subcommand enum and wire to `dev_switch`.

5. **`ecc dev status`**: detect whether installed files are regular files or symlinks;
   show `profile: dev (symlinked)` vs `profile: default (copied)`.

**Scope boundaries (do NOT do):**

- Do not remove or modify the existing `on` / `off` / `status` actions — they remain
  for users who prefer full reinstall semantics.
- Do not add symlink support to `ecc install` (global install path stays copy-based).
- Do not handle Windows junction points — Unix symlinks only for v1.
- Do not add automatic profile detection on shell startup.

**Acceptance criteria:**

- `ecc dev switch dev` creates symlinks from `~/.claude/agents/`, `~/.claude/commands/`,
  `~/.claude/skills/`, `~/.claude/rules/`, `~/.claude/hooks/` pointing into `ECC_ROOT`.
- `ecc dev switch default` removes symlinks and restores copied files (or errors clearly
  if no release manifest is found).
- `ecc dev switch dev --dry-run` prints the planned symlink operations without applying them.
- `ecc dev status` shows the active profile and symlink indicator.
- All new code is covered by unit tests using `InMemoryFileSystem` (with symlink support added).
- `cargo test` passes (all ~1185+ tests green).
- `cargo clippy -- -D warnings` passes with zero new warnings.
- `ecc-domain` crate has zero I/O imports (domain hook enforcement still passes).

**Verification steps:**

1. `cargo test` — full suite passes.
2. `cargo clippy -- -D warnings` — zero warnings.
3. `cargo build --release` — binary builds cleanly.
4. Manual smoke: `ECC_ROOT=<repo> ecc dev switch dev` then `ls -la ~/.claude/agents/` to
   confirm symlinks. Then `ecc dev switch default` and confirm symlinks removed.
5. `ecc dev status` output shows `profile:` line in both states.

**Run with:** `/spec-dev` — new feature on existing CLI subcommand

## Original Input

The ECC project has `ecc dev on|off|status` that toggles between user mode and dev mode.
Currently `ecc dev on` does a full clean + reinstall of assets from the ECC source directory
into `~/.claude/`. The user wants zero-copy, zero-file-edit instant switching using Unix
symlinks. Two configs: "ECC user mode" = installed `~/.claude/` assets from release, vs.
"ECC dev mode" = local working copy from repo checkout (`ECC_ROOT`). Problem: current
switching is slow (full clean + reinstall). Symlinks enable zero-copy instant switching.
Command surface: `ecc dev switch default|dev` — extend the existing `ecc dev` subcommand.

## Challenge Log

**Q1:** What exactly are the "two configs" — are they two different sets of content, or just
two different sources for the same content structure?

**A1:** Two sources for the same content structure: "ECC user mode" = installed `~/.claude/`
assets from release; "ECC dev mode" = local working copy from repo checkout (`ECC_ROOT`).

**Q2:** What is the core problem with the current `ecc dev on`?

**A2:** It is slow — full clean + reinstall every time. Symlinks enable zero-copy instant
switching with no file writes.

**Q3:** How should the command surface look?

**A3:** `ecc dev switch default|dev` — extend the existing `ecc dev` subcommand rather than
adding a new top-level command.

## Related Backlog Items

- [BL-053](BL-053-deploy-poweruser-statusline-via-ecc-install.md) — poweruser statusline
  via `ecc install` (open) — both touch the install/dev infrastructure surface
