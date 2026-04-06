# Token Optimization Audit — 2026-04-06

**BL-121** | Scope: All ECC components | Axes: CLI offload, Local LLM, Context bloat, Session count

## Executive Summary

41,719 total lines of content across agents (7,928), commands (5,878), skills (17,092), rules (3,517), and docs. Every session loads ~1,300 lines of baseline context (CLAUDE.md + common rules + ECC rules + Rust rules + skill name listing). The audit identified **39 findings** across 4 optimization axes, with **10 HIGH-impact** items that collectively could eliminate 30-40% of per-session token spend.

The highest-ROI cluster: 3 agents (`evolution-analyst`, `doc-generator` changelog step, `backlog-curator` duplicate check) already have CLI equivalents — redirecting them saves Opus/Sonnet tokens for zero implementation cost.

## Axis 1: Rust CLI Offload Candidates (15 findings)

Tasks with zero reasoning requirement that should run as compiled binaries.

| # | Component | Current Behavior | Proposed CLI Command | Impact | Complexity |
|---|-----------|-----------------|----------------------|--------|------------|
| 1.1 | `agents/drift-checker.md` | Claude reads plan.md + implement-done.md, diffs AC/PC IDs, computes drift level — pure set math | `ecc drift check` — parse artifacts, diff tables, write drift-report.md | HIGH | MEDIUM |
| 1.2 | `agents/module-summary-updater.md` | Claude fills fixed markdown template from parent-provided structured data | `ecc docs update-module-summary --changed-files <list>` — template expansion | HIGH | LOW |
| 1.3 | `agents/doc-generator.md` (changelog step) | Claude runs `git log`, parses conventional commits — **identical to existing `ecc analyze changelog`** | Call `ecc analyze changelog` instead of reimplementing | HIGH | TRIVIAL |
| 1.4 | `agents/evolution-analyst.md` (hotspot steps) | Opus runs git log queries — **identical to existing `ecc analyze hotspots` + `ecc analyze coupling`** | Call existing CLI commands, feed output to agent for interpretation only | HIGH | TRIVIAL |
| 1.5 | `agents/backlog-curator.md` (duplicate check) | Sonnet reads all BL-*.md, scores keyword overlap — **`ecc backlog check-duplicates` already exists** | Call existing CLI command | MEDIUM | TRIVIAL |
| 1.6 | `agents/diagram-updater.md` (trigger detection) | Haiku evaluates 3 path-based trigger heuristics — pure file/path analysis | `ecc diagram triggers --changed-files <list>` — output JSON triggers | MEDIUM | TRIVIAL |
| 1.7 | `agents/doc-reporter.md` (coverage calc) | Haiku counts documented vs undocumented public items per module — regex counting | `ecc docs coverage --scope <path>` — walk files, count doc comments | MEDIUM | MEDIUM |
| 1.8 | `commands/catchup.md` (git status) | Claude runs git status/log/stash/worktree and formats — no interpretation | Extend `ecc status --git` to cover all git context for /catchup | MEDIUM | LOW |
| 1.9 | `agents/cartographer.md` (slug derivation) | Haiku reads delta JSONs, derives slugs using documented algorithm | `ecc cartography plan --deltas <path>` — parse + classify + output JSON | MEDIUM | LOW |
| 1.10 | `commands/commit.md` (atomic lint) | Claude analyzes staged diff for multi-concern detection — directory-based heuristic | `ecc commit lint --staged` — flag files spanning >1 top-level dir | MEDIUM | LOW |
| 1.11 | `agents/doc-validator.md` (count drift) | Claude validates numeric claims in CLAUDE.md against filesystem | `ecc validate claude-md --counts` — grep claims, cross-check | MEDIUM | LOW |
| 1.12 | `agents/doc-validator.md` (file size) | Claude globs docs/**/*.md, counts lines, flags violations | `ecc validate doc-sizes --scope docs/` — extend existing validation | LOW | TRIVIAL |
| 1.13 | `agents/evolution-analyst.md` (bus factor) | Opus counts distinct authors per file via git shortlog | Already exists as `ecc analyze bus-factor` — redirect | LOW | TRIVIAL |
| 1.14 | `agents/diagram-generator.md` (dedup) | Haiku deduplicates diagram requests by priority — deterministic merge | `ecc diagram deduplicate` — priority queue merge | LOW | TRIVIAL |
| 1.15 | `commands/implement.md` (sources lookup) | Claude reads sources.md, matches modules, updates timestamps | `ecc sources touch --module <name>` — extend existing sources CLI | LOW | TRIVIAL |

