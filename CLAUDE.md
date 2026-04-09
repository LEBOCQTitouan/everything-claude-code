# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A collection of production-ready agents, skills, hooks, commands, rules, teams, and MCP configurations for software development using Claude Code. Core CLI is implemented in Rust (Hexagonal Architecture, DDD, Clean Code).

## Running Tests

```bash
cargo test              # Run all Rust tests (3028 tests)
cargo nextest run       # Faster test runner (~60% speedup, per-test isolation)
bats tests/statusline/  # Run statusline Bats tests (22 tests)
cargo clippy -- -D warnings  # Lint with zero warnings
cargo deny check        # Supply chain audit (licenses + advisories)
cargo vet check          # Supply chain audit (human-review verification)
cargo llvm-cov --workspace   # Coverage report (works on macOS)
cargo build --release   # Build release binary
cargo mutants -p ecc-domain   # Mutation testing (domain crate)
cargo xtask mutants            # Structured mutation testing (all scoped crates)
cargo xtask mutants --in-diff  # Diff-scoped mutation testing
cargo dist build        # Local release build test (cargo-dist, requires cargo-dist installed)
cargo +nightly miri test -p ecc-flock  # Miri UB detection (nightly only; FFI tests auto-skipped)
```

Optional: install [sccache](https://github.com/mozilla/sccache) for 11-14% faster builds (`export RUSTC_WRAPPER=sccache`). See `docs/getting-started.md` § Build Acceleration for full setup.

## Architecture

Hexagonal architecture: domain → ports → app → infra → CLI (9 crates). `ecc-workflow` is a standalone binary for workflow state management. `ecc-flock` is a shared POSIX flock utility. `workflow-templates/` contains installable GitHub Actions YAML templates. See `docs/ARCHITECTURE.md` for full structure.

## CLI Commands (top 10)

```
cargo test / cargo nextest run       Run all tests
cargo clippy -- -D warnings          Lint with zero warnings
cargo build --release                Build release binary
ecc workflow init|transition|status  Workflow state machine
ecc validate <target>                Validate agents|commands|hooks|skills|rules|teams
ecc validate claude-md --counts      Cross-check CLAUDE.md numeric claims
ecc drift check [--json]             Compute spec-vs-implementation drift
ecc docs update-module-summary       Update MODULE-SUMMARIES.md entries
ecc docs coverage --scope <path>     Doc comment coverage per module
ecc diagram triggers --changed-files Evaluate diagram generation heuristics
ecc commit lint --staged             Validate atomic commit concerns
ecc hook <id> [profiles]             Run a hook by ID
ecc backlog next-id|reindex|list     Backlog operations (list --available filters in-progress)
ecc worktree gc|status               Worktree lifecycle
ecc status [--json]                  Diagnostic snapshot (versions, phase, components)
ecc dev on|off|switch                Toggle/switch ECC config
```

Full CLI reference: `docs/commands-reference.md`

### Spec-Driven Pipeline (Doc-First)

`/spec` → `/spec-dev`, `/spec-fix`, `/spec-refactor` → `/design` → `/implement`

- `/spec` auto-classifies intent (dev/fix/refactor) and delegates to the matching `/spec-*` command
- Each `/spec-*` runs web research, grill-me interview, enters **Plan Mode** for doc-first review (spec draft + upper-level doc preview), adversarial review, then outputs the spec
- `/design` produces the technical design, enters **Plan Mode** for architecture preview (arch docs, diagrams, bounded contexts), then adversarial review
- `/implement` enters **Plan Mode** for implementation steps, then executes TDD loops with TaskCreate tracking and mandatory doc updates
- All three phases use Plan Mode so the user reviews artifacts before execution
- State machine in `.claude/workflow/` enforces phase ordering via `ecc-workflow` binary

## Command Workflows

Slash command workflows defined in `commands/` are mandatory. Follow every phase and step exactly as specified — do not skip, reorder, or modify phases. The spec-driven pipeline is enforced by `.claude/workflow/state.json` and hook-based gates.

## Dual-Mode Development

- **Spec-driven** (`/spec` or `/spec-*` → `/design` → `/implement`): for features, fixes, and refactors requiring design review.
- **Direct** (edit + `/verify`): for small, well-understood changes
- Use `/audit-*` independently at any time for health checks
- Use `/review` as a final craft conscience gate before shipping

## Gotchas

- Brevity rule (`rules/common/brevity.md`): all agents inherit output compression — no filler, no hedging, no pleasantries. Preserves code blocks and technical terms. See [caveman](https://github.com/JuliusBrussee/caveman).
- Workflow state is worktree-scoped: `resolve_state_dir()` resolves to `<git-dir>/ecc-workflow/state.json` in git repos (per-worktree isolation), falling back to `.claude/workflow/` for non-git dirs. A `.state-dir` anchor file at `.claude/workflow/.state-dir` pins the state directory path so hook subprocesses resolve correctly regardless of CWD. Written by `ecc-workflow init`, deleted by `ecc-workflow reset --force`. If anchor is missing/corrupt/stale, falls back to git-based resolution. The phase-gate hook additionally derives the worktree from the gated file path (walking parents to find `.git` file → `gitdir:` line), bypassing `CLAUDE_PROJECT_DIR` when it points to the main repo. Run `ecc-workflow status` from a worktree to verify isolation. Old state at `.claude/workflow/state.json` is auto-migrated on first write.
- `ecc workflow` mirrors `ecc-workflow` — use either during migration; `ecc-workflow` will become a thin wrapper
- `ecc-domain` crate must have zero I/O imports — pure business logic only (enforced by hook)
- Agent frontmatter `model` field controls which Claude model runs the agent — wrong value silently degrades quality
- Agent frontmatter `effort` field (low/medium/high/max) controls thinking budget via SubagentStart hook — must match model tier
- Test count in CLAUDE.md must be updated after adding or removing tests
- `pre:edit-write:workflow-branch-guard` blocks `.github/workflows/` edits on main/master/production — create a feature branch first
- ECC hooks are enforced by default. Bypass individual hooks via `ecc bypass grant --hook <hook_id> --reason <reason>` with full audit trail (ADR-0055). The old `ECC_WORKFLOW_BYPASS` env var is defunct and ignored (ADR-0056).
- `pre:write-edit:worktree-guard` blocks Write/Edit/MultiEdit on main branch — Claude must call EnterWorktree first; bypass with `ecc bypass grant` (lazy worktree: created on first write, not session start)
- `session:end:worktree-merge` auto-merges worktree to main at session end via `ecc-workflow merge` (rebase + full verify + ff-only + safety-checked auto-cleanup). After successful merge, 5-point safety check (uncommitted changes, untracked files, unmerged commits, stash, remote push) runs inside the merge lock. If all pass, worktree directory + branch are deleted automatically. If any check fails, worktree is preserved with a warning listing which checks failed. If merge fails, worktree preserved; retry with `ecc-workflow merge`
- `session:start` triggers `ecc worktree gc` automatically to clean stale worktrees from previous sessions (best-effort, non-blocking). GC now skips unmerged worktrees unless `--force` is passed
- Claude Code's `EnterWorktree` prepends `worktree-` to branch names (e.g., `ecc-session-*` becomes `worktree-ecc-session-*`). ECC handles both forms — `WorktreeName::parse()` strips the prefix automatically
- Fix-round budget: max 2 fix attempts per PC/E2E test before asking the user for help via AskUserQuestion (inspired by Stripe Minions CI budget pattern). User can grant more rounds, skip, abort, or provide guidance. Hard cap of 8 total rounds per PC.
- `test_names` field in tdd-executor output (BL-050): list of fully qualified test function names. When absent (older invocations), TDD Log shows "--". Type: list of strings. Backward compat: column degrades gracefully.
- `ECC_METRICS_DISABLED=1` disables all harness metrics recording (fire-and-forget kill switch). Reads remain functional. Set in environment to opt out of metrics overhead.
- CLI-redirected agents (doc-generator, evolution-analyst, backlog-curator) call `ecc analyze` and `ecc backlog` commands for raw data — agent still interprets results
- Audit-challenger is conditional: skipped when <3 findings AND all ≤MEDIUM (see BL-124)
- Local LLM offload (BL-128): agents with `local-eligible: true` call Ollama via MCP for mechanical tasks. Requires `ollama-mcp` bridge. Without Ollama, agents fall back to hosted model. Kill switch: `ecc config set local-llm.enabled false`
- `--continue` flag on /spec-* commands auto-invokes /design after spec PASS (BL-127). Opt-in only, never default.
- `design-reviewer` agent replaces sequential uncle-bob/robert/security-reviewer in /design (ADR 0058, BL-127). Old agents remain for standalone use.
- `ecc audit cache check/clear` — per-domain audit caching with content-hash + TTL invalidation (BL-127). `--force` bypasses cache.
- Batched tdd-executor: independent same-file PCs dispatch as single batch to reduce subagent overhead (BL-127)
- Glossary: **write-guard** = PreToolUse hook blocking writes outside worktree (exit 2); **lazy worktree** = worktree created on-demand at first write; **session merge** = automatic rebase+verify+ff-merge at session end; **fix-round budget** = max 2 fix attempts per PC before user escalation; **coverage delta** = before/after test coverage % comparison across TDD loop; **bounded context enumeration** = listing affected DDD contexts in /design output; **per-test-name inventory** = individual test function names from TDD cycles; **harness metrics** = hook success rate, phase-gate violation rate, agent recovery rate, commit atomicity score (SLO targets: 99%/5%/80%/95%); **tracer bullet** = thin vertical slice through all architectural layers, implemented first to validate architecture E2E before horizontal expansion

