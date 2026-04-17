---
title: "Competitor analysis — Claw Code and Goose vs ECC"
source: BL-140
date: 2026-04-12
status: complete
---

# Competitor Analysis: Claw Code & Goose vs ECC

## Scope

Compare Claw Code (72k★, clean-room Claude Code rewrite) and Goose (Block/Square, ~29k★, founding Agentic AI Foundation member) against ECC on four axes: tool dispatch, context management, agent isolation, hook/plugin systems. Identify patterns worth adopting.

## Summary Table

| Dimension | ECC | Claw Code | Goose |
|-----------|-----|-----------|-------|
| **Language** | Rust (9-crate hex arch) | Python primary + Rust runtime | Rust |
| **LLM provider** | Claude only (via Claude Code harness) | Provider-abstracted | Provider-abstracted (any LLM) |
| **Tool dispatch** | ECC: declarative tool manifest (`manifest/tool-manifest.yaml`) with preset resolution | YAML tool manifest → Serde → Rust closures | MCP extension system |
| **Context/session state** | `.claude/workflow/state.json` + per-worktree isolation | In-memory arenas + compaction algorithms | Core agent loop with plan→execute→evaluate |
| **Agent isolation** | Worktree-per-session (git-backed) | Session-state via arenas | Per-task extension scope |
| **Hook system** | Lifecycle events (PreToolUse, PostToolUse, SubagentStart, SessionStart, SessionEnd) | `Vec<Box<dyn Fn()>>` hook chain + plugin lifecycle (resume/persist/delegate) | Extension registration (MCP) |
| **Plugin/extension model** | Skills + slash commands + agents | MCP nodes chained as petgraph DAG | MCP server registry (3k+ entries in 2026) |
| **Distribution** | CLI binary (cargo-dist) | Python package | CLI + Desktop app |
| **Agent definition** | Markdown with YAML frontmatter | Markdown (AGENTS.md convention) | AGENTS.md convention |
| **Governance** | Independent | Independent | Linux Foundation / AAIF |

## Convergent Patterns

1. **Markdown + YAML agent definitions** — all three converge on `AGENTS.md`-style files. ECC's format aligns with this emerging standard.
2. **Worktree/session isolation** — ECC (git worktrees), Claw (arenas), Goose (per-task extension scope) all enforce isolation boundaries.
3. **Lifecycle hooks** — all three expose hook points around session/tool events.
4. **Hexagonal/modular architecture** — all three favor modular crates/modules over monolithic design.

## Divergent Patterns — What ECC Lacks

### 1. Declarative tool manifest (Claw Code)
Claw defines tool execution pipelines in YAML, parsed via Serde for type-safe dispatch. ECC hardcodes `allowedTools` lists in agent frontmatter and in the ad-hoc dispatch code in each command. A declarative manifest would enable:
- Compile-time validation of tool availability per agent
- Versioned tool interfaces
- Centralized tool registration instead of per-agent duplication

### 2. MCP DAG composition (Claw Code)
Claw resolves MCP nodes as a DAG via petgraph topological sort, enabling chained tool pipelines with dependency ordering. ECC invokes tools linearly with no declarative composition.

### 3. Plugin lifecycle: resume / persist / delegate (Claw Code)
Claw's plugin lifecycle includes distinct `resume`, `persist`, and `delegate` phases with automatic session-state restoration. ECC has SessionStart/SessionEnd but no formal persist/resume contract — session continuity is ad-hoc via `catchup`.

### 4. Provider abstraction layer (Claw + Goose)
Both competitors run against any LLM via a provider layer. ECC is Claude-exclusive by harness design. While this is correct positioning (ECC = "Everything *Claude* Code"), a provider-abstract research prototype could reveal which ECC features are Claude-specific vs transferable.

### 5. MCP-first extension model (Goose)
Goose treats MCP as the primary extension mechanism (3k+ registered servers in 2026). ECC treats MCP as optional config (`.claude.json`). Reframing extensions around MCP would leverage the growing ecosystem.

### 6. Agentic AI Foundation alignment (Goose)
Goose is a founding member of LF's AAIF alongside MCP and Agents.md. ECC could align with AAIF standards for interoperability signaling without giving up its Claude-specific niche.

### 7. Agentic loop pattern (Goose)
Goose's core loop is: plan → select tools → execute → evaluate → loop-or-exit. ECC's `/implement` TDD loop is closer to this than the spec pipeline, but lacks the explicit self-evaluation step between iterations.

## Patterns Worth Adopting

| Priority | Pattern | Target Component | Rationale |
|----------|---------|------------------|-----------|
| **HIGH** | Declarative tool manifest (YAML) | `agents/*.md` frontmatter → `ecc-agents` manifest crate | Eliminates per-agent `allowedTools` drift; enables compile-time validation |
| **HIGH** | AAIF / AGENTS.md standard alignment | `skills/ecc-component-authoring/` | Interoperability with Goose/Claw ecosystem without Claude lock-in loss |
| **MEDIUM** | Plugin resume/persist/delegate lifecycle | Hook system + `catchup` skill | Formalizes session continuity that's currently ad-hoc |
| **MEDIUM** | Agentic self-evaluation step in TDD loop | `/implement` phase | Explicit "did this iteration improve the spec?" check between PCs |
| **LOW** | MCP DAG composition via petgraph | New skill/crate | Novel but high implementation cost; monitor ecosystem first |
| **LOW** | Provider abstraction research prototype | Experimental branch | Defer until Claude Code harness itself abstracts providers |

## Follow-Up Backlog Entries (to create)

- **BL-NNN:** Declarative tool manifest — migrate `allowedTools` from agent frontmatter to a central YAML/TOML manifest with Serde validation
- **BL-NNN:** AGENTS.md / AAIF standard alignment audit — assess ECC agent format against emerging standard, document gaps
- **BL-NNN:** Formalize session resume/persist/delegate hooks — promote `catchup` patterns to first-class hook lifecycle
- **BL-NNN:** Agentic self-evaluation step in `/implement` — add "iteration-improves-spec?" check after each PC

## Sources

- [Claw Code repository (HarnessLab)](https://github.com/HarnessLab/claw-code-agent)
- [Claw Code landing page](https://claw-code.codes/)
- [openclaw/AGENTS.md](https://github.com/openclaw/openclaw/blob/main/AGENTS.md)
- [Goose repository (block/goose)](https://github.com/block/goose)
- [Goose architecture docs](https://block.github.io/goose/docs/goose-architecture/)
- [Block Open Source — Goose announcement](https://block.xyz/inside/block-open-source-introduces-codename-goose)
- [Linux Foundation AAIF (MCP + Goose + Agents.md)](https://www.solo.io/blog/aaif-announcement-agentgateway)