### Axis 1 Priority Summary

**Immediate wins (TRIVIAL complexity):** 1.3, 1.4, 1.5, 1.6, 1.12, 1.13, 1.14, 1.15 — redirect to existing CLI commands or add thin wrappers. Combined: eliminates ~6 agent/subagent invocations per pipeline run.

**Next wave (LOW-MEDIUM complexity):** 1.1, 1.2, 1.7, 1.8, 1.9, 1.10, 1.11 — new CLI commands but straightforward Rust (file parsing, template expansion, regex counting).

---

## Axis 2: Local LLM Offload Candidates (9 findings)

Tasks simple enough for a 7B-13B local model (Ollama/LM Studio).

| # | Component | Current Model | Proposed Fix | Local Model | Impact | Complexity |
|---|-----------|--------------|--------------|-------------|--------|------------|
| 2.1 | `agents/drift-checker.md` | haiku | Parse two markdown files, diff ID lists, fill fixed table — zero creativity | 7B (Mistral-7B-Instruct) | HIGH | TRIVIAL |
| 2.2 | `agents/module-summary-updater.md` | haiku | Fill fixed template from parent-provided structured inputs | 7B (Qwen2.5-7B-Instruct) | HIGH | TRIVIAL |
| 2.3 | `agents/doc-reporter.md` | haiku | Arithmetic + table formatting from pre-parsed coverage data | 7B (Llama-3.2-8B-Instruct) | HIGH | LOW |
| 2.4 | `agents/cartography-flow-generator.md` | haiku | Schema-fill from structured delta JSON with GAP markers | 7B (Mistral-7B-Instruct) | MEDIUM | LOW |
| 2.5 | `agents/cartography-journey-generator.md` | haiku | Same as flow-generator — parallel workload makes savings additive | 7B (Mistral-7B-Instruct) | MEDIUM | LOW |
| 2.6 | `agents/cartographer.md` | haiku | Pure routing logic — parse JSON, apply slug rules, dispatch | 7B (Qwen2.5-7B-Instruct) | MEDIUM | LOW |
| 2.7 | `agents/diagram-updater.md` | haiku | Mermaid syntax generation — needs reliable bracket handling | 13B (Qwen2.5-14B-Instruct) | MEDIUM | MEDIUM |
| 2.8 | `agents/diagram-generator.md` | haiku | Full Mermaid pipeline with mmdc validation retry loop | 13B (Qwen2.5-14B-Instruct) | MEDIUM | MEDIUM |
| 2.9 | `agents/convention-auditor.md` | sonnet | Grep-driven counting + threshold rules — model aggregates tool output | 13B (Mistral-Nemo-12B) | MEDIUM | LOW |

### Axis 2 Notes

- **Overlap with Axis 1**: Findings 2.1 and 2.2 are also Axis 1 candidates. CLI offload is preferred when feasible (zero inference cost). Local LLM is the fallback when the task needs *some* text generation but not Claude-grade reasoning.
- **Excluded**: `doc-generator` (inserting language-correct doc comments requires reliable syntax), `web-radar-analyst` (relevance judgment), `comms-generator` (creative writing).
- **Setup requirement**: Ollama with 7B model requires ~4GB RAM, 13B requires ~8GB. Users need `ollama pull` + MCP server configuration.

