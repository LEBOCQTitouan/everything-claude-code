# Command Reference

## Audit Commands

| Command | Domain |
|---------|--------|
| `/audit-full` | All domains — parallel run with cross-domain correlation |
| `/audit-archi` | Boundary integrity, dependency direction, DDD compliance |
| `/audit-code` | SOLID, clean code, naming, complexity |
| `/audit-convention` | Naming patterns, style consistency |
| `/audit-doc` | Coverage, staleness, drift |
| `/audit-errors` | Swallowed errors, taxonomy, boundary translation |
| `/audit-evolution` | Git hotspots, churn, bus factor, complexity trends |
| `/audit-observability` | Logging, metrics, tracing, health endpoints |
| `/audit-security` | OWASP top 10, secrets, attack surface |
| `/audit-test` | Coverage, classification, fixture ratios, E2E matrix |
| `/audit-web` | Web-based upgrade scan with Technology Radar output |

## Spec-Driven Pipeline

| Command | Purpose |
|---------|---------|
| `/spec` | Auto-classify intent (dev/fix/refactor) and delegate to matching `/spec-*` |
| `/spec-dev` | Spec a new feature — research, grill-me interview, adversarial review |
| `/spec-fix` | Spec a bug fix — investigation, blast radius, adversarial review |
| `/spec-refactor` | Spec a refactoring — smell catalog, current state analysis, adversarial review |
| `/design` | Technical design from spec — file changes, pass conditions, TDD order |
| `/implement` | Execute the design — TDD loop, code review, doc updates |
| `/project-foundation` | Bootstrap project-level PRD, architecture, ADR via guided interview with codebase-aware challenge |

## Side Commands

| Command | Purpose |
|---------|---------|
| `/doc-suite` | Full documentation pipeline: analysis, generation, cartography, validation, coverage, README sync |
| `/verify` | Build + tests + lint + code review + architecture review + drift check |
| `/review` | Robert professional conscience check |
| `/backlog` | Capture and manage implementation ideas |
| `/build-fix` | Fix build/type errors reactively |
| `/commit` | Git commit with intelligent staging, conventional message, atomic enforcement, pre-flight gates |
| `/catchup` | Session resumption — workflow state, git status, stale detection, memory |
| `/create-component` | Scaffold or update an ECC component (agent, command, skill, hook) |
| `/ecc-test-mode` | Isolated worktree for testing ECC config changes |
| `/comms` | Manage comms pipeline infrastructure — init repo, edit strategies, manage drafts, view calendar |
| `/comms-generate` | Generate multi-channel DevRel content from codebase with optional channel filtering |
| `/comms-plan` | Content ideation and scheduling — structured interview, web trend research, publication schedule |

## ECC Validate CLI Commands

Selected `ecc validate` subcommands that warrant detail beyond the CLAUDE.md top-10 table.

### `ecc validate claude-md markers`

Scans every `CLAUDE.md` and `AGENTS.md` under the repo root for `TEMPORARY (BL-NNN)` warning markers and flags any whose backlog entry file (`docs/backlog/BL-NNN-*.md`) is missing on disk.

Flags:

- `--strict`: escalate warnings to errors (exit 1). Intended for CI gates.
- `--audit-report`: emit a markdown table of all markers and their resolution status (`resolved`/`missing`) to stdout instead of per-line diagnostics.

Examples:

```bash
ecc validate claude-md markers                     # Warn on stderr; exit 0
ecc validate claude-md markers --strict            # Error on stderr; exit 1 if any missing
ecc validate claude-md markers --audit-report      # Markdown table to stdout
ecc validate claude-md counts                      # Legacy count validator (subcommand form)
ecc validate claude-md all --strict                # Run both counts + markers
```

The legacy `--counts` flag form is preserved for backward compatibility but emits a `DEPRECATED:` warning on stderr and is scheduled for removal in the next minor release.

