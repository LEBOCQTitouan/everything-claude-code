---
id: BL-121
title: Token optimization audit — comprehensive spend reduction opportunity mapping
status: implemented
created: 2026-04-05
promoted_to: ""
tags: [token-optimization, cost, audit, agents, commands, skills, hooks, rust-cli]
scope: EPIC
target_command: /audit
dependencies: [BL-095, BL-096]
---

## Optimized Prompt

Run a comprehensive token optimization audit across all ECC components to identify every opportunity to reduce spend per session and session count.

**Context:** ECC is a Rust CLI + agents/skills/commands/hooks/rules for Claude Code. 9 Rust crates, hexagonal architecture. Cost tracking (BL-096) and thinking budget tuning (BL-095) must be implemented first to supply before/after baselines.

**Scope of the audit:** Inspect every component in `agents/`, `commands/`, `skills/`, `hooks/`, `rules/` and the Rust CLI for token waste. Cover all four optimization axes:

1. **Offload mechanical work to the Rust CLI** — token counting, diff summarization, post-processing, file filtering, deduplication, regex matching, template expansion, and any task that is purely deterministic and has zero reasoning requirement.

2. **Offload simple/repetitive tasks to local LLMs or self-hosted MCP servers** — small classification tasks, lightweight summarization, boilerplate generation, or any subtask where a 7B–13B local model (Ollama, LM Studio, or equivalent) is sufficient. Include setup instructions as part of findings.

3. **Reduce context window bloat** — redundant skill loads across commands, oversized system prompts, repeated boilerplate in agent frontmatter, unnecessary file reads, stale context carried across stages, duplicate rule injection, and any pattern that inflates the context window without adding value.

4. **Reduce session count** — identify workflows that require multiple Claude sessions where a single well-structured session would suffice, and prompting patterns that trigger unnecessary session splits.

**Hard exclusions — do NOT flag for offloading:**
- Reasoning tasks: adversary, planner, uncle-bob, architect, spec-adversary, solution-adversary, requirements-analyst
- Security analysis: security-reviewer
- Architecture review: arch-reviewer, code-reviewer
- Any task where judgment, creativity, or multi-step reasoning is required

**Audit output format:** A prioritized report listing every finding. Each finding must include:
- Component name and file path
- Optimization axis (CLI offload / local LLM / context bloat / session count)
- Current behavior and why it wastes tokens
- Proposed fix (concrete, actionable)
- Estimated impact: HIGH / MEDIUM / LOW
- Implementation complexity: TRIVIAL / LOW / MEDIUM / HIGH

**Post-audit action:** Each finding or logical cluster of related findings becomes a separate backlog entry (or chunk) for `/spec`. Do not implement anything during the audit — only produce the report.

**Acceptance criteria:**
- All files in `agents/`, `commands/`, `skills/`, `hooks/`, `rules/` inspected
- Rust CLI commands reviewed for offload candidates
- Every finding categorized by axis and impact
- Report is sorted by impact (HIGH first) within each axis
- No reasoning/judgment tasks flagged for offloading
- Local LLM findings include setup requirements (model size, server type)
- Report concludes with a recommended execution order for follow-up `/spec` entries

**Verification:**
- Report covers all 4 optimization axes
- Every HIGH-impact finding has a concrete proposed fix
- No false positives in the exclusion list (no reasoning tasks flagged)

## Original Input

Token optimization audit for ECC — identify every opportunity to reduce spend per session and session count by offloading mechanical work to Rust CLI, routing simple tasks to local LLMs or self-hosted MCP servers, eliminating context window bloat, and reducing unnecessary session count. Audit only — output a prioritized report, then each finding becomes a separate backlog entry or /spec chunk.

## Challenge Log

**Mode:** backlog-mode (escalated to EPIC — all 5 stages)
**Depth profile:** standard

**Q1 [Clarification]:** Token optimization = both spend per session AND session count reduction equally, or one takes priority?
**A:** Both equally — spend per session and session count reduction are equally important goals.

**Q2 [Clarification]:** Self-hosted = local LLMs only, or also self-hosted MCP servers (e.g., custom tool servers)?
**A:** Local LLMs (Ollama, LM Studio) AND possibly self-hosted MCP servers. Findings should include setup instructions.

**Q3 [Assumption]:** Is cost tracking infrastructure (BL-096) and thinking budget tuning (BL-095) already implemented, or does this audit need to work without baseline data?
**A:** BL-095 and BL-096 are prerequisites — must be implemented first. No cost data exists yet.

**Q4 [Implication]:** Which specific mechanical tasks are strong candidates for Rust CLI offload — diff summarization, token counting, post-processing, file filtering?
**A:** All of those plus more — offload maximum possible to Rust CLI without losing quality.

**Q5 [Assumption]:** Is local LLM offload for reasoning tasks (adversary, planner, uncle-bob) acceptable, or is Claude required for all reasoning?
**A:** All reasoning stays with Claude — no-go for offloading reasoning to local LLMs or Rust.

**Q6 [Evidence]:** Do you have existing cost data to prioritize which agents/commands are highest spenders?
**A:** No cost data yet — reinforces BL-095/096 as prerequisites.

**Q7 [Viewpoint]:** Should this be one audit entry producing a report, or split into sub-entries per optimization axis from the start?
**A:** One audit entry (part a) producing a prioritized report. Then separate entries per optimization cluster after the report is produced.

**Q8 [Implication]:** After the audit report, are findings implemented one by one or batched into clusters per optimization axis?
**A:** Every finding listed first, then implemented one by one or by chunk via /spec.

## Related Backlog Items

- BL-094: Agent model routing optimization (model alignment — prerequisite context)
- BL-095: Extended thinking and effort tuning (prerequisite — must be implemented first)
- BL-096: Cost and token tracking (prerequisite — must be implemented first)