---

## Axis 3: Context Window Bloat (10 findings)

Patterns that inflate input context without adding value.

| # | Component | Current Behavior | Proposed Fix | Est. Waste | Impact | Complexity |
|---|-----------|-----------------|--------------|------------|--------|------------|
| 3.1 | `agents/*.md` (25 files) | Identical TodoWrite boilerplate block (~40 words) in 25 agents: "Create a TodoWrite checklist... If TodoWrite unavailable, proceed without tracking" | Extract to agent frontmatter convention: `tracking: todowrite` field. Agent runtime injects the block. Saves 25 × 40 = ~1,000 words of duplicate content | ~4,000 tokens across all agents | HIGH | LOW |
| 3.2 | `commands/*.md` (26 refs) | Identical narrative-conventions reference (~30 words) in 26 command/agent files: "See skills/narrative-conventions/SKILL.md conventions. Before each..." | Replace with single-line directive: `> **Narrative**: See narrative-conventions skill.` Already done in some files; standardize across all 26 | ~3,000 tokens total | HIGH | TRIVIAL |
| 3.3 | `rules/` (14 language dirs) | 14 language-specific rule directories totaling ~2,900 lines. Only Rust rules are relevant for this project. Other languages have `paths:` frontmatter but still appear in rule listings | Verify conditional loading works correctly — if language rules with non-matching paths are truly excluded from context, this is a non-issue. If they're loaded anyway, that's ~2,500 lines of waste per session | ~10,000 tokens if loaded | HIGH | TRIVIAL (verify) |
| 3.4 | `commands/spec-{dev,fix,refactor}.md` | Phase 3 (Web Research: ~7 lines), Phase 3.5 (Sources: ~5-9 lines), and partial Phase 0 are duplicated across all 3 spec commands | Already mitigated by `spec-pipeline-shared` skill pointers. Remaining duplication is ~20 lines × 3 = 60 lines — low priority | ~240 tokens | LOW | TRIVIAL |
| 3.5 | `CLAUDE.md` (166 lines) | Loaded every session. Contains CLI command reference (~60 lines), gotchas (~30 lines), and slash command list (~10 lines). CLI commands rarely needed in-context | Move CLI command reference to `docs/commands-reference.md` (already exists), keep only top-10 commands in CLAUDE.md. Lazy-load full reference when user asks about CLI | ~500 tokens | MEDIUM | TRIVIAL |
| 3.6 | `rules/common/performance.md` (82 lines) | Full model routing table, thinking effort tiers, and context management guidelines loaded every session — reference material, rarely actionable during coding | Split: keep model routing table (20 lines) in rules; move context management prose and troubleshooting to docs/. Agent frontmatter `model`/`effort` fields already encode routing | ~300 tokens | LOW | TRIVIAL |
| 3.7 | `rules/common/agents.md` (53 lines) | Full agent table + parallel execution patterns loaded every session — Claude already knows its agents from system prompt | Remove or slim to 10-line summary. The system-reminder already lists all available agents with descriptions | ~200 tokens | LOW | TRIVIAL |
| 3.8 | `rules/common/development-workflow.md` (47 lines) | Full 5-step workflow loaded every session — only relevant during /spec pipeline, not during direct edits | Add `paths:` frontmatter to scope to pipeline commands only, or mark as deferred-load | ~200 tokens | LOW | TRIVIAL |
| 3.9 | Agent `skills:` frontmatter | Some agents list skills they reference but never invoke. Each skill name adds to the system prompt skill listing | Audit: remove unused skill references from agent frontmatter. Minor savings (~5 tokens per unused skill × ~10 agents) | ~50 tokens | LOW | TRIVIAL |
| 3.10 | `rules/common/patterns.md` (31 lines) | Skeleton Projects and Repository Pattern descriptions — generic advice loaded every session | Move to a skill (loaded on demand) or remove if CLAUDE.md already covers it | ~120 tokens | LOW | TRIVIAL |

