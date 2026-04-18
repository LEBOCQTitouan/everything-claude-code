# Backlog Index

| ID | Title | Tier | Scope | Target | Status | Created |
|----|-------|------|-------|--------|--------|---------|
| BL-001 | Block auto-enable of MCP servers | 1 | LOW | direct edit | implemented | 2026-03-20 |
| BL-002 | Pin all MCP package versions | 1 | LOW | direct edit | implemented | 2026-03-20 |
| BL-003 | Prune stale local permissions | 1 | LOW | direct edit | implemented | 2026-03-20 |
| BL-004 | robert: read-only + memory + negative examples | 2 | MEDIUM | direct edit | implemented | 2026-03-20 |
| BL-005 | Update commands that call robert to handle his output | 2 | MEDIUM | direct edit | implemented | 2026-03-20 |
| BL-006 | spec-adversary: skills preload + negative examples | 2 | LOW | direct edit | implemented | 2026-03-20 |
| BL-007 | solution-adversary: skills preload + negative examples | 2 | LOW | direct edit | implemented | 2026-03-20 |
| BL-008 | drift-checker: skills preload | 2 | LOW | direct edit | implemented | 2026-03-20 |
| BL-009 | Add negative examples to planner agent | 2 | LOW | direct edit | implemented | 2026-03-20 |
| BL-010 | Create ubiquitous-language skill | 3 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-011 | Create grill-me skill | 3 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-012 | Create write-a-prd skill | 3 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-013 | Create interview-me skill | 3 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-014 | Create design-an-interface skill | 3 | HIGH | /spec dev | implemented | 2026-03-20 |
| BL-015 | Create request-refactor-plan skill | 3 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-016 | Create prd-to-plan skill | 3 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-017 | Create /catchup command | 4 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-019 | Create /spec command | 4 | MEDIUM | /spec dev | implemented | 2026-03-20 |
| BL-020 | Create /design command | 4 | LOW | direct edit | implemented | 2026-03-20 |
| BL-021 | Extract command reference tables from CLAUDE.md | 5 | LOW | direct edit | implemented | 2026-03-20 |
| BL-022 | Replace CLAUDE.md architecture block with pointer | 5 | LOW | direct edit | implemented | 2026-03-20 |
| BL-023 | Clean up stale workflow state | 5 | LOW | direct edit | implemented | 2026-03-20 |
| BL-024 | Add context:fork to heavy skills | 6 | LOW | direct edit | implemented | 2026-03-20 |
| BL-025 | Add memory:project to adversarial agents | 6 | LOW | direct edit | implemented | 2026-03-20 |
| BL-026 | Quarterly MCP version audit | 6 | LOW | process | implemented | 2026-03-20 |
| BL-027 | Cross-session memory system for actions, plans, and implementations | — | HIGH | /spec dev | implemented | 2026-03-21 |
| BL-028 | Add active web search phase to /spec-dev, /spec-fix, /spec-refactor | — | MEDIUM | /spec-dev, /spec-fix, /spec-refactor | implemented | 2026-03-21 |
| BL-029 | Persist specs and designs as versioned file artifacts | — | HIGH | /spec-dev, /spec-fix, /spec-refactor, /design | implemented | 2026-03-21 |
| BL-030 | Persist tasks.md as trackable artifact | — | HIGH | /implement | implemented | 2026-03-21 |
| BL-031 | Fresh context per TDD task via subagent isolation | — | HIGH | /implement | implemented | 2026-03-21 |
| BL-032 | Wave-based parallel TDD execution | — | MEDIUM | /implement | implemented | 2026-03-21 |
| BL-033 | Add /spec-quick for lightweight changes | — | MEDIUM | /spec-quick (new command) | archived | 2026-03-21 |
| BL-034 | Capture grill-me decisions in work-item files | — | MEDIUM | /spec-dev, /spec-fix, /spec-refactor | implemented | 2026-03-21 |
| BL-035 | Campaign file manifest for amnesiac agents | — | MEDIUM | /spec refactor | implemented | 2026-03-21 |
| BL-036 | Add numeric quality scores to adversary agents | — | MEDIUM | agents/spec-adversary.md, agents/solution-adversary.md | implemented | 2026-03-21 |
| BL-037 | AskUserQuestion preview field for architecture comparisons | — | LOW | commands/spec-dev.md, commands/design.md | implemented | 2026-03-21 |
| BL-038 | Add TaskCreate to audit-full and doc-orchestrator | — | LOW | commands/audit-full.md, agents/audit-orchestrator.md, agents/doc-orchestrator.md | implemented | 2026-03-21 |
| BL-039 | Add CronCreate suggestion to periodic commands | — | LOW | commands/audit-full.md, commands/review.md, commands/verify.md | archived | 2026-03-21 |
| BL-040 | Create meta-steering rules for ECC development | — | LOW | rules/ecc/development.md (new) | implemented | 2026-03-21 |
| BL-041 | Add CLAUDE_CODE_TASK_LIST_ID for cross-session persistence | — | LOW | .claude/hooks/workflow-init.sh | implemented | 2026-03-21 |
| BL-042 | Add background mode to /audit-full | — | LOW | commands/audit-full.md | archived | 2026-03-21 |
| BL-043 | Add QA strategist agent | — | LOW | agents/qa-strategist.md (new) | implemented | 2026-03-21 |
| BL-044 | Add grill-me interview step to /backlog add workflow | — | LOW | direct edit | archived | 2026-03-21 |
| BL-045 | Remove alias commands (plan, solution) and audit for further duplicates | — | MEDIUM | /spec-refactor | implemented | 2026-03-21 |
| BL-046 | Phase-gate hook blocks spec/plan/design file writes during active workflow phases | — | LOW | direct edit | implemented | 2026-03-21 |
| BL-047 | Automatic session-to-memory persistence with daily files | — | HIGH | /spec dev | implemented | 2026-03-21 |
| BL-048 | Comprehensive output summaries for spec → design → implement pipeline | — | MEDIUM | /spec-refactor | implemented | 2026-03-22 |
| BL-049 | Offload web research phase to Task subagents in /spec-* commands | — | MEDIUM | /spec-refactor | implemented | 2026-03-22 |
| BL-050 | Deferred pipeline summary tables — coverage delta, bounded contexts, per-test-name | — | MEDIUM | /spec-dev | implemented | 2026-03-22 |
| BL-051 | Explanatory narrative audit — all commands and workflows | — | HIGH | /spec refactor | implemented | 2026-03-22 |
| BL-052 | Replace .claude/hooks shell scripts with compiled Rust binaries | — | HIGH | /spec | implemented | 2026-03-22 |
| BL-053 | Poweruser statusline — explicit labels, usage bars, UX overhaul, and install fix | — | HIGH | /spec dev | implemented | 2026-03-22 |
| BL-054 | Full context clear + confirmation gate at /implement start | — | LOW | direct edit | archived | 2026-03-22 |
| BL-055 | Graceful mid-session exit when context gets heavy | — | MEDIUM | /spec dev | archived | 2026-03-22 |
| BL-056 | Context-aware doc generation step at end of /implement | — | HIGH | /spec dev | implemented | 2026-03-22 |
| BL-057 | Create grill-me-adversary companion skill with adaptive loop | — | MEDIUM | /spec | implemented | 2026-03-22 |
| BL-058 | Symlink-based instant config switching for ecc dev | — | MEDIUM | /spec-dev | implemented | 2026-03-22 |
| BL-059 | Auto-commit backlog edits at end of /backlog command | — | LOW | /spec-dev | implemented | 2026-03-22 |
| BL-060 | Simplify context management — remove graceful exit, keep warnings only | — | HIGH | /spec-refactor | implemented | 2026-03-23 |
| BL-061 | Interactive stage-by-stage questioning via AskUserQuestion for grill-me and backlog | — | HIGH | /spec-refactor | implemented | 2026-03-23 |
| BL-062 | Display full spec/design/implement artifacts inline in terminal | — | MEDIUM | /spec-refactor | implemented | 2026-03-26 |
| BL-063 | Create /commit slash command | — | MEDIUM | /spec-dev | implemented | 2026-03-26 |
| BL-064 | Full app cartography — user journeys, data flows, and element registry across all sessions | — | EPIC | /spec dev | implemented | 2026-03-26 |
| BL-065 | Full concurrent session safety — worktree isolation, serialized merge, codebase audit fixes | — | EPIC | /spec dev | implemented | 2026-03-26 |
| BL-066 | Deterministic backlog management — ID generation, duplicate detection, index auto-generation | — | MEDIUM | /spec dev | implemented | 2026-03-26 |
| BL-067 | Deterministic spec/design artifact validation — AC format, PC table, coverage mapping | — | HIGH | /spec dev | implemented | 2026-03-26 |
| BL-068 | Deterministic workflow state machine — typed state.json, phase transitions, artifact resolution | — | HIGH | /spec dev | implemented | 2026-03-26 |
| BL-069 | Deterministic convention linting — naming, placement, frontmatter field values | — | MEDIUM | /spec dev | implemented | 2026-03-26 |
| BL-070 | Deterministic wave grouping algorithm — PC parallelization from file-overlap analysis | — | MEDIUM | /spec dev | implemented | 2026-03-26 |
| BL-071 | Deterministic git analytics CLI — changelog generation, hotspot analysis, evolution metrics | — | MEDIUM | /spec dev | implemented | 2026-03-26 |
| BL-072 | Deterministic artifact scaffolding — spec, solution, and tasks template generation | — | MEDIUM | /spec dev | implemented | 2026-03-26 |
| BL-073 | Deterministic diagram trigger heuristics — auto-detect when diagrams need updating | — | LOW | /spec dev | implemented | 2026-03-26 |
| BL-074 | Deterministic doc metrics — staleness detection, coverage calculation, severity counting | — | LOW | /spec dev | implemented | 2026-03-26 |
| BL-075 | Deterministic task synchronization — single source of truth for TodoWrite and TaskCreate | — | HIGH | /spec dev | implemented | 2026-03-26 |
| BL-076 | Statusline Unicode byte-counting bug hides rate limit segments | — | LOW | /spec-fix | implemented | 2026-03-27 |
| BL-077 | Full documentation pass — coverage, drift validation, and gap analysis | — | EPIC | /spec-dev | implemented | 2026-03-27 |
| BL-078 | Context pre-hydration via hook before command runs | — | MEDIUM | /spec-dev | implemented | 2026-03-27 |
| BL-079 | Conditional rule/skill loading via frontmatter applicability | — | MEDIUM | /spec-dev | implemented | 2026-03-27 |
| BL-080 | TDD fix-loop budget cap at 2 rounds | — | LOW | direct edit | implemented | 2026-03-27 |
| BL-081 | Web-based upgrade audit command with Technology Radar output | — | EPIC | /spec-dev | implemented | 2026-03-27 |
| BL-082 | Add worktree display segment to statusline | — | LOW | direct edit | implemented | 2026-03-27 |
| BL-083 | Adversarial challenge phase for all /audit-* commands | — | HIGH | /spec-dev | implemented | 2026-03-27 |
| BL-084 | Backlog conformance audit — verify implementations match original backlog intent | — | MEDIUM | /spec dev | implemented | 2026-03-28 |
| BL-085 | WorktreeCreate/WorktreeRemove hooks break EnterWorktree tool | — | HIGH | /spec fix | implemented | 2026-03-28 |
| BL-086 | Knowledge sources registry — curated reference list with quadrant organization and command integration | — | HIGH | /spec | implemented | 2026-03-28 |
| BL-087 | Cargo xtask deploy — full local machine setup | — | HIGH | /spec-dev | implemented | 2026-03-28 |
| BL-088 | ecc update — self-update from GitHub Releases + cargo xtask deploy for dev | — | HIGH | /spec dev | implemented | 2026-03-28 |
| BL-089 | GitHub Actions skill + branch isolation hook for CI/CD workflow development | — | HIGH | /spec dev | implemented | 2026-03-28 |
| BL-090 | ECC component scaffolding — skill + /create-component command for agents, commands, skills, hooks | — | HIGH | /spec dev | 2026-03-28 | 2026-03-28 |
| BL-091 | ECC diagnostics — tiered verbosity with tracing, ecc status command, zero model cost | — | HIGH | /spec dev | implemented | 2026-03-28 |
| BL-092 | Structured log management — tracing + JSON rolling files + SQLite index + ecc log CLI | — | HIGH | /spec dev | implemented | 2026-03-28 |
| BL-093 | Three-tier memory system — semantic/episodic/working memory with SQLite index, consolidation, auto-gen MEMORY.md | — | EPIC | /spec dev | implemented | 2026-03-28 |
| BL-094 | Agent model routing optimization — downgrade misaligned agents from Opus to Sonnet/Haiku | — | HIGH | /spec refactor | implemented | 2026-03-28 |
| BL-095 | Extended thinking and effort tuning — adaptive thinking budgets per agent type | — | MEDIUM | /spec dev | implemented | 2026-03-28 |
| BL-096 | Cost and token tracking — observability prerequisite for optimization | — | MEDIUM | /spec dev | implemented | 2026-03-28 |
| BL-097 | Spec backlog in-work filtering — hide entries claimed by other sessions | — | MEDIUM | /spec-dev | implemented | 2026-03-29 |
| BL-098 | Socratic grill-me upgrade — depth-first questioning with OARS, laddering, MECE, and reflective rephrasing | — | HIGH | /spec-dev | implemented | 2026-03-29 |
| BL-099 | Migrate serde_yml to serde-yaml-ng — maintenance risk mitigation | — | MEDIUM | /spec-refactor | implemented | 2026-03-29 |
| BL-100 | sccache + mold build acceleration for dev environment | — | LOW | direct edit | implemented | 2026-03-29 |
| BL-101 | Miri unsafe code verification for ecc-flock | — | LOW | direct edit | implemented | 2026-03-29 |
| BL-102 | Promptware Engineering practices — prompt testing and monitoring | — | MEDIUM | /spec-dev | implemented | 2026-03-29 |
| BL-103 | Autonomous visual testing integration — vision-based UI validation | — | HIGH | /spec-dev | implemented | 2026-03-29 |
| BL-104 | Multi-agent team coordination — shared state and task handoff | — | HIGH | /spec-dev | implemented | 2026-03-29 |
| BL-105 | Bump crossterm 0.28 → 0.29.0 | — | LOW | direct edit | implemented | 2026-03-29 |
| BL-106 | Harness reliability metrics — benchmark ECC against reference patterns | — | MEDIUM | /spec-dev | implemented | 2026-03-29 |
| BL-107 | Audit-web guided profile — interactive setup, persisted dimensions, improvement suggestions | — | HIGH | /spec-dev | implemented | 2026-03-29 |
| BL-108 | Smart stop notification — only notify on final stop or user input needed | — | MEDIUM | /spec-dev | implemented | 2026-03-29 |
| BL-110 | Add cargo-semver-checks to CI pipeline | — | LOW | direct edit | implemented | 2026-03-31 |
| BL-111 | Enable GitHub Merge Queue for CI load reduction | — | LOW | direct edit | implemented | 2026-03-31 |
| BL-112 | Evaluate cargo-dist to replace custom release.yml | — | MEDIUM | /spec-refactor | implemented | 2026-03-31 |
| BL-113 | Upgrade rusqlite 0.34 to 0.38 | — | LOW | direct edit | implemented | 2026-03-31 |
| BL-114 | Upgrade rustyline 15 to 17 | — | LOW | direct edit | implemented | 2026-03-31 |
| BL-115 | Upgrade toml 0.8 to 0.9 | — | LOW | direct edit | implemented | 2026-03-31 |
| BL-116 | Add cargo-mutants mutation testing | — | MEDIUM | /spec-dev | implemented | 2026-03-31 |
| BL-117 | Evaluate release-plz for automated semver and changelog | — | MEDIUM | /spec-dev | implemented | 2026-03-31 |
| BL-118 | Add SLSA provenance attestations to release pipeline | — | MEDIUM | /spec-dev | implemented | 2026-03-31 |
| BL-119 | Create GitHub workflow templates for Claude Code integration | — | HIGH | /spec-dev | implemented | 2026-03-31 |
| BL-120 | Pattern Library for Agent-Assisted Development | — | EPIC | /spec-dev | implemented | 2026-04-04 |
| BL-121 | Token optimization audit — comprehensive spend reduction opportunity mapping | — | EPIC | /audit | implemented | 2026-04-05 |
| BL-122 | Worktree Auto-Merge and Cleanup Enforcement | — | workflow | ecc-workflow | implemented | 2026-04-06 |
| BL-123 | Caveman-style brevity optimization — reduce token consumption across all ECC agents and commands | — | HIGH | /spec-refactor | implemented | 2026-04-06 |
| BL-124 | Token optimization wave 1 — zero-cost CLI redirects and one-liner fixes | — | LOW | direct edit | implemented | 2026-04-06 |
| BL-125 | Token optimization wave 2 — boilerplate extraction and context trimming | — | LOW | direct edit | implemented | 2026-04-06 |
| BL-126 | Token optimization wave 3 — new CLI commands replacing agent work | — | HIGH | /spec-dev | implemented | 2026-04-06 |
| BL-127 | Token optimization wave 4 — pipeline architecture for session and subagent reduction | — | HIGH | /spec-dev | implemented | 2026-04-06 |
| BL-128 | Token optimization wave 5 — local LLM offload for mechanical agents | — | MEDIUM | /spec-dev | implemented | 2026-04-06 |
| BL-129 | Bidirectional pipeline transitions with justification logging | — | HIGH | /spec-refactor | implemented | 2026-04-07 |
| BL-130 | US/epic-level sub-tracking within /implement pipeline | — | MEDIUM | /spec-dev | in-progress | 2026-04-07 |
| BL-132 | Full ASCII diagram sweep of all 9 ECC crates | 5 | HIGH | direct edit | open | 2026-04-08 |
| BL-133 | Migrate workspace to Rust 2024 edition | — | MEDIUM | /spec-dev | implemented | 2026-04-09 |
| BL-134 | Audit CLAUDE.md for LLM-generated content | — | LOW | direct edit | implemented | 2026-04-09 |
| BL-135 | Add cargo-llvm-cov coverage gate to CI | — | LOW | direct edit | implemented | 2026-04-09 |
| BL-136 | Add cargo-vet for SLSA Level 2 supply chain compliance | — | MEDIUM | /spec-dev | implemented | 2026-04-09 |
| BL-137 | Apply difficulty-aware model routing from multi-agent research | — | HIGH | /spec-dev | archived | 2026-04-09 |
| BL-138 | Evaluate hex crate for compile-time architecture boundary enforcement | — | MEDIUM | /spec-dev | implemented | 2026-04-09 |
| BL-139 | Monitor Claude Code Agent Teams API for ECC integration | — | LOW | /spec-dev | implemented | 2026-04-09 |
| BL-140 | Competitor analysis: Claw Code and Goose agent frameworks | — | LOW | /spec-dev | implemented | 2026-04-09 |
| BL-141 | Monitor serde-saphyr for 0.1.0 stability release | — | LOW | direct edit | open | 2026-04-09 |
| BL-142 | Add docs/cartography/ to phase-gate allowlist | — | LOW | direct edit | implemented | 2026-04-09 |
| BL-143 | /project-foundation command — PRD + Architecture docs creation with codebase-aware challenge | — | MEDIUM | /spec-dev | implemented | 2026-04-12 |
| BL-144 | /party command — BMAD-style multi-agent round-table with auto-generated domain agents | — | HIGH | /spec-dev | open | 2026-04-12 |
| BL-145 | Wire party-mode into /spec phase as augmentation layer before adversarial review | — | MEDIUM | /spec-dev | open | 2026-04-12 |
| BL-146 | Declarative tool manifest — centralize allowedTools via Serde-validated YAML/TOML | — | HIGH | /spec-dev | implemented | 2026-04-12 |
| BL-147 | AGENTS.md / AAIF standard alignment audit | — | MEDIUM | /spec-dev | implemented | 2026-04-12 |
| BL-148 | Formalize session resume/persist/delegate hook lifecycle | — | MEDIUM | /spec-dev | open | 2026-04-12 |
| BL-149 | Add agentic self-evaluation step between /implement TDD iterations | — | MEDIUM | /spec-dev | implemented | 2026-04-12 |
| BL-153 | Redact or truncate feature field in tracing::info! to prevent log-injection amplification | — | MEDIUM | direct edit | open | 2026-04-17 |
| BL-154 | Widen ecc validate commands rule to catch backtick-embedded !$ARGUMENTS inline-code patterns | — | LOW | direct edit | open | 2026-04-17 |
| BL-155 | Add Foundation variant to Concern domain enum for /project-foundation workflow | — | MEDIUM | /spec-dev | open | 2026-04-17 |
| BL-156 | Safe worktree GC — skip active session worktrees | — | MEDIUM | /spec-fix | open | 2026-04-18 |

