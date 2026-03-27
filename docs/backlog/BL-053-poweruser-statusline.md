---
id: BL-053
title: Poweruser statusline — explicit labels, usage bars, UX overhaul, and install fix
status: open
created: 2026-03-22
updated: 2026-03-26
scope: HIGH
target_command: /spec dev
tags: [statusline, ux, install, poweruser, context-window, rate-limits, filesystem-port]
---

## Optimized Prompt

Overhaul the ECC statusline script for clarity, usability, and completeness. The current script works but is hard to read — fields blend together with no labels, separators, or grouping. This update addresses UX, adds rate limit usage bars, conditionally hides cost for subscribers, and fixes a bug where `ecc install` doesn't set the executable bit.

**Project**: everything-claude-code (Rust, hexagonal architecture, 7 crates)

### Part 1 — UX Overhaul: Explicit, Labeled, Grouped

The current output looks like:
```
Opus [###-----] 42% main $0.05 +50/-10 In:15.0k Out:4.5k 2m 0s ecc 4.2.0
```

The redesigned output must be **self-explanatory** — a user seeing it for the first time should understand every field without documentation.

**Design principles:**
- **Labels**: Every field gets a short label prefix (e.g., `ctx:`, `tok:`, `br:`)
- **Separators**: Use `│` (Unicode box-drawing) between logical groups
- **Grouping**: Fields are organized into semantic groups:
  1. **Identity**: Model name, ECC version
  2. **Context**: Context window bar with label and percentage
  3. **Usage**: Rate limit bars (5h session + 7d weekly) — subscription only
  4. **Activity**: Tokens, lines changed, duration
  5. **Git**: Branch name
  6. **Mode**: Vim mode, worktree/agent name — if active
- **Icons/symbols**: Use Unicode symbols for visual scanning:
  - `◆` or similar for model
  - `█░` for progress bars (instead of `#-`)
  - `⎇` for git branch
  - `+` green / `-` red for lines changed
  - `↑` / `↓` for tokens in/out
- **Color coding**: Not just for thresholds — use dim/muted colors for labels, bright for values

**Target output example** (wide terminal):
```
◆ Opus │ ctx: [████░░░░] 42% │ 5h: [██░░░░░░] 23% 7d: [█░░░░░░░] 12% │ ↑15.0k ↓4.5k │ +50 -10 │ 2m 0s │ ⎇ main │ ecc 4.2.0
```

**Target output example** (narrow terminal — graceful truncation):
```
◆ Opus │ ctx: [████░░░░] 42% │ 5h: 23% 7d: 12% │ ⎇ main
```

### Part 2 — Rate Limit Usage Bars

Display remaining model usage as loading bars for both available quotas:

- **5-hour session quota** (`rate_limits.five_hour.used_percentage`): `5h: [████░░░░] 45%`
- **7-day weekly quota** (`rate_limits.seven_day.used_percentage`): `7d: [██░░░░░░] 18%`

Color thresholds for rate limit bars:
- Green: < 60% used
- Yellow: 60-79% used
- Red: 80%+ used

These fields only appear when `rate_limits` is present in the JSON (Pro/Max subscribers only).

### Part 3 — Conditional Cost Hiding

**Rule**: If `rate_limits` is present in the JSON → user is on an Anthropic subscription (Pro/Max) → **hide session cost** (`$X.XX`). Cost is irrelevant for subscribers and wastes statusline space.

**Rationale**: The statusline JSON does not expose an explicit `accountType` field. The presence of `rate_limits` is the best available heuristic — it's only populated for Pro/Max subscribers. This is the community consensus approach (confirmed via web research 2026-03-26). If upstream adds an explicit field later, the script should be updated to use it.

If `rate_limits` is absent → user is on API billing → **show cost** as `$X.XX`.

### Part 4 — FileSystem Port Fix (executable permissions)

**Bug found 2026-03-26**: `ecc install` writes `statusline-command.sh` via `fs.write()` which creates files with default `0644` permissions. The script needs `0755` to be executable by Claude Code.

**Root cause**: The `FileSystem` port trait in `ecc-ports` has no `set_permissions` method.

**Fix required**:
1. Add `set_permissions(&self, path: &Path, mode: u32) -> Result<()>` to the `FileSystem` trait in `ecc-ports`
2. Implement in `ecc-infra` using `std::fs::set_permissions` with `std::os::unix::fs::PermissionsExt`
3. Add `set_permissions` to `InMemoryFileSystem` in `ecc-test-support` (track permissions in a `HashMap<PathBuf, u32>`)
4. Call `fs.set_permissions(&target_script, 0o755)` after `fs.write()` in `ensure_statusline_in_settings()`
5. Add a test: installed script has executable permissions
6. Update `ecc validate statusline` to check the script is executable (new check)