### Axis 3 Notes

- **Biggest win**: 3.1 (TodoWrite boilerplate) + 3.2 (narrative-conventions refs) — ~7,000 tokens of pure duplication across agents/commands. Fix is mechanical.
- **Verification needed**: 3.3 (language rules) — if Claude Code's conditional loading works correctly, this is zero waste. If not, it's the single largest bloat source at ~10,000 tokens.
- **Baseline context**: ~1,300 lines loaded per session (CLAUDE.md 166 + common rules 430 + ECC rules 92 + Rust rules 443 + skill listing ~170). This is reasonable for a complex project.

---

## Axis 4: Session Count Reduction (10 findings)

Workflows requiring unnecessary session splits or subagent explosions.

| # | Component | Current Behavior | Proposed Fix | Impact | Complexity |
|---|-----------|-----------------|--------------|--------|------------|
| 4.1 | `spec-dev.md` → `design.md` → `implement.md` | Hard STOP after each phase — 3 mandatory sessions minimum per feature. Each new session re-reads state.json, spec, design from disk | Add `--continue` flag to `/spec-dev` that flows into `/design` after adversarial PASS (user approval via Plan Mode). Or add `/spec-and-design` combined command | HIGH | MEDIUM |
| 4.2 | `commands/design.md` Phases 2-4 | `uncle-bob`, `robert`, `security-reviewer` run sequentially as separate subagents reading the same design output — triple re-read | Merge into single composite "design-reviewer" subagent with combined prompt covering SOLID + Oath + security. All three are read-only scanners | HIGH | LOW |
| 4.3 | `commands/implement.md` Phase 3 | Each Pass Condition gets a fresh tdd-executor subagent. 10-PC design = 10 subagent spawns. Sequential PCs on same files force re-reads | Group PCs sharing `## Files to Modify` into batched tdd-executor runs. Wave model already groups independent PCs; extend to batch sequential same-file PCs | HIGH | MEDIUM |
| 4.4 | `audit-full.md` + individual audits | `/audit-full` runs all domain audits. Individual `/audit-*` commands re-run the same agents ignoring prior full report | Per-domain cache sections in full-audit report. Individual audits check for recent full report (within N days), skip if fresh | MEDIUM | LOW |
| 4.5 | All `audit-*.md` commands | Universal `audit-challenger` subagent launched even for clean/low-finding audits (0-2 LOW findings) | Conditional: launch challenger only if ≥3 findings or severity ≥ HIGH. Skip for clean audits | MEDIUM | TRIVIAL |
| 4.6 | `commands/spec-dev.md` Phases 1-2 | `requirements-analyst` and `architect` run sequentially despite no data dependency between them | Launch in parallel (same pattern `spec-refactor` already uses for its Phase 1 trio) | MEDIUM | TRIVIAL |
| 4.7 | `commands/design.md` Phase 0 | Full spec re-read on every `/design` invocation — rebuilds context from disk | Add `## Context Brief` (≤200 lines) at top of spec for quick loading; defer full read to phases that need it | MEDIUM | LOW |
| 4.8 | `commands/implement.md` Phase 7.5 | `module-summary-updater` + `diagram-updater` always launch, even for single-line changes | Skip supplemental doc agents if PC count ≤ 2 or all changed files are tests/config | LOW | TRIVIAL |
| 4.9 | `commands/spec-refactor.md` Phase 1 | Three analysis agents always launch for every refactor, even small renames | Add `--quick` flag: skip `evolution-analyst` and `component-auditor` for targeted refactors | LOW | TRIVIAL |
| 4.10 | `spec-*.md` Phase 3 (web research) | Three separate web-research subagents across spec variants, no caching | Short-lived `.research-cache-<hash>.md` (1h TTL) — skip web subagent on cache hit | LOW | MEDIUM |