## Dependency Graph

```
BL-010 → BL-004 (ubiquitous-language skill enables robert skill preload)
BL-012 → BL-016 (prd-to-plan consumes write-a-prd output)
BL-014 → BL-020 (/design command wraps design-an-interface skill)
BL-002 → BL-026 (quarterly audit requires initial pinning)
BL-017 → BL-023 (/catchup prevents stale state recurrence)
BL-027 → BL-017 (memory system feeds /catchup command)
BL-027 → BL-004 (memory system feeds robert negative examples)
BL-025 → BL-027 (per-agent memory flags complement cross-session log)
BL-029 → BL-030 (spec files enable tasks.md persistence)
BL-029 → BL-031 (spec files enable subagent isolation)
BL-029 → BL-034 (spec files enable grill-me decision capture)
BL-031 → BL-032 (subagent isolation enables wave-based parallelism)
BL-030 → BL-017 (tasks.md enables /catchup progress display)
BL-041 → BL-030 (task list ID complements file-based tasks)
BL-047 → BL-027 (auto-memory extends cross-session memory system)
BL-064 → BL-056 (full cartography extends implement-end doc generation)
BL-064 → BL-029 (cartography consumes spec artifacts)
BL-064 → BL-030 (cartography consumes task artifacts)
BL-065 → BL-052 (Rust binaries solve race conditions natively)
BL-065 → BL-031 (extends subagent worktree isolation to full sessions)
BL-065 → BL-046 (phase-gate affected by state.json TOCTOU)
BL-066 → BL-059 (backlog auto-commit benefits from deterministic reindex)
BL-067 → BL-029 (spec validation operates on persisted spec artifacts)
BL-068 → BL-046 (typed state machine replaces shell-based phase gate)
BL-068 → BL-052 (Rust state machine replaces shell hook scripts)
BL-070 → BL-032 (deterministic wave grouping replaces LLM-based wave dispatch)
BL-072 → BL-029 (scaffolding generates the artifact files that persistence manages)
BL-072 → BL-030 (scaffolding generates tasks.md from solution PCs)
BL-075 → BL-030 (task sync manages the persisted tasks.md)
BL-075 → BL-041 (task sync complements task list ID persistence)
BL-078 → BL-052 (Rust hook binaries make pre-hydration faster)
BL-081 → BL-078 (web audit benefits from context pre-hydration for inventory phase)
BL-084 → BL-029 (conformance audit cross-references persisted spec artifacts)
BL-084 → BL-066 (deterministic backlog IDs improve traceability)
BL-085 → BL-065 (worktree hook fix unblocks BL-065 Sub-Spec C worktree isolation)
BL-088 → BL-087 (ecc update leverages xtask deploy infrastructure for dev mode)
BL-088 → BL-089 (ecc update needs release pipeline patterns from GHA skill)
BL-089 → BL-065 (branch isolation relies on worktree isolation for multi-session)
BL-092 → BL-091 (structured logs build on tracing foundation from diagnostics)
BL-093 → BL-092 (memory system shares SQLite infrastructure with log management)
BL-093 → BL-065 (memory consolidation uses flock for concurrent writes)
BL-094 → BL-096 (model routing optimization needs cost tracking for before/after)
BL-095 → BL-096 (thinking tuning needs cost tracking for before/after)
BL-096 → BL-092 (cost tracking shares ~/.ecc/logs/ infrastructure with structured logs)
BL-097 → BL-065 (in-work filtering relies on session identity from worktree isolation)
BL-097 → BL-066 (extends deterministic backlog management with transient lock files)
BL-098 → BL-011 (upgrades the grill-me skill created in BL-011)
BL-098 → BL-057 (upgrades grill-me-adversary companion created in BL-057)
BL-098 → BL-061 (builds on AskUserQuestion integration from BL-061)
BL-100 → BL-087 (build acceleration complements xtask deploy infrastructure)
BL-102 → BL-090 (promptware testing extends component scaffolding with eval harness)
BL-102 → BL-092 (prompt monitoring shares structured log infrastructure)
BL-104 → BL-065 (team coordination requires session identity from worktree isolation)
BL-104 → BL-093 (shared state benefits from memory system infrastructure)
BL-106 → BL-091 (harness metrics build on tracing foundation from diagnostics)
BL-106 → BL-092 (harness metrics share structured log infrastructure)
BL-107 → BL-081 (extends the audit-web command created in BL-081)
BL-107 → BL-083 (adversarial challenge phase complements self-improvement suggestions)
BL-109 → BL-091 (comms pipeline benefits from tracing for observability)
BL-109 → BL-092 (comms pipeline shares structured log infrastructure)
BL-120 → BL-086 (pattern library extends knowledge sources registry with internal patterns)
BL-120 → BL-079 (conditional rule loading enables language-aware pattern injection)
BL-120 → BL-093 (pattern search benefits from memory system infrastructure)
BL-121 → BL-095 (token audit needs thinking budget baseline from effort tuning)
BL-121 → BL-096 (token audit needs cost tracking baseline for before/after comparison)
BL-123 → BL-121 (caveman brevity benefits from token audit findings but can proceed independently)
BL-124 → BL-121 (wave 1 CLI redirects derived from token audit findings)
BL-125 → BL-121 (wave 2 boilerplate cleanup derived from token audit findings)
BL-126 → BL-121 (wave 3 new CLI commands derived from token audit findings)
BL-126 → BL-124 (wave 3 builds on wave 1 redirect patterns)
BL-127 → BL-121 (wave 4 pipeline changes derived from token audit findings)
BL-127 → BL-124 (wave 4 builds on wave 1 conditional patterns)
BL-128 → BL-121 (wave 5 local LLM offload derived from token audit findings)
BL-128 → BL-126 (wave 5 offloads agents not already replaced by CLI in wave 3)
BL-144 → BL-143 (/party recommended-panel heuristics consume /project-foundation PRD + arch docs)
BL-144 → BL-137 (party-mode agent invocation applies difficulty-aware model routing)
BL-145 → BL-144 (spec-phase integration requires /party command to exist first)
BL-156 → BL-065 (session-aware GC extends worktree isolation infrastructure)
BL-156 → BL-097 (reuses transient lock-file pattern for session liveness)
```

## Stats

- **Total:** 150
- **Open:** 9
- **In-progress:** 1
- **Implemented:** 132
- **Archived:** 7
- **2026-03-28:** 1
