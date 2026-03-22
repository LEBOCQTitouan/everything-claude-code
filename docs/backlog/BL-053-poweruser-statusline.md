---
id: BL-053
title: Deploy poweruser statusline via ecc install
status: open
created: 2026-03-22
scope: MEDIUM
target_command: /spec dev
tags: [statusline, ux, install, poweruser, context-window]
---

## Optimized Prompt

Create a custom ECC statusline script deployed by `ecc install` that maximizes information density for power users. The script receives JSON session data on stdin and renders a single-line, color-coded status bar.

**Fields to display (all available):**
- Model name (`model.display_name`) — always visible
- Context window usage (`context_window.used_percentage`) — visual progress bar with color thresholds: green <70%, yellow 70-89%, red 90%+
- Session cost (`cost.total_cost_usd`) — formatted as `$X.XX`
- Session duration (`cost.total_duration_ms`) — formatted as `Xm Ys`
- Lines changed (`cost.total_lines_added` / `total_lines_removed`) — `+N / -N`
- Git branch (via `git branch --show-current --no-optional-locks`)
- Rate limits (`rate_limits.five_hour.used_percentage`) — if available (Pro/Max only), with color coding
- Token counts (`context_window.total_input_tokens` / `total_output_tokens`) — compact format `In:Xk Out:Yk`
- Vim mode (`vim.mode`) — if vim mode enabled
- Worktree/agent name — if in worktree or agent session
- ECC version (from `ecc version` or hardcoded at install time)

**Implementation requirements:**
1. Ship as a shell script at `~/.claude/statusline.sh` (installed by `ecc install`)
2. Register in `~/.claude/settings.json` under `statusLine.command`
3. Handle null fields gracefully (use `// ""` or `// 0` in jq)
4. Cache git branch to `/tmp/ecc-statusline-git-cache` with 5s TTL to avoid perf issues in large repos
5. Keep rendering fast — the script blocks statusline updates
6. Use ANSI color codes for thresholds (context %, rate limits)
7. Truncate gracefully for narrow terminals
8. Add `ecc install` integration: the installer writes the script and updates settings.json
9. Add Rust domain model for statusline config in `ecc-domain` crate (`StatuslineConfig` struct)
10. Add `ecc validate statusline` subcommand to verify the script exists and settings.json points to it

**Research findings (web, 2026-03-22):**
- Community tools: ccstatusline (npx), kamranahmedse/claude-statusline, claude-powerline, Go-based claude-statusline
- ECC should ship its own script rather than depend on third-party npx packages — keeps the install self-contained
- Official `/statusline` slash command can generate scripts from natural language, but ECC should ship a curated, opinionated default
- Scripts must be fast, single-line output, handle nulls, and use `--no-optional-locks` on git commands
- All available JSON fields documented at code.claude.com/docs/en/statusline

## Framework Source

- **Web research**: Community statusline tools, official Claude Code docs, best practices from aihero.dev and GitHub discussions

## Related Backlog Items

- BL-035 (context window monitoring) — both display context %, but BL-035 focuses on warning/compaction triggers while this focuses on visual display. Independent implementations.
