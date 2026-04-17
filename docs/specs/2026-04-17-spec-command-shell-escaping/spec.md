# Spec: Fix shell-eval injection in slash-command templates

## Problem Statement

When the user invokes `/spec-dev`, `/spec-fix`, `/spec-refactor`, or `/project-foundation` with an argument containing any shell metacharacter (backtick, double-quote, single-quote, `$`, `\`, newline, Unicode control codepoint), the command fails at initialization with `(eval):N: unmatched "` or executes injected commands via command-substitution. Root cause: seven `!`-prefix template lines literally interpolate `$ARGUMENTS` into a shell command string before zsh tokenization, and Claude Code's template engine offers no built-in escape. Because real-world arguments frequently include markdown-formatted prose (bug reports quoting code in backticks, copy-pasted error messages with quotes), this blocks legitimate workflows. The Rust CLI is argv-safe once bytes cross the `clap` boundary — the fix lives entirely in the adapter layer.

## Research Summary

- Claude Code's `!`-prefix lines evaluate via the user's shell (zsh on darwin); `$ARGUMENTS` is literal string substitution at template-expansion time, not safe parameter passing.
- Template-level escaping (`printf %q`, stdin pipes inside the template) cannot work because the expanded text reaches the tokenizer already corrupted; fixes must be either argv-level or restructure to avoid shell-eval.
- Standard safe patterns: (a) stdin delivery via CLI flag, (b) env-var passing, (c) argv array via Bash-tool direct invocation bypassing shell entirely. Option (c) requires no Rust change; option (a) adds defense-in-depth at the CLI layer.
- The Rust CLI is pre-hardened: clap consumes argv directly, serde_json escapes serialized strings, `WorktreeName::generate` already sanitizes non-allowlisted chars in `crates/ecc-domain/src/worktree.rs:95-108`.
- Prior audit `docs/audits/full-2026-03-29.md:16` claimed "shell injection prevention via `Command::new`" — this verified the Rust subprocess spawn but never reviewed the slash-command template layer above it. Audit-scope gap.
- Same anti-pattern class flagged in `docs/audits/full-2026-04-09.md:77` (SEC-003 SQL interpolation in `sqlite_bypass_store::prune()`) — "escape at the boundary, not inside the expanded string."
- Empirical scan of 26 existing `campaign.md` files and 4 `state.json` files confirmed zero corrupted data on disk — the bug always failed before reaching the binary.

## Definitions (Canonical Test Vectors)

To eliminate ambiguity in edge-case ACs, the spec pins the following canonical sets:

- **`METACHAR_SET`** (ASCII shell metacharacters): `` ` `` (U+0060), `"` (U+0022), `'` (U+0027), `$` (U+0024), `\` (U+005C), `\n` (U+000A), `\r` (U+000D).
- **`CONTROL_SET`** (Unicode control codepoints): C0 range `U+0001..U+001F` **excluding** the common-whitespace subset `{U+0009, U+000A, U+000D}` (which are covered by `METACHAR_SET` or are whitespace); plus `U+007F` (DEL); plus the bidi overrides `U+202A..U+202E`; plus line separators `U+2028..U+2029`. NUL (`U+0000`) is called out separately (see **NUL policy** below).
- **`VALIDATE_REGEX`** (for `ecc validate commands` rule): the POSIX-ERE pattern `^[[:space:]]*!.*\$ARGUMENTS` applied per-line against `.md` files under `commands/`. A match yields a validation error identifying file path and 1-based line number.
- **NUL policy**: POSIX argv cannot carry `U+0000`; the positional `feature` arg path is exempt from NUL-byte support. The `--feature-stdin` path MUST accept `U+0000` bytes and round-trip them JSON-escaped as `\u0000` in `state.json.feature`.
- **UTF-8 policy**: `--feature-stdin` MUST reject bytes that are not valid UTF-8 at the boundary with a non-zero exit and a clear diagnostic; no partial `state.json` write occurs.
- **Trailing-newline policy**: `--feature-stdin` MUST strip **at most one** trailing LF (`U+000A`) from the stdin buffer to accommodate the common case of `echo "foo" | …`. Multiple trailing LFs, trailing CRLF, leading newlines, and internal newlines are preserved byte-identical.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Fix at template + CLI layer, not upstream | Claude Code template engine does literal substitution by design; no built-in escape exists | No |
| 2 | Remove `$ARGUMENTS` from all 7 `!`-prefix shell-eval lines | Only bulletproof fix; prose instructions + Bash-tool argv invocation bypass shell entirely | No |
| 3 | Add `--feature-stdin` flag to `ecc-workflow init` and `worktree-name` | Defense-in-depth: CLI accepts untrusted bytes safely even if a future template author re-introduces `$ARGUMENTS` | No |
| 4 | Mirror `--feature-stdin` in `ecc-cli/commands/workflow.rs` delegator | Parity requirement — delegator silently breaks if signatures drift | No |
| 5 | Add `ecc validate commands` rule rejecting `VALIDATE_REGEX` pattern | CI-enforced regression prevention; future-proofs the fix | No |
| 6 | Test coverage: 4 named layers (template lint, CLI argv unit, stdin unit, delegator parity) + property/fuzz | User-selected; matches risk profile and long-tail input coverage | No |
| 7 | Grill-me secondary shell-eval (`campaign append-decision`) out of scope | Separate follow-up ticket keeps this spec crisp; not the root-cause fix | No |
| 8 | SEC-003 SQL interpolation NOT bundled | Different subsystem, different severity; conflation dilutes spec scope | No |
| 9 | Document audit-scope gap + create backlog entry | Prior audits missed template layer; broader slash-command audit needed | No |
| 10 | Zero data migration | Empirical scan confirmed no corrupted state on disk | No |
| 11 | UTF-8 invalid bytes on stdin → reject at boundary | Matches `rules/rust/security.md` "validate at boundary"; JSON serialization requires UTF-8 | No |
| 12 | Trailing-newline policy: strip exactly one LF | Accommodates `echo "foo" \| …` common case; preserves intentional multi-line features | No |
| 13 | TTY-attached stdin with `--feature-stdin` → error within 100ms, no hang | Prevents interactive-mode foot-gun; matches Unix convention (e.g., `sort -`) | No |
| 14 | Stdin size cap: 64KB hard limit | Prevents trivial DoS from stuck producer; far exceeds realistic feature text length | No |
| 15 | Release coupling: template rewrite + CLI flag ship in same commit/tag | Partial rollout would produce templates calling a non-existent flag | No |

## User Stories

### US-001: Shell-metacharacter-safe slash-command initialization

**As a** developer invoking `/spec-dev`, `/spec-fix`, `/spec-refactor`, or `/project-foundation`,
**I want** the command to accept any argument string — including `METACHAR_SET`, `CONTROL_SET`, and NUL bytes via stdin — without shell-eval failures,
**so that** I can paste bug reports, error messages, and markdown-formatted feature descriptions directly as arguments.

#### Acceptance Criteria

**Template structural**
- **AC-001.1a** (lint) Given any `.md` file under `commands/`, when `ecc validate commands` runs with the new rule active, then it returns zero violations for the current-after-fix repository state. (Verification: automated; pinned `VALIDATE_REGEX` used by the rule.)
- **AC-001.1b** (behavior) Given each of `/spec-dev`, `/spec-fix`, `/spec-refactor`, `/project-foundation` is invoked with the canonical payload `` feat-`rm -rf /`-"$(whoami)"-'\n' `` (concatenating representative `METACHAR_SET` bytes), when the command initializes, then `state.json.feature` contains the exact input bytes and the session does not crash or attempt command-substitution.

**CLI argv and stdin path**
- **AC-001.2a** (stdin) Given `ecc-workflow init <concern> --feature-stdin` receives any byte-sequence composed of `METACHAR_SET ∪ CONTROL_SET ∪ {NUL}` (valid UTF-8), when the command completes, then `state.json.feature` deserializes to a string whose bytes equal the stdin input minus at most one trailing LF.
- **AC-001.2b** (argv) Given `ecc-workflow init <concern> <feature>` is invoked positionally with any feature bytes drawn from `METACHAR_SET ∪ CONTROL_SET` (excluding NUL, per POSIX argv), when the command completes, then `state.json.feature` round-trips byte-identical.
- **AC-001.3** Given `--feature-stdin` is passed and stdin is empty (immediate EOF with no bytes), no positional `feature` arg is given, then the command exits with code `2` and stderr contains `feature is empty` (or equivalent clear diagnostic).
- **AC-001.4** Given both `--feature-stdin` and a positional `feature` arg are supplied, then clap (or explicit validation) rejects the input with exit code `2` and stderr contains `--feature-stdin conflicts with positional feature`.
- **AC-001.5** Given any of the existing integration tests in `crates/ecc-workflow/tests/*.rs` that call `ecc-workflow init <concern> <feature>` positionally, when the fix lands, then each test passes without modification. The fix's test harness MUST NOT modify these files.
- **AC-001.6** (parity) Given the same concern and feature input, when invoked via `ecc-workflow init --feature-stdin` AND via `ecc workflow init --feature-stdin` (through the ecc-cli delegator), then both invocations produce: (i) identical exit codes, (ii) identical stderr error-class (same first line), (iii) identical resulting `state.json.feature` bytes.

**Validate-rule regression**
- **AC-001.7** Given a crafted `commands/fixture.md` containing the line `!ecc-workflow init dev "$ARGUMENTS"`, when `ecc validate commands` runs, then it exits non-zero and stderr contains the file name and 1-based line number of the offending line. Pinned rule regex: `^[[:space:]]*!.*\$ARGUMENTS`.

**Boundary and property**
- **AC-001.8** Given a `proptest` test with `ProptestConfig { cases: 1024, .. Default::default() }` generating arbitrary valid-UTF-8 strings up to 4096 bytes (via `proptest::string::string_regex("(?s-u:\\PC|\\s){0,4096}").unwrap()` or equivalent), when each generated string is piped via `--feature-stdin` to `ecc-workflow init dev`, then the resulting `state.json.feature` equals the input minus at most one trailing LF. Regression corpus is persisted under `crates/ecc-workflow/tests/proptest-regressions/`.
- **AC-001.9** (invalid UTF-8) Given `--feature-stdin` receives a byte sequence that is not valid UTF-8 (e.g., lone `0xFF` byte), when the command runs, then it exits non-zero before any `state.json` write, and stderr contains `invalid UTF-8 on stdin` (or equivalent).
- **AC-001.10** (size cap) Given `--feature-stdin` receives 65_537 bytes (64KB + 1), when the command runs, then it exits non-zero with stderr containing `stdin exceeds 64KB limit`; no `state.json` write occurs. Exactly 65_536 bytes (64KB) succeeds.
- **AC-001.11** (TTY guard) Given `--feature-stdin` is passed with stdin attached to a TTY (`libc::isatty(0) == 1`), when the command runs, then it exits non-zero within 100ms with stderr containing `stdin is a TTY; pipe input or use positional feature arg`. No blocking read occurs.
- **AC-001.12** (trailing newline) Given `printf 'foo\n' \| ecc-workflow init dev --feature-stdin`, when the command completes, then `state.json.feature == "foo"` (single trailing LF stripped). Given `printf 'foo\n\n' \| …`, then `state.json.feature == "foo\n"` (only one LF stripped). Given `printf 'foo\r\n' \| …`, then `state.json.feature == "foo\r"` (CR preserved, only the final LF stripped).

#### Dependencies

- Depends on: none (standalone fix).

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `commands/spec-dev.md` | adapter (templates) | Remove `$ARGUMENTS` from `!`-prefix lines at L14 (init) and L19 (worktree-name); replace with prose instructing Claude to invoke the CLI via the Bash tool with argv. |
| `commands/spec-fix.md` | adapter (templates) | Same pattern: L14 (init) + L19 (worktree-name). |
| `commands/spec-refactor.md` | adapter (templates) | Same pattern: L14 (init) + L19 (worktree-name). |
| `commands/project-foundation.md` | adapter (templates) | Remove `$ARGUMENTS` from `!`-prefix line at L18 (init). |
| `crates/ecc-workflow/src/main.rs` | CLI | Add `--feature-stdin` flag on `Init` and `WorktreeName` subcommands (optional, mutex with positional via clap `ArgGroup`). |
| `crates/ecc-workflow/src/commands/init.rs` | CLI adapter | Read feature bytes from stdin when `--feature-stdin` set; apply UTF-8 validation, 64KB cap, TTY guard, trailing-LF strip per AC-001.9/10/11/12. |
| `crates/ecc-workflow/src/commands/worktree_name.rs` | CLI adapter | Same stdin-read path (shared helper in a small module, e.g., `src/commands/feature_input.rs`). |
| `crates/ecc-cli/src/commands/workflow.rs` | CLI delegator | Mirror `--feature-stdin` in `WorkflowCommand::Init` and `WorkflowCommand::WorktreeName` enums; propagate via `build_args` / `delegate_to_ecc_workflow`; stream stdin through to the child process. |
| `crates/ecc-app/src/validate/commands.rs` | app validator | Add per-line scan with pinned `VALIDATE_REGEX`; emit error with file path + 1-based line number via `TerminalIO`. |
| `crates/ecc-workflow/tests/init.rs` | integration tests | Add special-character round-trip tests (AC-001.2a/2b), stdin-delivery tests, mutex-exclusive tests (AC-001.3/4), property/fuzz test (AC-001.8), UTF-8 rejection (AC-001.9), size cap (AC-001.10), TTY guard (AC-001.11), trailing-newline (AC-001.12). |
| `crates/ecc-workflow/tests/worktree_name.rs` (new) | integration tests | Parallel matrix for `worktree-name` stdin delivery. |
| `crates/ecc-workflow/tests/proptest-regressions/` (new dir) | persistent corpus | Generated by proptest on first failure; committed to VCS. |
| `crates/ecc-integration-tests/tests/workflow_cli_parity.rs` | integration tests | New parity test per AC-001.6. |

**Explicit non-impact (untouched)**:
- `crates/ecc-domain` (all modules)
- `crates/ecc-ports`
- `crates/ecc-infra`
- `hooks/` (no user-text shell interpolation confirmed)
- `phase_gate`, `memory_write` (do not read `feature` field)
- `state.json` schema version (no bump)
- `WorktreeName` invariants and `make_slug` behavior

## Constraints

1. **`ecc-cli` ↔ `ecc-workflow` parity**: `--feature-stdin` MUST be added to both `WorkflowCommand::Init`/`WorktreeName` (in `ecc-cli`) and `Subcommand::Init`/`WorktreeName` (in `ecc-workflow`) in the same commit. Parity test (AC-001.6) required.
2. **Backward compatibility**: existing integration tests pass feature positionally via `Command::new(bin).args([...])`. Positional arg MUST remain valid; `--feature-stdin` is strictly additive.
3. **Mutually-exclusive enforcement**: supplying both positional `feature` AND `--feature-stdin` MUST fail fast (clap `ArgGroup` preferred; manual validation acceptable).
4. **Preserve `WorktreeName` invariants**: domain-layer sanitization in `crates/ecc-domain/src/worktree.rs:95-108` remains authoritative; do NOT duplicate sanitization at the CLI layer.
5. **Preserve `state.json` schema**: no `version` bump; `feature: String` semantics unchanged.
6. **Template must work under zsh AND bash**: development spans macOS (zsh) and Linux CI (bash). No `printf %q` or shell-specific escaping inside `!`-prefix lines. (The fix removes shell-eval entirely, so this is satisfied by construction.)
7. **No shell-layer escape reliance**: fix MUST be argv-level, not shell-level.
8. **Test property**: `METACHAR_SET ∪ CONTROL_SET ∪ {NUL}` round-trip byte-identical through `--feature-stdin`, modulo the single-trailing-LF strip; `METACHAR_SET ∪ CONTROL_SET` (NUL excluded) round-trip byte-identical via positional argv.
9. **Atomic commits per TDD**: each AC produces (test-commit → impl-commit → refactor-commit).
10. **Stdin hard cap**: 64KB. Over-limit → non-zero exit, no partial write (AC-001.10).
11. **Release coupling**: the template rewrite and the CLI `--feature-stdin` flag MUST ship in the same commit series (and the same release tag). Templates MUST NOT reference `--feature-stdin` in a commit that lands before the CLI flag commit. (The fix's template rewrite does NOT actually use `--feature-stdin` — it uses the Bash-tool argv path — so this constraint is primarily forward-looking for any template author who adopts the flag later.)
12. **Validate-rule specificity**: the new rule's regex is pinned as `^[[:space:]]*!.*\$ARGUMENTS` and MUST be tested against: (a) the fixed post-fix repo state (zero violations), (b) a synthetic fixture containing the forbidden pattern (one violation, correct line number).

## Non-Requirements

- **NOT** fixing the `!ecc-workflow campaign append-decision --question "…" --answer "…"` shell-eval risk inside grill-me loops. Tracked as a **separate follow-up ticket**; out of scope here.
- **NOT** bundling SEC-003 (`sqlite_bypass_store::prune()` SQL interpolation) into this fix.
- **NOT** redesigning the slash-command template engine globally.
- **NOT** adding data migration for existing `state.json` or `campaign.md` files (empirically clean).
- **NOT** sanitizing at the edge via blacklist/rejection of shell metacharacters.
- **NOT** auditing every other `!`-prefix line across all commands exhaustively in this ticket (regression prevented by the new validate rule).
- **NOT** planning a future deprecation of `--feature-stdin` (if Claude Code ever provides a native escape, a separate deprecation spec will handle it).
- **NOT** guarding against concurrent `/spec-*` invocations against the same `state.json` path (pre-existing flock guard is authoritative; this fix does not widen or narrow that surface beyond what the existing tests cover).

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|---|---|---|
| `ecc-workflow` CLI stdin boundary | New `--feature-stdin` flag on `init` + `worktree-name` | Stdin reader must handle EOF, enforce 64KB cap, validate UTF-8, reject TTY, strip at-most-one trailing LF |
| Slash-command template engine (Claude Code) | Template rewrite | Claude-driven Bash-tool invocation with argv replaces shell-eval; argv-safe by construction |
| `ecc validate commands` rule | New per-line lint rule | CI gate: any `!`-prefix + `$ARGUMENTS` in `commands/*.md` is a validation error with file + line |
| `ecc-cli` → `ecc-workflow` delegator | Flag mirroring + stdin streaming | Parity test (AC-001.6) ensures `ecc workflow init` matches `ecc-workflow init` on exit code, stderr class, and `state.json.feature` bytes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|---|---|---|---|
| Fix visible in release | project | `CHANGELOG.md` | Unreleased / fix: "fix(workflow): remove `$ARGUMENTS` from `!`-prefix shell-eval lines; add `--feature-stdin`; add validation rule" |
| New CLI flag | project | `docs/commands-reference.md` | Document `--feature-stdin` on `ecc-workflow init` and `worktree-name` with size cap, UTF-8, TTY, trailing-LF semantics |
| Gotcha / anti-pattern | project | `CLAUDE.md` Gotchas section | Add note about `!`-prefix + `$ARGUMENTS` prohibition and the new validate rule |
| Convention | project | `rules/ecc/development.md` Anti-Patterns section | Add anti-pattern entry |
| Audit-scope gap (non-file) | follow-up | `docs/backlog/` | New backlog entry for comprehensive slash-command template audit |
| Secondary grill-me shell-eval risk (non-file) | follow-up | `docs/backlog/` | New backlog entry for `campaign append-decision` shell-eval |

## Open Questions

_(none — all resolved during grill-me interview and adversarial round 1)_

## Consulted Sources

- `docs/sources.md` → `cli-design` subject: [clap documentation](https://docs.rs/clap/latest/clap/) — flag mutex group (`ArgGroup`) patterns
- `docs/sources.md` → `ai-coding` subject: [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code) — slash-command template semantics

## Revision History

| Round | Date | Notes |
|-------|------|-------|
| 1 | 2026-04-17 | Initial draft after grill-me |
| 2 | 2026-04-17 | Adversary round 1 CONDITIONAL: split AC-001.1/1.2 into a/b variants, added AC-001.9/10/11/12 for edge cases (UTF-8, size cap, TTY, trailing-newline), pinned VALIDATE_REGEX + CONTROL_SET + METACHAR_SET, pinned proptest iteration budget, added release-coupling + deprecation + concurrency non-requirements, resolved validate/commands.rs path |

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Is the shell interpolation the true root cause? | User requested deeper investigation; after broader grep the 4-file/7-line scope was confirmed and Claude Code template engine confirmed to do literal substitution with no upstream escape | User |
| 2 | Fix scope — minimal vs proper? | Proper fix with guard-rail: template removal + `--feature-stdin` CLI flag + `ecc validate commands` rule | Recommended |
| 3 | Test coverage scope? | All 4 layers (CLI argv, stdin, validation rule, template integration) plus property/fuzz on stdin round-trip | User |
| 4 | Which regression vectors should be hard constraints? | All four: ecc-cli/workflow parity, positional backward-compat, grill-me secondary as Non-Requirement follow-up, preserve domain invariants | User |
| 5 | How to incorporate prior audit findings? | Document audit-scope gap in spec + create backlog entry for comprehensive slash-command template audit | User |
| 6 | Reproduction steps sufficient? | Derived repro + canonical test-vector list (backtick, double-quote, single-quote, dollar, backslash, newline, Unicode control) | Recommended |
| 7 | Data impact — any migration or cleanup needed? | Zero data impact confirmed by empirical scan of 26 campaign.md files and 4 state.json files — all feature strings clean | User |
| **Root cause** | Symptom vs cause distinction | Symptom: `(eval):N: unmatched "` or injected-command execution. Root cause: literal `$ARGUMENTS` substitution into `!`-prefix shell string before zsh tokenization; Claude Code template engine provides no upstream escape | n/a |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Shell-metacharacter-safe slash-command initialization | 12 (1a, 1b, 2a, 2b, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12) | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1a | `ecc validate commands` returns zero violations against the post-fix repo state (pinned VALIDATE_REGEX) | US-001 |
| AC-001.1b | Canonical payload `` feat-`rm -rf /`-"$(whoami)"-'\n' `` round-trips byte-identical through each spec-* / project-foundation command invocation | US-001 |
| AC-001.2a | Stdin-delivered feature (`METACHAR_SET ∪ CONTROL_SET ∪ {NUL}`) round-trips byte-identical modulo single trailing LF | US-001 |
| AC-001.2b | Positional-arg feature (`METACHAR_SET ∪ CONTROL_SET`, NUL excluded) round-trips byte-identical | US-001 |
| AC-001.3 | Empty stdin + no positional → exit code 2 with `feature is empty` | US-001 |
| AC-001.4 | Both `--feature-stdin` and positional → exit code 2 with `--feature-stdin conflicts with positional feature` | US-001 |
| AC-001.5 | Existing integration tests in `crates/ecc-workflow/tests/*.rs` pass without modification | US-001 |
| AC-001.6 | `ecc workflow init --feature-stdin` (via ecc-cli delegator) matches `ecc-workflow init --feature-stdin` on exit code, stderr class, and state.json.feature bytes | US-001 |
| AC-001.7 | Crafted `commands/fixture.md` with `!ecc-workflow init dev "$ARGUMENTS"` → validation error with file + 1-based line | US-001 |
| AC-001.8 | Proptest (1024 cases, ≤4KB UTF-8) stdin round-trip preserves bytes modulo single trailing LF | US-001 |
| AC-001.9 | Invalid UTF-8 on stdin → non-zero exit with `invalid UTF-8 on stdin`; no state.json write | US-001 |
| AC-001.10 | Stdin 65_537 bytes → non-zero exit with `stdin exceeds 64KB limit`; 65_536 bytes succeeds | US-001 |
| AC-001.11 | TTY-attached stdin + `--feature-stdin` → non-zero exit within 100ms with `stdin is a TTY; pipe input or use positional feature arg` | US-001 |
| AC-001.12 | Trailing-LF policy: `'foo\n'` → `"foo"`, `'foo\n\n'` → `"foo\n"`, `'foo\r\n'` → `"foo\r"` | US-001 |

### Adversary Findings

| Round | Dimension | Score | Verdict | Key Rationale |
|-------|-----------|-------|---------|---------------|
| 1 | Ambiguity | 62 | CONDITIONAL | AC-001.1 grep-as-proxy for behavioral claim; AC-001.2 OR-disjunction; "Unicode control codepoint" underspecified |
| 1 | Edge cases | 55 | CONDITIONAL | Invalid UTF-8, stdin size cap, CRLF, TTY, NUL byte, concurrency all unaddressed |
| 1 | Scope | 85 | PASS | Non-Requirements explicit; cutline defensible |
| 1 | Dependencies | 80 | PASS | US-001 standalone; "or equivalent" hedge on validate/commands.rs resolved in round 2 |
| 1 | Testability | 65 | CONDITIONAL | AC-001.1 negative-space; AC-001.8 proptest budget missing; AC-001.6 "behavior matches" undefined |
| 1 | Decisions | 82 | PASS | All 10 rationales load-bearing |
| 1 | Rollback | 72 | PASS | No schema bump, no migration; deprecation + release coupling flagged for round 2 |
| 2 | Ambiguity | 90 | PASS | Definitions block pins METACHAR_SET / CONTROL_SET / VALIDATE_REGEX; diagnostic strings bounded |
| 2 | Edge cases | 88 | PASS | 4 new ACs close gap; BOM residual noted as non-blocking |
| 2 | Scope | 92 | PASS | 8-item Non-Requirements, narrower than root cause warrants |
| 2 | Dependencies | 95 | PASS | Verified file paths exist; 7-line count verified against grep |
| 2 | Testability | 88 | PASS | 12 ACs deterministic; proptest budget pinned; parity axes concrete |
| 2 | Decisions | 85 | PASS | 15 decisions; release coupling closes rollout gap |
| 2 | Rollback | 80 | PASS | Release coupling + zero-migration + pinned schema |
| **2 avg** | — | **86** | **PASS** | Advance to `/design` |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-17-spec-command-shell-escaping/spec.md` | Full spec (Problem, Research, Definitions, Decisions, User Stories, ACs, Affected Modules, Constraints, Non-Requirements, E2E, Doc Impact, Open Questions, Consulted Sources, Revision History, Phase Summary) |
| `docs/specs/2026-04-17-spec-command-shell-escaping/spec-draft.md` | Final working draft (mirror of spec.md) |
| `docs/specs/2026-04-17-spec-command-shell-escaping/campaign.md` | 8 decisions persisted during grill-me and adversary rounds |
| `/Users/titouanlebocq/.claude/plans/expressive-wandering-turtle.md` | Plan-mode doc-first preview (spec draft + doc preview) — approved |