#### Kill switch — `ECC_CLAUDE_MD_MARKERS_DISABLED`

Setting the environment variable `ECC_CLAUDE_MD_MARKERS_DISABLED=1` short-circuits the `markers` subcommand to exit 0 with a stderr notice (`markers: disabled via ECC_CLAUDE_MD_MARKERS_DISABLED`), regardless of what markers are on disk.

**Do not rely on this flag for normal operation.** It exists as an emergency CI brake for cases where the lint produces unexpected false positives and blocks every PR. The correct fix in that situation is to either file the missing backlog entry, remove the stale marker, or revert the CI wiring commit — not to rely on this bypass. The env var is intentionally undocumented in README.

Walker details: depth cap 16, deny-list `.git/ target/ node_modules/ .claude/worktrees/`, symlink-skip, lexicographic cross-file order, line-number-ascending within-file order. All diagnostics pass through a control-byte sanitizer before emission.

## ECC Worktree CLI Commands

### `ecc worktree gc`

Garbage-collects stale session worktrees. Consults `.ecc-session` heartbeat + `kill -0 <pid>` to skip live sibling sessions (BL-156). Self-skip via `CLAUDE_PROJECT_DIR` + `.git` walking prevents the SessionStart-triggered automatic GC from deleting its own worktree.

Flags:

- `--force`: bypass the 24h staleness threshold but PRESERVE liveness check (live worktrees still skipped with stderr `--force respects liveness; use --force --kill-live to override`).
- `--force --kill-live`: explicit destructive override; deletes live worktrees after interactive confirmation. Requires both flags together (clap enforces).
- `--yes`: bypass the `--force --kill-live` confirmation prompt. Required in non-TTY (scripts/CI); non-TTY without `--yes` exits non-zero.
- `--dry-run`: preview only. Prints `WOULD DELETE: <name> (reason: ...)` to stdout. Makes zero destructive calls. Skips the `--kill-live` prompt entirely (pointless for non-destructive previews).
- `--json` (with `--dry-run`): emit `[{name, action, reason}]` structured schema.

Examples:

```bash
ecc worktree gc                                 # safe default: live sessions skipped
ecc worktree gc --force                         # bypass 24h age threshold; live still skipped
ecc worktree gc --force --kill-live             # interactive destructive override
ecc worktree gc --force --kill-live --yes       # scripted destructive override
ecc worktree gc --dry-run                       # preview plain text
ecc worktree gc --dry-run --json                # preview JSON
```

#### Environment variables

- `ECC_WORKTREE_LIVENESS_DISABLED=1`: disables BOTH read (GC consult) and write (hook heartbeat) paths. Falls back to BL-150 logic. Emits `WARN: worktree liveness check disabled via ECC_WORKTREE_LIVENESS_DISABLED` once per process. Emergency kill switch — do not rely on for normal operation.
- `ECC_WORKTREE_LIVENESS_TTL_SECS`: heartbeat freshness window in seconds (default 3600 = 60 min). Malformed values (non-numeric, negative, zero) emit WARN + fall back to default; never panic, never silently use 0.
- `ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS`: fallback skip-young window when `current_worktree()` resolver returns `None` (default 3600). Same validation contract as above.
- `CLAUDE_PROJECT_DIR`: used by the self-identity resolver to skip the current session's worktree. Canonicalize-guarded against symlink-based mis-attribution (SEC-002).

### `ecc worktree status`

Displays each session worktree's status (`live` / `stale` / `dead` / `missing_session_file` / `malformed`) using the same `LivenessChecker` helper as `ecc worktree gc`. `--json` emits a `liveness_reason` field per entry.

## ECC Memory Commands

| Command | Purpose |
|---------|---------|
| `ecc memory prune --orphaned-backlogs [--apply]` | Prune memory files for BLs marked implemented. Dry-run by default. |
| `ecc memory restore --trash <YYYY-MM-DD> [--apply]` | Restore trashed memory files from retention dir. |
