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