### Implementation requirements

1. Ship as shell script at `~/.claude/statusline-command.sh` (installed by `ecc install`)
2. Register in `~/.claude/settings.json` under `statusLine.command`
3. Handle null fields gracefully (use `// ""` or `// 0` in jq)
4. Cache git branch to `/tmp/ecc-sl-cache-{PWD_HASH}` with 5s TTL
5. Keep rendering fast (< 50ms) — the script blocks statusline updates
6. Use ANSI color codes for thresholds AND labels (dim for labels, bright for values)
7. Use Unicode box-drawing characters for separators (`│`)
8. Use Unicode block characters for progress bars (`█░`)
9. Truncate gracefully for narrow terminals — priority-based field dropping with labels
10. `ecc install` must `chmod +x` the script after writing it (Part 4)
11. `ecc validate statusline` must verify: script exists, valid shebang, uses jq, settings configured, **script is executable**
12. Rust domain model (`StatuslineConfig`) already exists — update if needed

### Acceptance criteria

- [ ] All statusline fields have explicit labels
- [ ] Fields are grouped with `│` separators
- [ ] Unicode progress bars (`█░`) replace ASCII (`#-`)
- [ ] 5h and 7d rate limit bars displayed for subscribers
- [ ] Cost hidden when `rate_limits` present in JSON
- [ ] Cost shown when `rate_limits` absent (API billing)
- [ ] `FileSystem` port gains `set_permissions` method
- [ ] `InMemoryFileSystem` tracks permissions
- [ ] `ensure_statusline_in_settings` calls `set_permissions(path, 0o755)`
- [ ] `ecc validate statusline` checks executable bit
- [ ] Narrow terminal truncation preserves labels on remaining fields
- [ ] Script renders in < 50ms
- [ ] All existing statusline tests updated for new format
- [ ] New tests for: permission setting, cost hiding logic, rate limit bars

### Scope boundaries — do NOT

- Do not add per-model rate limit breakdowns (not available in JSON upstream)
- Do not make network calls from the script
- Do not depend on external tools beyond `jq`, `git`, and standard Unix utilities
- Do not change the JSON schema Claude Code sends — work with what's available

### Verification steps

1. Run `ecc install` → verify script at `~/.claude/statusline-command.sh` is executable (`-rwxr-xr-x`)
2. Run `ecc validate statusline` → all checks pass including executable check
3. Feed sample JSON with `rate_limits` → verify cost hidden, rate limit bars shown
4. Feed sample JSON without `rate_limits` → verify cost shown, no rate limit bars
5. Test with terminal width 40, 80, 120, 200 → verify graceful truncation
6. Verify all fields have labels and separators
7. Verify progress bars use Unicode block characters

**Research findings (web, 2026-03-22 + 2026-03-26):**
- Community tools: ccstatusline (npx), kamranahmedse/claude-statusline, claude-powerline, Go-based claude-statusline, daniel3303/ClaudeCodeStatusLine
- `rate_limits` presence is the best heuristic for subscription detection — no `accountType` field exists in statusline JSON
- Rate limit headers (`anthropic-ratelimit-unified-5h-utilization`, `anthropic-ratelimit-unified-7d-utilization`) are parsed by Claude Code but exposed in JSON as `rate_limits.five_hour` and `rate_limits.seven_day`
- Multiple open GitHub issues requesting per-model quota data (#28999, #29300, #30784, #34074) — not yet available
- ECC should ship its own script rather than depend on third-party npx packages

## Challenge Log

**Q1 (2026-03-26)**: "Remaining model usage for all models" — per-model breakdown or account-level quotas?
**A1**: Show the two available quotas (5h session + 7d weekly) as loading bars. Per-model not available upstream.

**Q2 (2026-03-26)**: What does "more explicit / better UI/UX" mean concretely?
**A2**: Labels on every field, separators between groups, icons/symbols, color for labels vs values — all upgrades considered.

**Q3 (2026-03-26)**: Best way to detect Anthropic subscription to hide cost?
**A3**: Presence of `rate_limits` in JSON is the best heuristic. Confirmed via web research — no explicit accountType field exists. Community consensus.

**Q4 (2026-03-26)**: Should the FileSystem port fix be separate or included here?
**A4**: Include it — the permission bug directly blocks the statusline from working.

**Original research (2026-03-22)**: Community tools survey, official docs, JSON schema analysis.

## Related Backlog Items

- BL-035 (context window monitoring) — both display context %, but BL-035 focuses on warning/compaction triggers while this focuses on visual display. Independent implementations.