### Axis 4 Notes

- **Biggest structural cost**: 4.1 (pipeline fragmentation) — the 3-session minimum per feature is the dominant session-count driver. Even with `--continue`, user approval gates are needed.
- **Quick wins**: 4.5 (conditional challenger) and 4.6 (parallel spec agents) are TRIVIAL changes with immediate effect.

---

## Cross-Axis Overlap

Some findings appear in multiple axes. Choose one implementation:

| Finding | Axis 1 (CLI) | Axis 2 (Local LLM) | Recommendation |
|---------|-------------|-------------------|----------------|
| drift-checker | 1.1 | 2.1 | **CLI** — pure set math, no LLM needed |
| module-summary-updater | 1.2 | 2.2 | **CLI** — template expansion, no LLM needed |
| diagram trigger detection | 1.6 | 2.7 (partial) | **CLI** — path heuristics only; keep LLM for Mermaid authoring |
| cartography slug derivation | 1.9 | 2.6 (partial) | **CLI** — documented algorithm; keep LLM for content generation |

---

## Recommended Execution Order

### Wave 1: Zero-cost redirects (TRIVIAL, no new code)
1. **1.3** — doc-generator changelog → `ecc analyze changelog`
2. **1.4** — evolution-analyst hotspots → `ecc analyze hotspots` + `ecc analyze coupling`
3. **1.5** — backlog-curator duplicate check → `ecc backlog check-duplicates`
4. **1.13** — evolution-analyst bus factor → `ecc analyze bus-factor`
5. **3.2** — standardize narrative-conventions one-liner across 26 files
6. **4.5** — conditional audit-challenger (≥3 findings gate)
7. **4.6** — parallel requirements-analyst + architect in spec-dev

### Wave 2: Boilerplate cleanup (TRIVIAL-LOW, mechanical edits)
8. **3.1** — extract TodoWrite boilerplate to frontmatter convention
9. **3.5** — slim CLAUDE.md CLI reference
10. **3.3** — verify language rule conditional loading
11. **3.6-3.10** — rule file trimming

### Wave 3: New CLI commands (LOW-MEDIUM complexity)
12. **1.1** — `ecc drift check`
13. **1.2** — `ecc docs update-module-summary`
14. **1.7** — `ecc docs coverage`
15. **1.6** — `ecc diagram triggers`
16. **1.10** — `ecc commit lint --staged`
17. **1.11** — `ecc validate claude-md --counts`

### Wave 4: Architectural changes (MEDIUM complexity)
18. **4.1** — `--continue` flag or combined `/spec-and-design`
19. **4.2** — composite design-reviewer subagent
20. **4.3** — batched tdd-executor for same-file PCs
21. **4.4** — per-domain audit caching

### Wave 5: Local LLM infrastructure (MEDIUM complexity, optional)
22. **2.4-2.6** — cartography trio on local 7B
23. **2.7-2.8** — diagram agents on local 13B
24. **2.9** — convention-auditor on local 13B
25. Setup: Ollama MCP server integration, model routing config

---

## Estimated Impact

| Wave | Findings | Token Savings | Session Savings | Effort |
|------|----------|--------------|-----------------|--------|
| 1 | 7 | ~15% per-session (eliminate 4-6 unnecessary agent invocations) | — | 1 session |
| 2 | 5 | ~5% baseline context reduction (~7K tokens) | — | 1 session |
| 3 | 6 | ~10% per-pipeline (6 new CLI commands replace agent work) | — | 3-4 sessions |
| 4 | 4 | ~10% per-pipeline (fewer subagents, optional phase merging) | 1-2 sessions per feature | 4-5 sessions |
| 5 | 4 | API cost → local compute (variable, depends on usage) | — | 2-3 sessions |

**Total estimated reduction**: 30-40% per-session token spend (Waves 1-3), plus optional session count reduction (Wave 4) and API cost elimination for mechanical tasks (Wave 5).
