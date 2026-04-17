---
id: BL-153
title: "Redact or truncate feature field in tracing::info! to prevent log-injection amplification"
scope: MEDIUM
target: "direct edit"
status: open
created: "2026-04-17"
source: "docs/specs/2026-04-17-spec-command-shell-escaping/design.md Security Findings #1, #9"
ring: trial
tags: [security, observability, logging]
---

## Context

`crates/ecc-workflow/src/commands/transition.rs:391` emits `tracing::info!(feature = %state.feature, ...)` which logs the `feature` field verbatim. After the shell-eval injection fix (spec 2026-04-17), the `--feature-stdin` path accepts up to 64KB of user-supplied text including NUL bytes, shell metacharacters, and control codepoints. This pre-existing log write now:

1. **Amplifies log-bloat**: a 64KB feature string hitting `info!` on every transition bloats log storage.
2. **Amplifies secret-exposure**: users pasting bug reports, stack traces, or error messages may include API keys, passwords, tokens, or `.env` snippets. These land in observability pipelines verbatim.
3. **Enables log-injection**: NUL bytes (`\x00`) and newlines in `feature` can forge fake log records in pipelines that treat NUL/LF as record separators (ELK, Loki naive configs).

Security review flagged this as LOW severity (pre-existing, amplified by stdin path). The shell-escape fix explicitly tracked it as a follow-up rather than bundling — scope discipline.

## Prompt

Redact or truncate the `feature` field in all `tracing` macro call-sites that log workflow state. Options:

1. **Truncate**: `feature = %state.feature.chars().take(120).collect::<String>() + "…"`. Simple; loses long-feature debugging context.
2. **Redact control bytes**: replace NUL/CR/LF and other C0 controls with `\u00XX` escapes before logging. Prevents injection; preserves text.
3. **Gate behind debug!**: demote `info!(feature = ...)` to `debug!`. Sacrifices default observability.
4. **Full redaction**: log `feature_len` and `feature_hash` (SHA256 first 8 chars) instead of content. Maximum security; hardest to debug.

Recommendation: combine (1) + (2) — truncate to 120 chars AND escape control bytes. Single `impl Display` wrapper type `FeatureForLogging(&'a str)` that applies both transformations.

## Acceptance Criteria

- [ ] All `tracing::{info,warn,error,debug,trace}!` macro call-sites that reference `state.feature` (or equivalent) wrap it in a sanitizing formatter.
- [ ] Control bytes (`U+0000`..`U+001F`, `U+007F`) rendered as escape sequences (`\u0000` style) in log output.
- [ ] Features longer than 120 chars truncated with a suffix marker (`…`, `[...]`, or similar).
- [ ] Unit tests: feature containing NUL, LF, and long string all render safely.
- [ ] No regression to existing log-level filtering or structured-field parsing.
- [ ] `#[derive(Debug)]` on `WorkflowState` still redacts (or documents that `Debug` is for dev only).

## Out of Scope

- Redaction of other fields (concern, phase, timestamps are safe ASCII enums/ISO8601).
- Log-injection prevention in non-tracing code paths (e.g., println!).
- Formal secret-scanner integration — separate ticket.
