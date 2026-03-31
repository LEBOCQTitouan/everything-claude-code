---
description: Manage comms pipeline infrastructure — init repo, edit strategies, manage drafts, view calendar.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob, AskUserQuestion, TodoWrite
---

# /comms

Manage comms pipeline infrastructure. Bare `/comms` shows a status overview. Use subcommands for targeted operations.

## Subcommands

| Subcommand | Usage | Description |
|------------|-------|-------------|
| (none) | `/comms` | Show status overview |
| `init` | `/comms init` | Scaffold comms directory structure |
| `strategy` | `/comms strategy <channel>` | View/edit a channel's strategy |
| `drafts` | `/comms drafts [list\|approve\|finalize]` | Manage draft lifecycle |
| `calendar` | `/comms calendar` | View content calendar |

## `/comms` — Status Overview

When run with no subcommand, display a status summary:

1. **Check** for `comms/` directory in the current project. If not found, print:
   ```
   No comms repo found. Run /comms init to get started.
   ```
   Then stop.

2. **Show** the following if comms repo exists:
   - Repo path (absolute)
   - Active channels (directories present under `comms/drafts/`)
   - Draft counts by status: draft, approved, published (parse CALENDAR.md front-matter or filenames)
   - Last generation date (most recent file modification time in `comms/drafts/`)

## `/comms init` — Scaffold Comms Structure

Scaffold the comms infrastructure in the current project.

> **Tracking**: Create a TodoWrite checklist. If TodoWrite is unavailable, proceed without tracking.

TodoWrite items:
- "Check for existing comms directory"
- "Create directory structure"
- "Create default strategy files"
- "Initialize git repo in comms/"
- "Commit initial scaffold"

Steps:

1. **Check** if `comms/` already exists. If it does, ask the user:
   ```
   comms/ already exists. Overwrite (recreate structure) or skip?
   ```
   If skip: stop. If overwrite: proceed.

2. **Create** the directory structure:
   ```
   comms/
     strategies/
       social.md
       blog.md
       devblog.md
       docs-site.md
     drafts/
       social/
       blog/
       devblog/
       docs-site/
     CALENDAR.md
   ```

3. **Write** default strategy files for all 4 channels. Each file should contain:
   - Channel name and audience
   - Tone and format guidelines
   - Constraints (length, hashtags, etc.)
   Use placeholders if no project-specific context is available.

4. **Initialize** a git repo in `comms/` if not already a repo:
   ```bash
   git -C comms/ init
   ```

5. **Commit** the scaffold to the comms repo:
   ```bash
   git -C comms/ add .
   git -C comms/ commit -m "chore: scaffold comms repo"
   ```

## `/comms strategy <channel>` — View/Edit Strategy

View and interactively edit a channel's strategy file.

Valid channels: `social`, `blog`, `devblog`, `docs-site`.

Steps:

1. **Validate** the channel argument. If invalid or missing, report:
   ```
   Unknown channel. Valid channels: social, blog, devblog, docs-site
   ```
   Then stop.

2. **Check** that `comms/strategies/<channel>.md` exists. If not, suggest running `/comms init`.

3. **Display** the strategy file contents.

4. **Ask** the user if they want to edit it. If yes, open the file for editing (present content and accept revised content via AskUserQuestion).

5. **Write** the updated content back to the strategy file.

6. **Commit** changes to the comms repo:
   ```bash
   git -C comms/ add strategies/<channel>.md
   git -C comms/ commit -m "docs(strategy): update <channel> strategy"
   ```

## `/comms drafts [list|approve|finalize]` — Draft Lifecycle

Manage the lifecycle of generated drafts.

### `/comms drafts` or `/comms drafts list`

1. **Scan** all files under `comms/drafts/` recursively.
2. If no files found, print:
   ```
   No drafts found. Run /comms-generate first.
   ```
   Then stop.
3. **Display** a table with columns: Date, Channel, Title, Status, File Path.
   - Date: file creation date or date in filename (YYYY-MM-DD prefix)
   - Status: read from file front-matter (`status:` field) or default to `draft`

### `/comms drafts approve <file>`

Update a draft's status from `draft` to `approved`.

1. **Validate** the file path. If not found under `comms/drafts/`, report an error.
2. **Read** the file and check the current `status:` field in front-matter.
3. **Update** `status: draft` → `status: approved` in the front-matter.
4. **Write** the updated file.
5. **Commit** to comms repo:
   ```bash
   git -C comms/ add drafts/<channel>/<file>
   git -C comms/ commit -m "docs(drafts): approve <file>"
   ```

### `/comms drafts finalize <file>`

Update an approved draft's status to `published` and update CALENDAR.md.

1. **Validate** the file path. If not found, report an error.
2. **Check** `status:` in front-matter. If not `approved`, block with:
   ```
   Must approve first. Run: /comms drafts approve <file>
   ```
3. **Update** `status: approved` → `status: published` in the front-matter.
4. **Update** `comms/CALENDAR.md` — mark the corresponding entry as `published`.
5. **Write** both files.
6. **Commit** to comms repo:
   ```bash
   git -C comms/ add drafts/<channel>/<file> CALENDAR.md
   git -C comms/ commit -m "docs(drafts): finalize <file>"
   ```

## `/comms calendar` — View Content Calendar

Display the content calendar.

1. **Check** for `comms/CALENDAR.md`. If not found, print:
   ```
   No calendar found. Run /comms init to get started.
   ```
   Then stop.

2. **Read** and display `comms/CALENDAR.md` entries grouped by date.

## Commit Rules

- All commits go to the **comms repo** (`git -C comms/ ...`), never to the main project repo.
- Commit message format: `docs(comms): <description>` or `chore(comms): <description>`.
- Read-only subcommands (`list`, `calendar`, bare status) MUST NOT commit.
- Mutating subcommands (`init`, `strategy`, `approve`, `finalize`) MUST commit immediately after completing.
