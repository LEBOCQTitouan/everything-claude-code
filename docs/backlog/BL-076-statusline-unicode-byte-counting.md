---
id: BL-076
title: "Statusline Unicode byte-counting bug hides rate limit segments"
scope: LOW
target: /spec-fix
status: implemented
created: 2026-03-27
related: [BL-053]
---

# BL-076 — Statusline Unicode byte-counting bug hides rate limit segments

## Problem

The statusline script (`~/.claude/statusline-command.sh`) uses `${#stripped}` (bash string length) to measure output width before deciding which segments to include. In bash, `${#var}` counts **bytes**, not visible characters. Unicode characters used in the statusline (◆, ⎇, █, ░, ↑, ↓) are 3 bytes each, causing the script to massively overcount the visible width.

**Impact:** The `build_output` function thinks the line is ~2x wider than it actually is, so it stops adding segments early. Rate limit bars (`5h:`, `7d:`) and other lower-priority segments (duration, cost, ecc version) are silently dropped even when terminal width is sufficient.

**Evidence:** With actual JSON from Claude Code containing valid `rate_limits.five_hour.used_percentage: 26` and `rate_limits.seven_day.used_percentage: 47`, the rate limit segments never appear. The stripped output measures 107 bytes but only ~50 visible characters.

## Root Cause

`statusline-command.sh` lines 198, 224, 229: `${#stripped}` / `${#STRIPPED}` count bytes in bash when the string contains multi-byte UTF-8 characters.

## Ready-to-Paste Prompt

```
/spec-fix

Bug: statusline-command.sh drops rate limit and other segments due to Unicode byte-counting.

Root cause: `${#stripped}` in bash counts bytes, not characters. Unicode chars (◆, ⎇, █, ░, ↑, ↓)
are 3 bytes each, so the script thinks output is ~2x wider than visible width.

Affected lines: 198 (`${#stripped}` in build_output loop), 224 and 229 (`${#STRIPPED}` in final
width checks).

Fix approach: Replace `${#var}` with a pure-bash character-length method — no subprocess spawning
(no wc -m). Consider using `printf '%s' "$var" | wc -m` only if no pure-bash solution exists.
Prefer LC_ALL-aware approach or parameter expansion trick.

Test: verify that with COLUMNS=120 and rate_limits present in JSON, the 5h: and 7d: bars appear
in output. Also verify narrow terminal (<60 cols) still degrades gracefully.
```
