---
id: BL-052
title: Replace .claude/hooks shell scripts with compiled Rust binaries
status: open
created: 2026-03-22
promoted_to: ""
tags: [rust, hooks, cross-platform, portability, workspace]
scope: HIGH
target_command: /spec
---

## Optimized Prompt

```
/spec

Project: everything-claude-code (Rust Cargo workspace, 7 crates, hexagonal architecture)

Task: Replace every shell script in .claude/hooks/ with a compiled Rust binary of equivalent
behaviour, so the hook layer works on any OS without a POSIX shell dependency.

## Current state

The following 11 shell scripts live in .claude/hooks/ and are referenced by hooks.json:
  tdd-enforcement.sh
  scope-check.sh
  doc-level-check.sh
  stop-gate.sh
  doc-enforcement.sh
  pass-condition-check.sh
  e2e-boundary-check.sh
  workflow-init.sh
  phase-transition.sh
  memory-writer.sh
  phase-gate.sh

Each script is a standalone executable invoked by the Claude Code hooks runtime.
Claude Code hooks pass context via stdin (JSON) and read exit code + stdout.

## Design decisions already made

1. Source code lives in a new crate inside the existing Cargo workspace.
   Suggested name: ecc-hooks (or nest under src/hooks/).
2. Each hook becomes its own [[bin]] target inside that crate so cargo build
   produces one binary per hook.
3. Compiled binaries are placed in .claude/hooks/ with the same names as the
   current .sh files (minus the .sh extension, or with a .exe suffix on Windows).
   The .sh files are removed after successful compilation and validation.
4. OPEN DESIGN QUESTION — build trigger: when and how are these binaries built?
   Options to evaluate during /spec:
     A. Bundled in cargo build --release (always built with the workspace)
     B. Built by ecc install (post-install step)
     C. Built on-demand by a new ecc hooks build subcommand
     D. Pre-built and committed as binaries (not preferred — binary blobs in git)
   The spec must choose one option and justify it.

## Scope boundaries

In scope:
- All 11 scripts under .claude/hooks/
- New Cargo crate for hook binaries within the existing workspace
- Choosing and implementing the build trigger (option A, B, or C)
- Updating hooks.json to reference binary names instead of .sh paths
- Updating install.sh / ecc install to compile or distribute the binaries
- Tests: at minimum a smoke test per binary (exit 0 on valid input)
- Updating CLAUDE.md test count after adding tests

Out of scope:
- Scripts in skills/, scripts/, statusline/, bin/ — not part of the hook runtime
- Rewriting any hook logic beyond a faithful port to Rust
- Adding new hook functionality
- Modifying the hooks runtime or hooks.json schema

## Acceptance criteria

- [ ] All 11 .sh hook files have an equivalent Rust binary that passes its existing
      test suite (tests/hooks/)
- [ ] Binaries run on macOS, Linux, and Windows without a POSIX shell
- [ ] Build trigger decision is documented in an ADR under docs/adr/
- [ ] hooks.json updated to reference new binary paths
- [ ] cargo test passes (zero regressions)
- [ ] cargo clippy -- -D warnings passes
- [ ] CLAUDE.md test count updated

## Verification steps

1. cargo build --release — all binaries produced in target/release/
2. Run each binary manually with a sample stdin payload and verify exit code
3. Run tests/hooks/ test suite against the new binaries
4. Run cargo clippy -- -D warnings
5. Confirm .sh files removed and hooks.json updated
6. Smoke-test a full ECC workflow (e.g., ECC_WORKFLOW_BYPASS=0 claude) to verify
   hooks fire correctly
```

## Original Input

"I want to replace every sh element used by every wf in this project with equivalent rust code
(compiled) so that it is usable on as much os as possible. I dont know when those elements
should be built."

## Challenge Log

**Q1:** Are you asking to (A) move shell logic into the ecc hook subcommand, (B) replace with
standalone compiled Rust binaries one per hook, or (C) something else?
**A1:** B — standalone compiled Rust binaries, one per hook.

**Q2:** Where should the compiled binaries live and how should they be built?
(A) new crate in workspace, (B) separate workspace, (C) no preference?
**A2:** Code lives in a crate (within the existing workspace). Compiled binaries get inserted in
the same space as the current .sh files (.claude/hooks/).

**Open question captured in entry:** The user does not yet know WHEN these should be built.
The spec must evaluate and decide between: bundled in cargo build, ecc install post-step,
or a new ecc hooks build subcommand.

## Related Backlog Items

None directly. Indirectly touches any entry that references .claude/hooks/ scripts
(BL-046 phase-gate hook, BL-041 workflow-init.sh).
