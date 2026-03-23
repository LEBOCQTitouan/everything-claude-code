# 0016. Directory-Level Symlinks for Dev Profile Switching

Date: 2026-03-23

## Status

Accepted

## Context

BL-058 adds `ecc dev switch dev|default` for instant config switching between release-installed copies and a local ECC checkout. The switching mechanism uses Unix symlinks to point `~/.claude/` asset directories to the developer's working copy in `ECC_ROOT`.

Two approaches were considered: (1) per-file symlinks, where each individual agent, command, skill, and rule file gets its own symlink, and (2) directory-level symlinks, where entire directories (`agents/`, `commands/`, `skills/`, `rules/`) are symlinked as units.

ECC has single ownership over all files in these directories — there are no user-created files mixed in (user customizations go in project-level directories, not global `~/.claude/`).

## Decision

Use directory-level symlinks for the four managed directories: `agents/`, `commands/`, `skills/`, `rules/`.

The `hooks/` directory is excluded because ECC hooks are registered in `settings.json` entries, not as files in a `~/.claude/hooks/` directory. Symlinking a hooks directory would have no effect on hook execution.

## Consequences

**Positive:**

- Simpler implementation: 4 symlink operations instead of potentially hundreds
- Instant switching: no file enumeration or manifest walking needed
- Any new files added to ECC source directories are immediately visible in dev mode
- Follows the GNU Stow pattern (industry-standard symlink farm manager)

**Negative:**

- Cannot have per-file mixed ownership (some files from ECC, some user-created in the same directory). This is acceptable because user customizations use project-level directories.
- `dev_off` and `clean_from_manifest` must be symlink-aware to avoid traversing into `ECC_ROOT` and deleting source files. Mitigated by `is_symlink` guard using `remove_file` (not `remove_dir_all`) on symlinked directories.
- Profile detection is all-or-nothing per directory: if one directory is symlinked and others are copied, status reports "Mixed (inconsistent)".
