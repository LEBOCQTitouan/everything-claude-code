---
id: BL-154
title: "Widen ecc validate commands rule to catch backtick-embedded !$ARGUMENTS inline-code patterns"
scope: LOW
target: "direct edit"
status: open
created: "2026-04-17"
source: "docs/specs/2026-04-17-spec-command-shell-escaping adversary round 3 finding #2"
ring: trial
tags: [validation, defense-in-depth, shell-injection, commands]
---

## Context

The shell-eval injection fix (spec 2026-04-17) added `ecc validate commands` rule with pinned regex `^[[:space:]]*!.*\$ARGUMENTS`. This catches line-start `!`-prefix patterns but **misses** backtick-embedded `!`-prefix occurrences like:

```
1. Run: `!ecc-workflow worktree-name dev "$ARGUMENTS"` — capture the output name
```

The pre-fix codebase had 7 offending lines total; the narrow regex only flagged 3 of them at the `!`-prefix-at-column-0 sites. The fix's template rewrite REMOVED all 7 occurrences, so post-fix the narrow regex sees zero violations and AC-001.1a/1a is satisfied as written. However, future template authors could re-introduce the backtick-embedded pattern and the validate rule would not catch it.

The spec's `VALIDATE_REGEX` is pinned in three places (Definitions, AC-001.1a, AC-001.7). The design respected this pin rather than widening silently — broader enforcement requires a small spec amendment.

## Prompt

Widen the validate rule to also flag backtick-embedded `!`-prefix `$ARGUMENTS` patterns. Implementation options:

1. **Two-pattern scan**: keep the spec-pinned narrow regex as the authoritative one; add a second pattern `` `!.*\$ARGUMENTS `` matched per-line. Both emit the same error.
2. **Single superset regex**: `(?m)(?:^[[:space:]]*|\x60)!.*\$ARGUMENTS`. Requires Rust `regex` multi-line mode or per-line application. Requires spec amendment to re-pin the broader form.

Recommendation: option (1) for backward compat with the spec's pinned regex + additive second rule. Document the second rule in the rule registry.

## Acceptance Criteria

- [ ] New test: fixture with backtick-embedded `` `!ecc-workflow ... "$ARGUMENTS"` `` line produces validation error with file + 1-based line number.
- [ ] Existing narrow-regex tests continue to pass.
- [ ] Post-fix repo state (no offending lines of either flavor) continues to exit 0.
- [ ] Rule documentation in `crates/ecc-app/src/validate/commands.rs` module-level docs explains both patterns.
- [ ] If implementing option (2): spec document updated accordingly (minor revision to Definitions + AC-001.7).

## Out of Scope

- Catching `!`-prefix patterns with `<feature>` template variable (separate concern; `commands/design.md:20`, `commands/implement.md:20`).
- Catching `$ARGUMENTS` in other shell contexts (e.g., `$(…)` command substitution, `${ARGUMENTS}` parameter expansion). No such patterns currently exist in templates; YAGNI.
- Markdown-aware parsing to distinguish fenced-code from prose. The current rule conservatively flags both (locked by PC-036).
