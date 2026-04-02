---
id: BL-059
title: Auto-commit backlog edits at end of /backlog command
status: "implemented"
created: 2026-03-22
promoted_to: ""
tags: [backlog, git, commit, commands]
scope: LOW
target_command: /spec-dev
---

## Optimized Prompt

Enhance `commands/backlog.md` so that every mutating subcommand (`add`, `promote`,
`archive`) automatically commits its changes to git at the end of the subcommand
flow.

**Project context:** ECC is a Rust workspace plus a collection of Markdown-based
slash commands in `commands/`. The `/backlog` command is defined in
`commands/backlog.md` and is executed by Claude Code as a slash command. It
creates and modifies files under `docs/backlog/`. Currently it leaves those files
uncommitted.

**What changes:**

Add a "Commit" instruction block at the end of each mutating subcommand section
in `commands/backlog.md`:

- **`add`**: after persisting the entry file and updating `BACKLOG.md`, commit with:
  `docs(backlog): add BL-NNN <slug>`
  where `<slug>` is the filename slug of the new entry (e.g.
  `docs(backlog): add BL-059 backlog-autocommit`).

- **`promote <id>`**: after updating the entry file and the index, commit with:
  `docs(backlog): promote BL-NNN`

- **`archive <id>`**: after updating the entry file and the index, commit with:
  `docs(backlog): archive BL-NNN`

The `list` and `match` subcommands are read-only — they must NOT trigger a commit.

**Commit instruction phrasing** (follow ECC convention — see `rules/ecc/development.md`):

> You MUST commit immediately after updating the backlog files. Use `git add
> docs/backlog/` then commit with the message format above.

**Scope boundaries (do NOT do):**

- Do not add commit logic to `list` or `match`.
- Do not change the entry format, ID generation, or any other backlog logic.
- Do not create a new command or hook — this is a direct edit to `commands/backlog.md`.

**Acceptance criteria:**

- Running `/backlog add <idea>` ends with a git commit containing only
  `docs/backlog/BL-NNN-<slug>.md` and `docs/backlog/BACKLOG.md`.
- Running `/backlog promote <id>` ends with a git commit for that entry file and
  the index.
- Running `/backlog archive <id>` ends with a git commit for that entry file and
  the index.
- Commit messages follow the format `docs(backlog): <action> BL-NNN [<slug>]`.
- `list` and `match` produce no git commits.

**Verification steps:**

1. Open a clean branch, run `/backlog add <test idea>`, confirm a commit appears
   with message `docs(backlog): add BL-NNN <slug>` and staged files limited to
   `docs/backlog/`.
2. Run `/backlog promote BL-059`, confirm commit message `docs(backlog): promote BL-059`.
3. Run `/backlog archive BL-059`, confirm commit message `docs(backlog): archive BL-059`.
4. Run `/backlog list` and `/backlog match <text>`, confirm no new commits.

**Run with:** `/spec-dev` — targeted edit to an existing command file

## Original Input

The `/backlog` command creates and modifies files in `docs/backlog/` but never
commits them. Every subcommand that mutates the backlog (`add`, `promote`,
`archive`) should auto-commit its changes at the end. Commit message format:
`docs(backlog): <action> BL-NNN <slug>` — e.g.
`docs(backlog): add BL-058 symlink-config-switching`,
`docs(backlog): promote BL-001`,
`docs(backlog): archive BL-005`.

## Challenge Log

**Q1:** Should all subcommands auto-commit, or only `add`?

**A1:** All mutating subcommands — `add`, `promote`, and `archive`.

**Q2:** What commit message format?

**A2:** `docs(backlog): <action> BL-NNN <slug>` — e.g.
`docs(backlog): add BL-058 symlink-config-switching`,
`docs(backlog): promote BL-001`,
`docs(backlog): archive BL-005`.

## Related Backlog Items

None.
