# ADR 0066: Feature-Input Boundary Validation Policy

## Status

Accepted — 2026-04-17

## Context

Slash-command templates invoke `ecc-workflow init` and `ecc-workflow worktree-name` via the Bash tool. The user-supplied feature description frequently contains shell metacharacters (backticks, quotes, `$`, `\`, newlines) when users paste bug reports or markdown-formatted prose. The pre-fix path interpolated `$ARGUMENTS` into a `!`-prefix shell-eval template line, which broke on zsh's tokenizer when metacharacters appeared.

This ADR records the four-guard boundary-validation policy added to the `--feature-stdin` path in `crates/ecc-workflow/src/commands/feature_input.rs`. The ordering is load-bearing — future refactors MUST preserve it.

## Decision

`read_feature_from_stdin<R: Read>(reader: R, is_tty: bool) -> Result<String, FeatureInputError>` enforces these guards **in this exact order**:

1. **TTY check first**: if `is_tty` → return `Err(IsTty)` with pinned diagnostic `"stdin is a TTY; pipe input or use positional feature arg"`. **No `read()` call is performed.** This prevents indefinite blocking when `--feature-stdin` is invoked interactively.
2. **Bounded read**: `reader.take(64 * 1024 + 1).read_to_end(&mut buf)`. Wrapping with `take(MAX + 1)` BEFORE `read_to_end` prevents DoS from a stuck producer.
3. **Size post-check**: if `buf.len() > 64 * 1024` → return `Err(TooLarge)` with `"stdin exceeds 64KB limit"`.
4. **UTF-8 validation**: `String::from_utf8(buf)`; on failure → `Err(InvalidUtf8)` with `"invalid UTF-8 on stdin"`. No partial `state.json` write occurs.
5. **Strip at-most-one trailing LF**: if the string ends with `'\n'`, drop the last char. Exactly one LF is stripped to accommodate `echo "foo" | …` while preserving intentional multi-line features.
6. **Non-empty check**: if the result is empty → `Err(Empty)` with `"feature is empty"`.

**Additional constraints**:
- Zero `unsafe` code. `std::io::IsTerminal` (stable Rust 1.70+) provides `is_terminal` at the caller boundary; the function itself takes `is_tty: bool` for testability.
- Diagnostic strings are pinned — test assertions depend on verbatim matches.
- `FeatureInputError::Io(std::io::Error)` wraps low-level errors with the fixed prefix `"stdin read error: {source}"` to keep AC assertions stable.
- NUL bytes (`U+0000`) are preserved byte-identical on the stdin path (`String::from_utf8` accepts them). POSIX argv rejects NUL, so the positional path is exempt.

## Consequences

**Positive**:
- Claude Code slash-command templates can accept any byte sequence via stdin without corrupting zsh's tokenizer.
- Four independent validation failures are distinguishable by the caller via `FeatureInputError` variants (enables programmatic matching, structured logging).
- 100ms TTY-rejection SLA is achievable because the TTY check happens before any syscall that could block.
- Regression guard via `ecc validate commands` rule (pinned regex `^[[:space:]]*!.*\$ARGUMENTS`) — CI blocks any PR that re-introduces `$ARGUMENTS` in an `!`-prefix line.

**Negative**:
- Four-guard ordering is a hidden contract: a future developer who swaps TTY-check past the `Read::take` step breaks AC-001.11 silently (test PC-010 with counting-reader catches this structurally).
- 64KB cap is a magic number; users pasting very long bug reports may hit it. Mitigated by the cap being well above realistic feature length (> 8× typical bug report).
- UTF-8 validation rejects raw binary input (legitimate users of `cat /dev/urandom | …` fail fast). Acceptable per spec.

**Neutral**:
- `tracing::info!(feature = %state.feature, …)` at `crates/ecc-workflow/src/commands/transition.rs:391` logs feature text verbatim. Pre-existing. With stdin-delivered 64KB features, this amplifies log-bloat and secret-exposure risk. **Tracked as follow-up backlog entry** — not in this fix's scope.

## Alternatives Considered

- **`printf %q` shell-level escape**: rejected. Fails because `$ARGUMENTS` is substituted by Claude Code's template engine BEFORE shell tokenization; no shell-level escape can recover from already-corrupted quoting.
- **Tempfile handoff** (`--feature-file <path>`): more complex, adds filesystem I/O + cleanup lifecycle. Stdin is strictly simpler and sufficient.
- **Base64 encoding**: opaque in logs; worse observability than UTF-8 stdin.
- **Sanitize at the edge (blacklist metachars)**: UX-hostile. Users legitimately paste markdown containing backticks, quotes, etc.

## Related Artifacts

- Spec: `docs/specs/2026-04-17-spec-command-shell-escaping/spec.md`
- Design: `docs/specs/2026-04-17-spec-command-shell-escaping/design.md` (adversary-PASS round 4, avg 84/100)
- Implementation: `crates/ecc-workflow/src/commands/feature_input.rs` + `commands/spec-*.md`, `commands/project-foundation.md`
- Validate rule: `crates/ecc-app/src/validate/commands.rs`
- Follow-up backlog entries: `tracing::info!(feature)` redaction; widen validate rule to catch backtick-embedded `!$ARGUMENTS`; add `Foundation` variant to `Concern` enum.
