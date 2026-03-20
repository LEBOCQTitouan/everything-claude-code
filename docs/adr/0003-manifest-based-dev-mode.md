# ADR-0003: Manifest-Based Dev Mode Switching

## Status

Accepted

## Context

ECC installs agents, commands, skills, rules, and hooks into `~/.claude/`. Developers need to quickly toggle between "ECC-enhanced" and "vanilla Claude" config without manually deleting and reinstalling files. Three approaches were considered:

1. **Symlink swapping** — Point `~/.claude/agents` etc. at an ECC directory or an empty directory. Requires all artifact dirs to be symlinks, which conflicts with user-added files in those directories.

2. **Profile directories** — Maintain `~/.claude/profiles/ecc/` and `~/.claude/profiles/vanilla/` and swap the active profile. Adds significant complexity (backup/restore, migration) for a simple two-state toggle.

3. **Manifest-based clean + reinstall** — Use the existing `.ecc-manifest.json` to surgically remove only ECC-managed files (`dev off`), and run `install --clean --force` to restore them (`dev on`).

## Decision

Use manifest-based clean for `dev off` and idempotent force-reinstall for `dev on`:

- `ecc dev on` calls `install_global()` with `{ force: true, interactive: false, clean: true }`
- `ecc dev off` reads the manifest and calls `clean_from_manifest()` to remove only tracked artifacts
- `ecc dev status` reads the manifest to report the current state

No new domain logic is needed — the entire feature is app-layer orchestration of existing building blocks.

## Consequences

- **Easier**: Single command to switch configs; no manual file management
- **Easier**: Surgical removal preserves user hooks and non-ECC files in settings.json
- **Easier**: Idempotent `dev on` ensures consistent state regardless of starting point
- **Harder**: Named profiles (e.g., "minimal ECC" vs "full ECC") would require a different approach — out of scope for now
- **Risk**: If the manifest is corrupted or deleted, `dev off` cannot clean up — user falls back to `ecc install --clean-all`
