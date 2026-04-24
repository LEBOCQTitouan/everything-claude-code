# ECC v2 — Clean-Room Foundation

**Scope:** clean-room Rust re-implementation of [Everything Claude Code](https://github.com/titouanlebocq/everything-claude-code). Preserves the proven architectural spine (hexagonal, doc-first pipeline, worktree isolation) and fixes the structural debt that is causing in-flight data loss today.

**Audience:** the person (or agent) who will build v2 from an empty directory.

**Status as of authoring:** ECC v1 is at workspace version 4.2.0. ~109,442 LOC Rust across 9 crates. 67 agents, 35 commands, 121 skills, 80 rules, 3 teams, 67 ADRs, 3,384 tests (Grade B audit, 0 CRITICAL, 2 HIGH, 21 MEDIUM, 15 LOW).

**Context for this rewrite:** the BL-156 bug (`ecc worktree gc` deletes live parallel-session worktrees) destroyed the in-flight BL-156 implementation itself twice during the session that produced this document. The recursive failure is the strongest possible argument that the current architecture has a structural leak — worth preserving the spine, replacing the broken load-bearing mechanics.

---

## 0. Reading guide

| Section | For | Length |
|---------|-----|--------|
| 1. Product intent | Understand what ECC is for | short |
| **2. First principles** | **Understand the philosophy that drives every other decision** | **short — critical** |
| 3. Feature catalog | Know what has to be reproduced | long — reference |
| 4. Architecture blueprint | Know what survives | medium |
| 5. Pain points | Know what v2 must fix | medium |
| 6. v2 direction | Know the shape of the solution | medium |
| 7. Build plan | Know the order of work | long |
| 8. Non-goals | Know what NOT to do | short |
| 9. Open questions | Decisions still pending | short |
| Appendices A-C | Migration tables, ADR carry-forward, test strategy | reference |

**Read sections 1, 2, 5, 6, 7 end-to-end to understand the plan.** Sections 3, 4, and the appendices are reference material — skim as needed during implementation. Section 2 is deliberately first-class; when the PRD is silent on a specific decision, re-derive from P1-P4.

---

## 1. Product intent

### 1.1 What ECC is

ECC is **infrastructure for agentic software engineering**. It wraps Claude Code (the CLI) with:

1. **A spec-driven pipeline** — every non-trivial change flows through `/spec → /design → /implement`, gated by adversarial reviewers and anchored in durable artifacts (`docs/specs/<date>-<slug>/spec.md`, `design.md`, `tasks.md`, `campaign.md`).
2. **Opinionated agent roles** — 67 agents with specific mandates (uncle-bob, robert, security-reviewer, spec-adversary, solution-adversary, planner, tdd-executor, code-reviewer, …). Adversarial review is first-class, not optional.
3. **A Rust CLI (`ecc`) + workflow binary (`ecc-workflow`)** — the only components in the system that have hard correctness requirements. The rest is prompts.
4. **A hook system** — pre/post tool use, session start/end, subagent lifecycle — enforced by default, auditable bypass via `ecc bypass grant`.
5. **Session isolation** — each pipeline run is a git worktree, merged on session end via a serialized, verified ff-only merge.
6. **A backlog + cartography + memory + observability stack** — durable state that persists across sessions (ideas, user journeys, structured logs, token costs, metrics).

### 1.2 What v2 changes

User directive for this rewrite: **same Rust stack, fix structural debt, go further**. Four shifts:

1. **Sessions never eat each other's work.** Worktree GC must not delete live sessions. State.json must be per-session, not shared. This is the #1 correctness goal.
2. **Shrink surface area where possible.** 121 skills is too many — many are stale language-pattern dumps. 34 commands is too many — several are thin variants. Audit and consolidate.
3. **Bounded plans by default.** The pipeline should produce specs and designs that are executable in a single session without routine context exhaustion. 75-PC implementation specs (like BL-156) are a symptom of overreach.
4. **Observable failures.** 23 `let _ =` in cartography is not a budget problem, it's a culture problem. v2 logs every I/O failure; silent swallowing is an arch-level rule.

Not changing: the Rust + hexagonal spine, the doc-first pipeline, adversarial review, POSIX flock for concurrency, ADR discipline, TDD + property tests, the CLI command model.

### 1.3 Success criteria for v2 (one year after ship)

- Two or more parallel sessions can run in the same repo for days without data loss.
- A fresh contributor can add a new command end-to-end (domain → port → adapter → CLI wiring → agent integration) in one working day with only the in-repo documentation.
- `cargo test --workspace` stays under 3 minutes on a developer laptop.
- Every production I/O call has either a test proving the failure path or a `tracing::warn!` proving the failure is visible.
- No `let _ =` on a `Result` outside tests (enforced by clippy lint or grep hook).
- `ecc worktree gc` has six integration tests covering the liveness matrix and zero known false-positives.

---

## 2. First principles

Everything in this document — the feature catalog, the architecture, the pain points, the build plan — follows from one thesis and four principles. They are stated explicitly here so the v2 builder can re-derive decisions when the PRD is silent on specifics.

### The thesis

**A pipeline for agentic engineering must be preserve-by-default, adversarial-by-design, and self-hostable under its own failure modes.** The rest is consequences.

**Self-hostability is the fitness test.** A development tool that can't be used to improve itself has an observability or recovery leak somewhere. §5.9 (Recursive failure) documents the specific leak that proved v1 fails this test — the BL-156 bug destroyed its own in-flight fix three times in one session. The whole rewrite exists to make v2 pass the self-hostability test.

### The four principles

**P1 — Silence is cultural, not incidental.**

23 `let _ =` in `cartography/delta_helpers.rs` is not a budget problem — it's a value choice. If you can't observe failure, you can't fix failure; if you can't fix failure, you eventually lose work.

→ v2 bans silent failure at lint level (`clippy::let_underscore_must_use` in workspace roots). Every I/O adapter returns `Result<T, AdapterError>`. Every app-layer consumer handles with `tracing::warn!` + context. See §5.3, §6.2 #8 (clippy config), Appendix C (test strategy — every `tracing::warn!` path covered).

**P2 — Preserve-by-default over clean-up-by-default.**

When signals disagree, keep the work. The cost of a lingering worktree is bounded disk; the cost of a deleted live worktree is unbounded work loss. That asymmetry generalizes — v2 systematically biases toward "keep" over "clean."

→ BL-156 liveness verdict requires ALL four signals (lock + pid + start_time + heartbeat) for delete; any ambiguity preserves. tasks.md is the session handoff contract — no in-memory state crosses session boundaries. Artifacts persist at every pipeline phase. Backward transitions clear timestamps but keep history. Merge cleanup refuses if ANY safety check fails. See §4.3, §6.1 bullet "New in v2" #5, §6.2 #5 (LivenessVerdict).

**P3 — Adversarial review is the pipeline, not a QA bolt-on.**

Every phase has an adversary that can fail it — spec-adversary (7 dimensions, rubric-scored), solution-adversary (8 dimensions), uncle-bob, robert, security-reviewer, pc-evaluator. They can abort, demand rework, escalate. v2 adds a session-budget dimension: specs with >30 PCs fail CONDITIONAL. Adversaries exist to fight the planner's and the user's optimism.

→ §4.2 (pipeline phases with explicit adversary pass thresholds), §6.2 #7 (session-budget dimension), §7 Phase 5 (v2 self-audits with its own adversaries; must produce zero new findings that aren't already on the v2 backlog). Reviews are antagonistic by contract.

**P4 — Bounded cognition as infrastructure, not discipline.**

The pipeline treats the agent (and the human) as a resource with finite capacity. Fix-round budgets (2 auto, escalate to user, hard cap 8). Subagent isolation with fresh context per PC. Wave-based TDD parallelizes independent PCs but serializes merges. Context exhaustion during /implement isn't a planning failure — it's a design failure if the spec was allowed to exist at that size.

→ §5.7 (context exhaustion), §6.2 #7 (session-budget adversary), §6.1 bullet "New in v2" #5 (tasks.md as session handoff contract). v2 makes the budget a first-class adversary concern.

### What this philosophy is NOT

Worth sharpening by contrast:

- **Not "move fast and break things."** v2 moves deliberately — grill-me, Plan Mode, adversary rounds, post-PC self-eval. Every phase is reviewable.
- **Not "trust the AI."** v2 assumes agents fail constantly in specific, observable ways. Fix-round budgets, structured-event emissions (not LLM text matching), property tests at domain level, golden-file tests for parsers.
- **Not "automate everything."** The human stays in the loop via AskUserQuestion at decision points. Auto mode is an escape hatch, not default.
- **Not minimalism.** v2 prunes (121 skills → 60, 35 commands → 22) but keeps aggressive scope. Fewer things, each with sharper edges.

### Crystallized

**Build the tool assuming the tool will be used against itself.**

The load-bearing mechanics — worktree GC, state.json, silent I/O, lock files — must survive adversarial conditions (concurrent sessions, parallel runs, crashes, partial writes, PID reuse, clock skew) because they will be attacked by accident every hour. v1 was built as if they wouldn't. v2 is the atonement.

---

## 3. Feature catalog (what must be reproduced)

### 2.1 The `ecc` CLI binary

28 top-level subcommands. Every subcommand has a corresponding `commands/<name>.rs` file in `ecc-cli`. Global flags: `-v/--verbose` (count: -v info, -vv debug, -vvv trace), `-q/--quiet`.

| Subcommand | Purpose | Critical? |
|------------|---------|-----------|
| `analyze` | Deterministic git-history analysis (hotspots, coupling, bus factor) | Yes |
| `version` | Show binary version | Yes |
| `install` | Install ECC config to `~/.claude/` | Yes |
| `init` | Initialize ECC in current project | Yes |
| `audit` | Audit ECC config health (not codebase audit) | Yes |
| `audit cache check/clear` | Per-domain audit cache (content-hash + TTL) | Yes |
| `completion` | Generate shell completions | Yes |
| `hook <id>` | Run hook by ID (dispatch entry point) | Yes (load-bearing) |
| `validate <target>` | Validate content files (11 targets below) | Yes |
| `claw` | NanoClaw interactive REPL | Medium |
| `dev on/off/switch` | Toggle/switch ECC config | Yes |
| `backlog next-id/check-duplicates/reindex/update-status/migrate/list` | Backlog management | Yes |
| `drift check` | Spec-vs-impl drift detection | Medium |
| `docs update-module-summary/coverage` | Doc generation utilities | Medium |
| `diagram triggers --changed-files` | Diagram update heuristics | Low |
| `commit lint --staged` | Atomic commit concern validation | Yes |
| `bypass grant/list/summary/prune/gc` | Hook bypass token management | Yes |
| `worktree gc/status/...` | Worktree lifecycle (THIS IS THE BROKEN ONE) | Critical fix |
| `sources` | Knowledge sources registry | Medium |
| `log` | Query structured logs | Medium |
| `memory` | Manage memory store | Medium |
| `audit-web` | Web-based upgrade audit profile | Medium |
| `status` | Diagnostic snapshot | Yes |
| `config` | Manage ECC configuration | Yes |
| `update` | Self-update from GitHub Releases | Medium |
| `workflow` | Delegates to ecc-workflow binary | Yes |
| `cost summary/breakdown/compare/export/prune/migrate` | Token cost tracking | Medium |
| `metrics summary/record-gate/export/prune` | Harness reliability metrics | Medium |

**Validate targets** (subcommands of `ecc validate`):
1. `agents` — frontmatter schema, naming, cross-links
2. `commands` — frontmatter schema, naming, reject `$ARGUMENTS` in shell-eval lines (ADR-0066)
3. `conventions` — naming, tool placement, required fields per content type
4. `hooks` — hooks.json schema
5. `skills` — SKILL.md frontmatter, naming, manifest integration
6. `rules` — frontmatter, file naming
7. `paths` — scan for personal paths in shipped files
8. `statusline` — statusline installation (ecc-status-line present + executable)
9. `teams` — team manifest schema
10. `patterns [--fix]` — pattern library frontmatter, regenerate `patterns/index.md`
11. `spec <path>` + `design <path>` — AC format, PC table, coverage mapping
12. `claude-md counts/markers/all` — numeric claims, TEMPORARY markers, strict/warn modes
13. `cartography [--coverage]` — journey/flow staleness, coverage dashboard

### 2.2 The `ecc-workflow` binary

Standalone binary — not under hexagonal architecture (architecturally intentional per ADR-0019). Directly uses `std::fs`, `std::process`, `std::io`. 17 top-level commands + 3 command families.

| Command | Purpose |
|---------|---------|
| `init <concern> [<feature>\|--feature-stdin]` | Initialize workflow state |
| `transition <target> [--artifact <type>] [--path <p>] [--justify <msg>]` | Advance to target phase; `--justify` mandatory for backward transitions (ADR-0064/0065) |
| `history [--json]` | Display transition history |
| `toolchain-persist <test> <lint> <build>` | Persist detected toolchain |
| `memory-write <kind> [args]` | Write memory entries (action, work-item, daily, memory-index) |
| `phase-gate` (stdin JSON) | Gate Write/Edit/MultiEdit during plan/solution phases |
| `stop-gate` | Warn on incomplete workflow at session end (exit 0) |
| `grill-me-gate` | Warn when spec lacks grill-me section (exit 0) |
| `tdd-enforcement` (stdin JSON) | Track RED/GREEN/REFACTOR state (exit 0) |
| `status` | Current phase, feature, artifact paths |
| `artifact <type>` | Resolve and validate artifact path |
| `reset --force` | Reset workflow to idle (destructive, gated) |
| `scope-check` | Compare git diff vs design's File Changes (warn, exit 0) |
| `doc-enforcement` | Require sections in implement-done.md at "done" phase |
| `doc-level-check` | Warn on oversized CLAUDE.md/README.md/ARCHITECTURE.md |
| `pass-condition-check` | Require pass conditions in implement-done.md |
| `e2e-boundary-check` | Require E2E Tests section at "done" |
| `worktree-name <concern> [<feature>\|--feature-stdin]` | Generate session-isolated worktree name |
| `wave-plan <design_path>` | Compute wave plan from design's PC + File Changes tables |
| `merge` | Rebase + verify + ff-merge session worktree into main |
| `backlog add-entry <title> [...]` | Atomic backlog add with flock |
| `tasks sync/update/init` | Task status synchronization |
| `campaign init/append-decision/show` | Campaign manifest management |
| `recover` | Archive state + reset to idle |

### 2.3 22 port traits (`ecc-ports`)

Every production I/O goes through one of these. v2 adds `FileSystem::rename`, `FileSystem::touch` (uncle-bob CRITICAL-1 from BL-156 design review).

| Trait | Cardinal methods | Current impl |
|-------|------------------|--------------|
| FileSystem | read/write/exists/is_dir/is_file/create_dir_all/remove_file/remove_dir_all/copy/read_dir/read_dir_recursive/symlink ops/permissions/rename | OsFileSystem |
| ShellExecutor | run_command/run_command_in_dir/command_exists/spawn_with_stdin | ProcessExecutor |
| TerminalIO | stdout_write/stderr_write/prompt/is_tty/terminal_width | StdTerminal |
| Environment | var/home_dir/current_dir/temp_dir/platform/architecture/current_exe | OsEnvironment |
| Clock | now/now_timestamp | SystemClock |
| GitInfo | git_dir(working_dir) | OsGitInfo |
| GitLogPort | log_oneline/log_json/log_stats/diff_summary | GitLogAdapter |
| WorktreeManager | create/list/remove/prune/current/branch | OsWorktreeManager |
| BypassStore | grant/query/summary/delete | SqliteBypassStore |
| CacheStore | set/get/delete/clear/exists | FileCacheStore |
| ConfigStore | load/save/delete | FileConfigStore |
| CostStore | append/query/summary/prune/export | SqliteCostStore |
| LogStore | append/query/delete | SqliteLogStore |
| MemoryStore | write/read/list/delete/clear | SqliteMemoryStore |
| MetricsStore | record/query/summary/delete/export | SqliteMetricsStore |
| BacklogEntryStore | save/load/delete/list_all | FsBacklogRepository |
| BacklogLockStore | lock/unlock/is_locked/list_locks | FsBacklogRepository (same struct) |
| BacklogIndexStore | save_index/load_index/clear | FsBacklogRepository |
| FileLock | acquire/release/is_locked | FlockLock (Unix only) |
| TarballExtractor | extract(path, dest) | TarExtractor |
| ReleaseClient | list_releases/download_asset/latest_tag | GitHubReleaseClient |
| ReplInput | readline/add_history | RustylineInput |

### 2.4 67 agents

Grouped by role. v2 will audit for consolidation; likely ~50 agents post-consolidation. Full list with model + effort in Appendix A.

| Category | Count | Examples |
|----------|-------|----------|
| Architecture | 3 | arch-reviewer, architect, architect-module |
| Audit | 8 | audit-orchestrator, component-auditor, evolution-analyst, test-auditor |
| Code review (language-specific) | 10 | rust-reviewer, typescript-reviewer, python-reviewer, go-reviewer, kotlin-reviewer, java-reviewer, csharp-reviewer, cpp-reviewer, shell-reviewer, code-reviewer (generic) |
| Documentation / cartography | 11 | doc-analyzer, doc-generator, cartographer, diagram-generator, compass-context-writer |
| Build / error resolution | 4 | build-error-resolver, go-build-resolver, kotlin-build-resolver, drift-checker |
| Specialized | 5 | database-reviewer, design-reviewer, e2e-runner, interface-designer, qa-strategist |
| Planning / orchestration | 5 | planner, party-coordinator, doc-orchestrator, requirements-analyst, backlog-curator |
| Adversarial / review | 5 | solution-adversary, spec-adversary, uncle-bob, robert, interviewer |
| Supporting / meta | 11 | harness-optimizer, tdd-executor, tdd-guide, pc-evaluator, security-reviewer, web-scout, web-radar-analyst, comms-generator, etc. |
| Domain (BMAD) | 5 | bmad-architect, bmad-dev, bmad-pm, bmad-qa, bmad-security |

Model distribution: 34 sonnet, 21 opus, 12 haiku. Effort distribution: 33 medium, 14 high, 12 low, 8 max.

### 2.5 35 commands

Grouped by family. v2 will consolidate — several are thin wrappers.

| Family | Commands |
|--------|----------|
| Spec pipeline | spec, spec-dev, spec-fix, spec-refactor |
| Core pipeline | design, implement, verify, commit |
| Audit family (13) | audit-full, audit-archi, audit-backlog, audit-code, audit-convention, audit-doc, audit-errors, audit-evolution, audit-observability, audit-security, audit-test, audit-web, ecc-test-mode |
| Communications | comms, comms-plan, comms-generate |
| Doc pipeline | doc-suite |
| Backlog & planning | backlog, project-foundation |
| Scaffolding | create-component, scaffold-workflows |
| Party / testing | party, mutants, generate-domain-agents |
| Meta | catchup, build-fix, review |

### 2.6 121 skills

Too many. Full list in Appendix A. Categories:

| Category | Count |
|----------|-------|
| Language-patterns (rust/python/kotlin/java/swift/…) | 32 |
| Testing (per language) | 14 |
| Doc-generation | 12 |
| Audit methodology | 12 |
| Workflow meta | 11 |
| Methodology / misc | ~20 |
| Doc analysis | 7 |
| Code quality | 6 |
| Infrastructure | 6 |
| Pipeline shared | 3 |
| Doc guidelines | 3 |

Top-3 by word count: grill-me (1927), configure-ecc (1884), claude-workspace-optimization (1762). Tiniest: github-actions (37, already superseded), continuous-agent-loop (152), enterprise-agent-ops (171).

**v2 target**: ~60 skills. Preserve methodology + pipeline-shared + audit-methodology; prune half of language-patterns (let the agent Read the actual code instead); consolidate overlapping doc-generation skills.

### 2.7 80 rule files

`rules/common/` (~6 files) + 9 language directories (rust/python/go/typescript/kotlin/java/php/perl/shell) each with 5 files (coding-style, testing, patterns, hooks, security). Plus `rules/ecc/` (dev conventions), README index, performance.md, e2e-testing.md, github-actions.md.

### 2.8 3 teams

Manifest files in `teams/`:
- `implement-team.md` — agents + max-concurrent for /implement dispatch
- `audit-team.md` — parallel audit dispatch roster
- `review-team.md` — code-review language roster

### 2.9 9 registered hooks

From `.claude/settings.json`:

| Hook | Event | Handler | Purpose |
|------|-------|---------|---------|
| phase-gate | PreToolUse (Write\|Edit\|Bash) | `ecc-workflow phase-gate` | Block source writes during plan/solution phases |
| tdd-enforcement | PostToolUse | `ecc-workflow tdd-enforcement` | Track RED/GREEN/REFACTOR state |
| stop-gate | Stop | `ecc-workflow stop-gate` | Warn on incomplete workflow |
| doc-enforcement | Stop | `ecc-workflow doc-enforcement` | Warn missing docs |
| pass-condition-check | Stop | `ecc-workflow pass-condition-check` | Warn missing PC results |
| e2e-boundary-check | Stop | `ecc-workflow e2e-boundary-check` | Warn missing E2E coverage |
| scope-check | Stop | `ecc-workflow scope-check` | Warn scope creep |
| doc-level-check | Stop | `ecc-workflow doc-level-check` | Warn oversized docs |
| grill-me-gate | Stop | `ecc-workflow grill-me-gate` | Warn incomplete grill-me |

Plus `.claude/hooks/` tiered handlers (~50 handlers across tier1_simple, tier2_tools/notify, tier3_session).

### 2.10 5 workflow templates

`workflow-templates/`:
- claude-pr-review.yml
- claude-pr-review-fork-safe.yml
- claude-issue-triage.yml
- claude-release-notes.yml
- claude-ci-linter.yml

### 2.11 Statusline + memory + cartography

- **Statusline**: `statusline/statusline-command.sh` + `context-persist.sh`. 22 Bats tests. Renders worktree, phase, workflow concern, token cost.
- **Memory**: SQLite at `~/.ecc/memory/memory.db`. Three tiers (semantic/episodic/working). Auto-consolidation. Daily files.
- **Cartography**: `docs/cartography/{journeys,flows,elements}/`. Two-tier (element gen + index regen). Write-time noise filter.
- **Observability**: tracing + JSON rolling files + SQLite index. `~/.ecc/logs/*.jsonl` + `logs.db`.

---

## 4. Architecture blueprint (what survives into v2)

### 3.1 9-crate hexagonal layout — keep intact

```
┌──────────────────────────────────────────────────────────┐
│ CLI Layer (ecc-cli)                                       │
│ 28 subcommands, thin wiring of infra→app                  │
├──────────────────────────────────────────────────────────┤
│ App Layer (ecc-app)                                       │
│ Use cases, hook dispatch, validation, orchestration       │
├──────────────────────────────────────────────────────────┤
│ Domain Layer (ecc-domain) ← PURE (enforced by hook)       │
│ 21 bounded contexts, zero std::fs/std::process/tokio      │
├──────────────────────────────────────────────────────────┤
│ Ports Layer (ecc-ports) ← TRAITS ONLY                     │
│ 22 traits (growing to 24 with rename+touch in v2)         │
├──────────────────────────────────────────────────────────┤
│ Infra Layer (ecc-infra) ← Adapters                        │
│ 10 core adapters + 5 SQLite stores + auxiliaries          │
├──────────────────────────────────────────────────────────┤
│ ecc-test-support — InMemory doubles for every port        │
├──────────────────────────────────────────────────────────┤
│ ecc-workflow — Standalone binary (not hexagonal)          │
│ DECOMPOSITION REQUIRED IN V2: phase_gate.rs 959 LOC etc   │
├──────────────────────────────────────────────────────────┤
│ ecc-flock — POSIX flock RAII wrapper (keep as-is)         │
├──────────────────────────────────────────────────────────┤
│ ecc-integration-tests — Workspace integration harness     │
└──────────────────────────────────────────────────────────┘
```

**Dependency direction** (enforced):
- CLI → App → (Ports ← Infra) + Domain
- Ports → Domain
- Domain → (nothing)
- ecc-workflow → (nothing; standalone by design)

**Enforcement**: pre-write hook blocks `use std::fs`, `use std::process`, `use tokio`, `use reqwest` in `crates/ecc-domain/src/`. v2 extends to block same imports in `crates/ecc-app/src/` for any file that doesn't explicitly opt-in (via `#[allow(clippy::disallowed_types)]` comment referencing a specific exemption).

### 3.2 Spec-driven pipeline — keep intact, fix overreach

```
User input (slash command or backlog item)
  ↓
/spec (router → /spec-dev | /spec-fix | /spec-refactor)
  ├─ Phase 0: project detect, worktree enter, workflow init
  ├─ Phase 1-4: requirements-analyst → architect → web research → audit/backlog scan
  ├─ Phase 5: grill-me interview (AskUserQuestion, one-at-a-time, ≤8 questions)
  ├─ Phase 6: Plan Mode (doc-first review)
  ├─ Phase 7: write spec
  ├─ Phase 8: spec-adversary (max 3 rounds; CONDITIONAL→refine, FAIL→restart)
  └─ Phase 9: persist spec.md, transition plan→solution, STOP
  ↓
/design
  ├─ Phase 0-6: planner → uncle-bob → robert → security → E2E boundaries → doc plan → AC coverage
  ├─ Phase 7: Plan Mode (architecture preview: ARCHITECTURE.md changes + Mermaid + bounded contexts)
  ├─ Phase 8: write solution
  ├─ Phase 9: solution-adversary (max 3 rounds, 8 dimensions)
  └─ Phase 10: persist design.md, transition solution→implement, STOP
  ↓
/implement
  ├─ Phase 0-2: state validation, parse solution, generate tasks.md + waves
  ├─ Phase 3: TDD loop (tdd-executor subagent per PC, RED→GREEN→REFACTOR, atomic commits)
  │  ├─ Fix-round budget (2 auto, escalate to user, hard cap 8)
  │  └─ Post-PC self-evaluation (conditional: fix_rounds>0, integration/e2e, wave boundary)
  ├─ Phase 4: E2E tests (un-ignore activated tests)
  ├─ Phase 5: code-reviewer pass
  ├─ Phase 6: doc updates (CHANGELOG mandatory, ADRs for every "ADR Needed? Yes")
  ├─ Phase 7.5: supplemental docs (module-summary-updater, diagram-updater, compass-context-writer in parallel)
  ├─ Phase 7: write implement-done.md
  └─ Phase 8: ecc-workflow merge (rebase + verify + ff-only + cleanup), transition implement→done
```

**Adversaries pass thresholds:**
- spec-adversary: avg ≥ 70 AND no dimension < 50 → PASS; else CONDITIONAL (with suggested ACs) or FAIL (restart grill-me). 3-round cap. 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback.
- solution-adversary: same thresholds. 8 dimensions: coverage, order, fragility, rollback, architecture, blast radius, missing PCs, doc plan.

**State machine transitions** (`ecc-workflow`):
- Forward: `idle → plan → solution → implement → done`
- Backward (with `--justify`): `implement → solution`, `solution → plan`, `implement → plan`
- All transitions logged to `state.json.history[]` as TransitionRecord.

### 3.3 Session safety — rebuild the load-bearing piece

v1 has three mechanisms:
1. **Per-session worktrees** — `EnterWorktree` creates `.claude/worktrees/ecc-session-<timestamp>-<concern>-<feature>`
2. **Per-worktree state** — `state.json` resolves to `<git-common-dir>/ecc-workflow/state.json` via `.state-dir` anchor file
3. **POSIX flock locks** — `ecc-flock` crate wraps advisory locks for shared-state ops (backlog, memory, merge)

**What breaks in v1:**
- `ecc worktree gc` deletes live session worktrees because the PID-from-worktree-name is the init-process PID (dead on arrival). Heuristic is `(age > 24h OR pid_dead) AND modified > 30min` — `pid_dead == true` constant + 30min window → live sessions get deleted.
- state.json gets clobbered across sessions because another session's `ecc-workflow init` overwrites the file for the session that owned it.
- `.state-dir` anchor file is subject to resolution drift when hook subprocesses resolve CWD incorrectly.

**What v2 adopts** (from BL-156 design PASS 80/100):
- Per-worktree `.ecc-session.lock` file with `{pid, start_time (ps -o lstart=), created_at_secs, schema_version=1}` (2 KiB cap, mode 0600, `st_uid == geteuid()` check)
- Per-worktree `.ecc-session.heartbeat` file (mtime touched on every post-tool hook)
- Pure-domain `assess_session_liveness(input) -> LivenessVerdict` with 6 variants: Alive, StaleDeadPid, StaleReusedPid, StaleMissingLock, StaleSchemaVersion, StaleMalformed
- Liveness rule: `alive = lock_exists AND pid_alive AND start_time_match AND heartbeat_age < 5 × HEARTBEAT_INTERVAL (150s)`
- `kill(pid, 0)`: EPERM→alive, ESRCH→dead
- Fail-safe bias: all I/O errors default to "preserve worktree"
- Single `--force-kill-live` flag (not 4-flag combo) + `ECC_ACK_DATA_LOSS=1` env for non-TTY
- Shared helper `is_locked_by_live_session(mgr, clock, path)` called from BOTH gc and status (single source of truth)

**Additional v2 requirement**: state.json should be worktree-scoped at birth (no anchor file complexity). Every command that touches state resolves to `<git-common-dir>/ecc-workflow/<worktree-name>/state.json`, computed from `git rev-parse --show-toplevel` at every call — no cached path, no anchor file.

---

## 5. Pain points (what v2 must fix)

### 4.1 BL-156: worktree GC deletes live sessions — **#1 correctness goal**

Already captured. The fix is fully designed (see §4.3 and the BL-156 design file in v1 history). v2 ships this from day one instead of adding it as a retrofit.

### 4.2 BL-133: Clock port bypass (11 call sites, growing)

`SystemTime::now()` directly called in `ecc-app` bypasses the `Clock` port. Audit 2026-04-09 found 1 call site; audit 2026-04-18 found 11. Classic viscosity — wrong path easier than right path.

**v2 fix**:
- Clippy lint `disallowed_methods` configured to ban `std::time::SystemTime::now`, `std::time::UNIX_EPOCH`, `std::time::Instant::now` in `ecc-app/src/**` and `ecc-domain/src/**`
- Expose `Clock` as a field on every orchestration struct; no hidden `SystemTime::now()` in helpers
- `MockClock` is the only way to construct time in tests (no real sleeps, no time-dependent flakes)

### 4.3 Silent failures in cartography (23 `let _ =` instances)

`crates/ecc-app/src/cartography/delta_helpers.rs` swallows 23 I/O errors with `let _ =`. Downstream: scaffold creation fails invisibly.

**v2 fix**:
- Workspace-level clippy lint `let_underscore_must_use` at error level. Every `Result` must be handled explicitly.
- grep hook in CI that blocks any PR introducing `let _ =` outside `#[cfg(test)]` code
- Every I/O adapter returns a `Result<T, AdapterError>` — app code handles the error with `tracing::warn!` + context, never drops silently

### 4.4 Oversized files (13 files > 800 LOC)

Top offenders: `phase_gate.rs` 959, `transition.rs` 848, `backlog.rs` 1467. `ecc-workflow` has 5 files >500 LOC — the crate operates outside hex architecture and has accumulated complexity.

**v2 fix**:
- `ecc-workflow` decomposed into subdirectory modules at birth: `phase_gate/`, `transition/`, `merge/`, `backlog/`, `tasks/`, `campaign/` each with submodules ≤200 LOC
- Pre-commit hook blocks PRs that add files >800 LOC (soft: warn at 600, hard block at 800)

### 4.5 State.json clobbering across concurrent sessions

Session A runs `ecc-workflow init dev ...`; session B runs `ecc-workflow init fix ...`. B's state overwrites A's. We hit this four times during the session that wrote this document.

**v2 fix**:
- state.json path computed at every call from `(git_common_dir, current_worktree_name)`
- `ecc-workflow init` refuses to run if a state.json exists for the current worktree (require `ecc-workflow reset --force` first)
- Every command snapshots the relevant state.json fields at entry, then validates invariants at exit — stale state detected immediately

### 4.6 34 commands / 121 skills — surface too large

Many are thin variants or near-duplicates:
- `ecc-test-mode` is effectively `audit-full` with a different label
- `review` is a thin shim over `code-reviewer`
- `build-fix` is a thin shim over `build-error-resolver`
- 12 audit-* commands share 80% of their phases
- Per-language skills (rust-testing, python-testing, kotlin-testing, …) are mostly "use the language's native test framework"

**v2 fix**:
- Consolidate audit-* commands into `audit --domain <domain>` with a domain registry
- Delete thin shim commands; keep the underlying agent
- Prune language-pattern skills that don't encode ECC-specific judgment (the agent can Read the rules file directly)
- Target: ~22 commands, ~60 skills

### 4.7 Context exhaustion during /implement

BL-157 spec had 85 PCs; BL-156 spec had 75 PCs. Neither completes in a single Claude Code session. /implement routinely context-exhausts around Layer 3.

**v2 fix**:
- Spec-adversary adds a "session-budget" dimension — any spec with >30 PCs or >15 file changes fails with CONDITIONAL, recommending a split
- /design produces a phase plan with explicit session breakpoints — "implement Layer 0 in session 1, resume in session 2"
- tasks.md becomes the session handoff contract — any session can resume from "first incomplete PC"

### 4.8 Bus factor — solo dev owns 85.7% of commits

Documented risk. v2 doesn't solve this by itself, but it MUST:
- Ship complete onboarding docs at day one (not retrofitted)
- Ship a working local dev loop in under 5 minutes from `git clone`
- Have every non-trivial module with a `docs/context/<module>.md` compass file (≤35 lines)

### 4.9 Recursive failure: the bug that ate its own fix

Meta-observation: during this session, BL-156 was spec'd (PASS 76/100) and designed (PASS 80/100), then the worktree was destroyed — three times total this session. v1 cannot be self-fixed with its own pipeline while BL-156 is live.

**v2 starts from a repo where BL-156 is already fixed.** That's the whole point of the clean-room rewrite. It must not have the failure mode that prevents its own evolution.

---

## 6. v2 direction (what goes further)

### 5.1 Design principles (inherited vs new)

**Inherited from v1 (keep):**
- Hexagonal architecture, strict dependency direction
- Domain purity (zero I/O)
- Port/adapter pattern with trait-based DI
- Per-module error enums with thiserror
- No `.unwrap()` / `.expect()` in production
- Newtype wrappers for domain primitives
- serde with `deny_unknown_fields` on inbound
- POSIX flock for session-level concurrency
- Per-session git worktrees
- Doc-first spec pipeline with adversarial review
- TDD with property tests on domain invariants
- ADR discipline (Context/Decision/Consequences/Alternatives)
- CLAUDE.md stays reductive, ARCHITECTURE.md carries intent, rules/ carry style

**New in v2:**
1. **Session liveness is load-bearing.** Lock file + heartbeat + PID + start-time match. All four signals AND'd. Preserve-on-ambiguity bias.
2. **Silent failure is banned at lint level.** No `let _ =` on Result outside tests. CI-enforced.
3. **State.json is worktree-path-computed, not anchor-cached.** Every call resolves path freshly.
4. **Session budget is a spec concern.** Specs with >30 PCs fail adversary.
5. **tasks.md is the session handoff contract.** Every session can resume from "first incomplete PC". No in-memory state crosses session boundaries.
6. **Observability is default-on.** Every I/O failure reaches `tracing::warn!` at minimum; `let _ = ` is a ban.
7. **Disabled concurrency during init.** `ecc-workflow init` on a worktree with existing state.json refuses without `--force-reset`.
8. **File size is a pre-commit gate.** >800 LOC blocks; >600 LOC warns.

### 5.2 Structural fixes (specific mechanics)

1. **`ecc-flock` + session lock**: extend `ecc-flock` with `SessionLockFile` value type. Domain owns schema, infra owns I/O. Integrate from day one — no hook writes a lock after v1's "hook model prevents it" lesson.
2. **Clock port threading**: every orchestration struct takes `&dyn Clock`. Clippy lint bans direct `SystemTime::now()`. Hook handlers use a passed-in clock, never create one.
3. **FileSystem port extensions**: `rename(from, to)`, `touch(path)`, `chmod(path, mode)`. Every session-file operation goes through these.
4. **`ecc-workflow` decomposed**: each top-level command is a subdirectory. Phase-gate, transition, merge, backlog, tasks, campaign each own their submodules.
5. **LivenessVerdict as domain type**: 6-variant enum with Display impl for one-place display mapping. `decide_gc_action(verdict, flags) -> GcDecision` pure function consumed by gc + status + merge.
6. **State.json worktree-scoped**: `resolve_state_path(git_common_dir, worktree_name) -> PathBuf`. No anchor file.
7. **Adversary session-budget dimension**: spec-adversary fails CONDITIONAL if PC count > 30 OR file-change count > 15.
8. **Clippy workspace config**: `#![deny(clippy::let_underscore_must_use, clippy::disallowed_methods)]` in ecc-domain and ecc-app roots.

### 5.3 New capabilities worth shipping at day one

Not v1 retrofits — things v1 doesn't do well that v2 should start with:

1. **First-class session browser** — `ecc session list/show/resume/archive`. Every session's artifacts (spec, design, tasks, implement-done, commits, adversary reports, grill-me decisions) discoverable from one command.
2. **Cross-session debt index** — replaces BL-157 concept. `ecc debt list/show/link-backlog`. Debt captured by adversaries (uncle-bob, robert, security-reviewer, pc-evaluator, solution-adversary) during /spec, /design, /implement; persisted to `docs/debt/DEBT-NNN-<slug>.md`.
3. **Reproducible session record** — every `/spec`, `/design`, `/implement` invocation recorded to `~/.ecc/sessions/<session-id>/` with full inputs, agent outputs, commits, and state snapshots. Replayable with `ecc session replay <id>` for incident investigation.
4. **Hook-level observability** — every hook invocation logs to structured log + metrics. `ecc metrics harness` produces 4 SLO reports: hook success rate (≥99%), phase-gate violation rate (≤5%), agent recovery rate (≥80%), commit atomicity score (≥95%).
5. **Local LLM fallback built-in** — Ollama optional but pre-wired. Mechanical agents (doc-generator, diagram-generator, cartography-flow-generator) auto-offload to local model when available. Flag-gated at config level.
6. **Daemon-free heartbeat** — sessions send heartbeats via `post-tool` hook touches. No background process. v1's design deliberately avoids a daemon; v2 sticks with this.
7. **`ecc verify --quick`** — pre-commit equivalent of `cargo check + cargo test --lib + cargo clippy -- -W`. Under 30 seconds on a laptop. Separate from `/verify` which runs the full gate.

### 5.4 What v2 explicitly does NOT do

- **No move off Rust.** User directive.
- **No move to a daemon model.** Hooks + tasks.md = session handoff contract.
- **No remote workers / distributed execution.** Single-machine.
- **No real-time collaboration.** Sessions are serial per worktree, parallel across worktrees.
- **No hidden state.** If a command depends on a file, that file's path is in its output.
- **No CLAUDE.md bloat.** CLAUDE.md stays under 200 lines. ARCHITECTURE.md carries architecture intent; rules/ carry style; `docs/context/<component>.md` compass files carry module-specific tribal knowledge.

---

## 7. Build plan (phased roadmap)

Ship v2 in **5 phases**, each PR-mergeable and independently useful. Estimated effort in parens assumes one full-time developer; scale for part-time.

### Phase 0 — Bootstrap (1 week)

**Goal**: empty → compilable skeleton with CI green.

- [ ] `cargo new --lib ecc-v2`
- [ ] Workspace Cargo.toml with 9 crate scaffolds (ecc-domain, ecc-ports, ecc-app, ecc-infra, ecc-cli, ecc-workflow, ecc-test-support, ecc-flock, ecc-integration-tests)
- [ ] Edition 2024, pinned Rust toolchain
- [ ] dist.toml from v1, adjusted for new crate names
- [ ] CI: GitHub Actions (ci.yml, cd.yml, release.yml, maintenance.yml) — ported from v1 workflow-templates
- [ ] Workspace lints: `#![deny(clippy::let_underscore_must_use, clippy::disallowed_methods)]`
- [ ] `clippy.toml` with disallowed_methods for SystemTime::now in app/domain
- [ ] Pre-commit hook: block files >800 LOC, warn at 600
- [ ] `cargo deny`, `cargo vet`, `cargo audit` in CI

**Ship criteria**: `cargo test --workspace` passes (zero tests), `cargo clippy` passes, CI badges green.

### Phase 1 — Core infrastructure (3-4 weeks)

**Goal**: hex architecture working end-to-end with `ecc --version` and `ecc status`.

- [ ] `ecc-domain`: copy-port 9 bounded contexts verbatim from v1, drop anything flagged as debt (Clock bypasses, swallowed errors in cartography)
- [ ] `ecc-ports`: all 22 traits + 2 new (FileSystem::rename, FileSystem::touch, FileSystem::chmod, WorktreeManager::probe_session_liveness)
- [ ] `ecc-infra`: OsFileSystem, ProcessExecutor, StdTerminal, OsEnvironment, SystemClock, OsGitInfo, OsWorktreeManager, FlockLock. SQLite stores come later.
- [ ] `ecc-test-support`: InMemory doubles for every port, including MockWorktreeManager and MockFileSystem with rename/touch
- [ ] `ecc-cli`: `ecc --version`, `ecc status` — wiring pattern proven
- [ ] `ecc-workflow`: `ecc-workflow status`, `ecc-workflow init --concern dev --feature-stdin` — minimum viable state machine
- [ ] **BL-156 shipped from day one**: Session lock, heartbeat, LivenessVerdict, assess_session_liveness, decide_gc_action. `ecc worktree gc` fully correct against 6 E2E scenarios.
- [ ] Multi-process concurrency tests in `ecc-integration-tests` covering BL-156 scenarios (concurrent live, crashed, PID reuse, orphan, merge race, heartbeat stale).

**Ship criteria**: All 22 port traits + 2 new implemented and tested. BL-156's 6 E2E scenarios green. `ecc --version` + `ecc status` + `ecc worktree gc` + `ecc-workflow status` + `ecc-workflow init` functional.

### Phase 2 — Spec pipeline + hooks (4-6 weeks)

**Goal**: `/spec`, `/design`, `/implement` commands work end-to-end. Hooks enforce gates.

- [ ] `/spec` → `/spec-dev` | `/spec-fix` | `/spec-refactor` router + all three spec commands
- [ ] `/design` full implementation
- [ ] `/implement` full implementation, including TDD executor, wave dispatch, fix-round budget, post-PC self-eval, tasks.md as session handoff contract
- [ ] `/verify`, `/commit`, `/audit-full`
- [ ] Hook system: 9 registered hooks + tiered handlers (tier1_simple, tier2_tools, tier3_session)
- [ ] Adversaries: spec-adversary, solution-adversary, uncle-bob, robert, security-reviewer, design-reviewer, pc-evaluator
- [ ] `ecc-workflow` phase-gate, transition (forward + backward with justify), merge, backlog, tasks, campaign
- [ ] Worktree auto-merge at session end
- [ ] Backlog CRUD (`ecc backlog next-id/check-duplicates/reindex/update-status/list`)
- [ ] Validation CLI (`ecc validate agents/commands/hooks/skills/rules/claude-md/...`)
- [ ] **Session-budget adversary dimension**: spec-adversary fails CONDITIONAL if PCs > 30 or file-changes > 15

**Ship criteria**: A full pipeline execution from `/spec` to `/implement` to `ecc-workflow merge` succeeds end-to-end on a trivial feature. State machine transitions + backward transitions work. Worktree auto-cleanup at session end.

### Phase 3 — Prompt ecosystem (2-3 weeks)

**Goal**: agents + skills + rules + teams in parity with v1, pruned.

- [ ] Audit v1's 67 agents, port ~50 (consolidate thin shims, remove legacy)
- [ ] Audit v1's 121 skills, port ~60 (remove language-patterns the agent can infer from rules; consolidate overlapping doc-generation skills)
- [ ] Port `rules/common/` (~6 files) + 9 language directories
- [ ] Port `rules/ecc/` (development conventions)
- [ ] Port 3 team manifests
- [ ] Port 5 workflow templates (`workflow-templates/`)
- [ ] Add `ecc-help` skill updated for v2

**Ship criteria**: `ecc validate agents` + `ecc validate commands` + `ecc validate skills` + `ecc validate hooks` + `ecc validate rules` + `ecc validate teams` all pass.

### Phase 4 — Persistence + observability (2-3 weeks)

**Goal**: SQLite stores, tracing, metrics, memory, cartography, cost.

- [ ] `ecc-infra`: SqliteLogStore, SqliteCostStore, SqliteMetricsStore, SqliteMemoryStore, SqliteBypassStore
- [ ] Tracing config with rolling JSON file writer + SQLite index
- [ ] `ecc log query` / `ecc memory` / `ecc cost` / `ecc metrics` CLI
- [ ] Cartography: journeys, flows, elements, delta processing
- [ ] `/doc-suite` command + cartography-processing skill
- [ ] Compass-context-writer: `docs/context/<component>.md` files
- [ ] Statusline + context-persist shell scripts + 22 Bats tests
- [ ] `ecc session list/show/resume/archive` (v2's new capability #1 from §6.3)
- [ ] `docs/debt/` registry (v2's new capability #2)

**Ship criteria**: Session replay works. Hook telemetry produces 4 SLO reports. Memory consolidation runs daily. Cartography journeys render correctly.

### Phase 5 — Parity + production readiness (2-3 weeks)

**Goal**: 1.0 release. Self-hostable.

- [ ] `ecc install` + `ecc init` (bootstrap ECC v2 into a fresh repo)
- [ ] `ecc update` (self-update from GitHub Releases)
- [ ] Release pipeline: cargo-dist config, SLSA attestations, cosign signing
- [ ] Complete doc site: README, ARCHITECTURE.md, getting-started.md, 67 ADRs audited, commands-reference.md, MODULE-SUMMARIES.md
- [ ] Migration guide for v1 users (workspace paths, state.json format, lock file upgrades)
- [ ] `/audit-full` + 12 domain audits + audit challenger
- [ ] `ecc session replay <id>` for reproducible session record
- [ ] `/party` multi-agent round-table (BMAD-style, 5 domain agents)
- [ ] `/project-foundation` scaffolding
- [ ] Final round: full adversary review on the v2 codebase by v2's own agents. Must produce no findings that aren't documented in the v2 backlog.

**Ship criteria**: v2 self-hosts — its own development uses its own `/spec`, `/design`, `/implement` pipeline. Grade A audit (0 CRITICAL, 0 HIGH, ≤3 MEDIUM).

### Timeline summary

| Phase | Target weeks | Cumulative |
|-------|--------------|------------|
| 0 | 1 | 1 |
| 1 | 3-4 | 4-5 |
| 2 | 4-6 | 8-11 |
| 3 | 2-3 | 10-14 |
| 4 | 2-3 | 12-17 |
| 5 | 2-3 | 14-20 |

**Aggressive estimate: 14 weeks. Realistic estimate: 20 weeks (5 months full-time).** Part-time (10 hrs/week) → ~12 months.

---

## 8. Non-goals

- **No drop-in backwards compatibility with v1 data files.** Migration guide will explain how to port state.json, backlog files, memory DB. But the v2 formats are allowed to change where improvement justifies it.
- **No Windows first-class support** during Phases 0-4. Ships Linux + macOS. Windows via degraded "graceful fallback" path (no `ps -o lstart=`, falls back to 24h age heuristic). Full Windows support is a Phase 5+ concern.
- **No plugin system.** v1 doesn't have one, v2 doesn't need one. Agents + skills + commands ARE the plugin system.
- **No GUI / web UI.** CLI + slash commands + statusline only.
- **No remote-state backend.** File-based + SQLite only. Cloud sync is an out-of-band concern (git push + GitHub Releases).
- **No scheduler / cron primitive in v2 core.** Claude Code has triggers; v2 uses those.
- **No auto-generation of the full v2 codebase by v1's /implement.** v1's pipeline is broken for specs >30 PCs. v2 must be hand-built (human + Claude in interactive mode, not full pipeline).

---

## 9. Open questions (pending decisions)

| # | Question | Suggested default | Decide by |
|---|----------|-------------------|-----------|
| 1 | Keep `ecc-workflow` as a standalone binary, or absorb into `ecc`? | Keep separate (ADR-0019 rationale stands) | Phase 0 |
| 2 | Keep SQLite for all stores, or move to sled/redb for some? | SQLite (proven, rusqlite ecosystem) | Phase 4 |
| 3 | Support multiple adversary agents in parallel (separate AskUserQuestion tabs), or one at a time? | Sequential (current behavior) | Phase 2 |
| 4 | Retain the "audit-* × 12" command family, or consolidate under `audit --domain <domain>`? | Consolidate | Phase 3 |
| 5 | Keep BMAD-style party-coordinator agent or drop as over-engineered? | Keep (BL-144 shipped; useful for large specs) | Phase 5 |
| 6 | 121 skills → 60 skills pruning list — which get cut? | See Appendix A.3 | Phase 3 |
| 7 | Session-budget adversary threshold: 30 PCs? 20? 50? | Start at 30, tune after 5 specs | Phase 2 |
| 8 | Include auto-split of oversized specs, or just reject? | Reject + recommend (user runs /spec again) | Phase 2 |
| 9 | Replace shell-script statusline with Rust binary? | Keep shell (low blast radius; Bats tests prove it) | Phase 4 |
| 10 | Ship v2 as new repo, or branch off v1? | New repo. Clean slate is the whole point. | Phase 0 |

---

## Appendix A: Feature migration table

### A.1 Agents (67 v1 → ~50 v2)

Listed alphabetically. Keep / consolidate / drop recommendation per agent.

**Keep (high confidence):** arch-reviewer, architect, architect-module, audit-orchestrator, audit-challenger, backlog-curator, build-error-resolver, cartographer, code-reviewer, compass-context-writer, component-auditor, convention-auditor, design-reviewer, diagram-generator, doc-analyzer, doc-generator, doc-orchestrator, doc-validator, drift-checker, e2e-runner, error-handling-auditor, evolution-analyst, harness-optimizer, interviewer, module-summary-updater, observability-auditor, party-coordinator, pc-evaluator, planner, qa-strategist, refactor-cleaner, requirements-analyst, robert, security-reviewer, solution-adversary, spec-adversary, tdd-executor, tdd-guide, test-auditor, uncle-bob, web-radar-analyst, web-scout, comms-generator, interface-designer

**Consolidate** (merge pairs/triples):
- cartography-element-generator + cartography-flow-generator + cartography-journey-generator → single cartographer with mode flag
- diagram-generator + diagram-updater → single agent with diff-mode flag
- doc-reporter + doc-updater → fold into doc-generator
- go-build-resolver + kotlin-build-resolver → fold into build-error-resolver (language is a param)
- language-reviewers (10 agents: cpp, csharp, go, java, kotlin, python, rust, shell, typescript, php-reviewer, perl-reviewer) → keep rust-reviewer + typescript-reviewer + python-reviewer (most common); others are thin frontmatter variants, drop in favor of `code-reviewer --language <lang>`

**Drop** (legacy / redundant):
- bmad-* agents (5) — the /party command uses them but they're thin personas; fold into a single `bmad --role <role>` agent spec
- database-reviewer — fold into security-reviewer + code-reviewer
- tdd-guide (kept as skill, not separate agent)

Net: 67 → 48-50 agents.

### A.2 Commands (35 v1 → ~22 v2)

**Keep:** spec, spec-dev, spec-fix, spec-refactor, design, implement, verify, commit, backlog, project-foundation, create-component, scaffold-workflows, doc-suite, catchup, review, build-fix, party

**Consolidate:**
- audit-* (12 commands) → `audit --domain <domain>` with domain registry. Single command.
- comms-plan + comms-generate + comms → single `comms <subcommand>` command.

**Drop:**
- ecc-test-mode → collapse into `audit`
- mutants → `ecc mutants` (CLI, not slash command)
- generate-domain-agents → collapse into `create-component --type domain-agent`

Net: 35 → 22 commands.

### A.3 Skills (121 v1 → ~60 v2)

Too long for full table here. Pruning policy:

**Keep (40 skills):**
- All pipeline-shared (3): agentic-engineering, spec-pipeline-shared, tdd-workflow
- All workflow-meta (11)
- All doc-generation (12)
- All audit-methodology (12)
- Core methodology (10): ai-first-engineering, cost-aware-llm-pipeline, deep-research, design-an-interface, iterative-retrieval, pc-evaluation, prd-to-plan, prompt-optimizer, search-first, web-research-strategy

**Consolidate (15 → 8):**
- rust-testing + golang-testing + python-testing + typescript-testing + kotlin-testing + csharp-testing + cpp-testing + shell-testing + django-tdd → 1 "testing" meta-skill that points to rules/<lang>/testing.md for specifics. 9 → 1.
- springboot-patterns + springboot-security + springboot-tdd + springboot-verification → 1 "springboot" skill. 4 → 1.
- kotlin-coroutines-flows + kotlin-exposed-patterns + kotlin-ktor-patterns → 1 "kotlin" skill. 3 → 1.
- django-patterns + django-security → 1. 2 → 1.
- swift-actor-persistence + swift-concurrency-6-2 + swift-protocol-di-testing + swiftui-patterns → 1 "swift" skill. 4 → 1.
- (etc for other language clusters)

**Drop (20 skills):**
- github-actions (37 words, superseded by ci-cd-workflows)
- enterprise-agent-ops (never referenced)
- foundation-models-on-device (Apple-specific, not ECC-core)
- nutrient-document-processing (third-party integration, not essential)
- clickhouse-io (unused)
- jpa-patterns, springboot-verification if springboot consolidated
- project-guidelines-example (example file, not a skill)
- continuous-agent-loop (covered by autonomous-loops)
- Any skill with <200 words that's purely a pointer to a rule file

Net: 121 → ~60 skills.

### A.4 Rules (80 files → ~40 files)

Keep all of `rules/common/` (6) and `rules/ecc/` (2). Prune language-specific rule sets for languages with <5% ECC usage (PHP, Perl → drop). Consolidate kotlin/java (JVM-family) rules where possible.

Net: 80 → ~40 files.

---

## Appendix B: ADR carry-forward list

Of v1's 67 ADRs, **18 are foundational** and should be ported verbatim (with renumbering) to v2 on day one:

- Hexagonal Architecture (v1 ADR-0001)
- Doc-First Spec Pipeline (v1 ADR-0006)
- Separate Workflow Crate (v1 ADR-0019)
- Concurrent Session Safety (v1 ADR-0024) — updated to include BL-156 learnings
- Domain Purity (v1 ADR-0028)
- Error Type Strategy (v1 ADR-0029)
- Keyless Sigstore Signing (v1 ADR-0038)
- Auditable Workflow Bypass (v1 ADR-0055)
- Declarative Tool Manifest (v1 ADR-0060)
- AAIF Alignment (v1 ADR-0062)
- Party Command (v1 ADR-0064)
- Tribal Knowledge Docs (v1 ADR-0065)
- Feature Input Boundary Validation (v1 ADR-0066)
- clap-derive Deny-List (v1 ADR-0067)

**Plus new v2 ADRs (day-one commits):**
- V2-ADR-0001: Session Liveness via Lock + Heartbeat + PID + StartTime (from BL-156 design)
- V2-ADR-0002: state.json is worktree-path-computed, no anchor file
- V2-ADR-0003: Silent Failures Banned at Lint Level (no `let _ =` on Result outside tests)
- V2-ADR-0004: Session-Budget Dimension in Spec-Adversary
- V2-ADR-0005: Clippy Disallowed Methods (SystemTime::now in app/domain banned)
- V2-ADR-0006: tasks.md as Session Handoff Contract
- V2-ADR-0007: Windows Graceful Degradation (Unix+procps is primary)

---

## Appendix C: Test migration strategy

v1: 3,384 workspace tests, cargo llvm-cov for coverage, cargo-mutants on ecc-domain, 22 Bats statusline tests, integration tests in `ecc-integration-tests` crate.

**v2 test invariants:**

1. **Domain crate: 100% function coverage + property tests on all invariants.** Target: ≥95% branch coverage. Every `Result` path tested. Zero real I/O.
2. **App crate: ≥80% function coverage.** Every `tracing::warn!` path tested (proves failure is observable). MockClock + MockFileSystem for all time / I/O interaction.
3. **Infra crate: 60-70% function coverage via `#[ignore]`-gated real-system tests.** CI runs with `ECC_E2E_ENABLED=1`; local runs default to unit-only.
4. **ecc-workflow: 90% coverage.** Every state transition tested. Every hook handler tested with stdin fixture JSON.
5. **ecc-flock: 100% coverage including Miri for UB detection.** FFI code is load-bearing.
6. **Integration tests: ~30 scenarios at workspace level.** Full pipeline /spec→/design→/implement on fixtures. BL-156 six-scenario matrix. Multi-process concurrent-session scenarios.
7. **Bats statusline: 22 tests ported as-is.**
8. **Property tests**: every domain invariant gets at least one `proptest!` block (fingerprint collisions, liveness verdict determinism, state transition legality, backlog ID monotonicity).
9. **Mutation testing**: `cargo mutants -p ecc-domain` in weekly CI; target ≥90% mutants killed.

**Final gate: `cargo test --workspace` under 3 minutes on M2/Ryzen 5600X laptop.** Budget enforced by nextest sharding if needed.

---

## Closing note

**The point of this document is not to be comprehensive about v1.** V1 documents itself — that's one of its strengths. The point is to capture, while the session context is fresh from BL-156 spec + design + implementation failure:

1. What the **spine** of ECC is (hexagonal Rust, doc-first pipeline, adversarial review, POSIX flock, worktree isolation). Keep.
2. What the **broken load-bearing piece** is (worktree GC, state.json sharing). Replace.
3. What the **rot** is (121 skills, 34 commands, silent errors, oversized files, Clock bypass, bus factor, recursive failure). Fix.
4. What **goes further** (session browser, debt registry, reproducible session record, hook-level observability, local LLM offload, session-budget adversary, tasks.md as handoff contract). Ship.

A contributor (human or AI) reading this cold should be able to start `cargo new --lib` on a clean directory and make recognizable progress toward v2 in under 2 hours.

— Authored during the session that demonstrated why v2 is necessary.
