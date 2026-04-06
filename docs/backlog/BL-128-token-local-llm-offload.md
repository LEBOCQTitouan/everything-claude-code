---
id: BL-128
title: "Token optimization wave 5 — local LLM offload for mechanical agents"
status: open
created: 2026-04-06
promoted_to: ""
tags: [token-optimization, local-llm, ollama, mcp, cost]
scope: MEDIUM
target_command: /spec-dev
dependencies: [BL-121, BL-126]
---

## Optimized Prompt

Implement local LLM offload infrastructure and migrate 4 agent categories from the BL-121 audit (Wave 5):

1. **Ollama MCP server integration** — add an MCP server config that routes to a local Ollama instance. Add model routing to `~/.ecc/config.toml`: `local-model = "mistral:7b-instruct"`, `local-model-large = "qwen2.5:14b-instruct"`. Graceful fallback to Haiku if Ollama unavailable (connection refused → log warning, use hosted model).
2. **Cartography trio on 7B** — `cartography-flow-generator`, `cartography-journey-generator`, `cartographer` all do schema-fill from structured delta JSON with GAP markers. Add frontmatter field `local-eligible: true` and route to local 7B via MCP when available. Requirements: Ollama, ~4GB RAM.
3. **Diagram agents on 13B** — `diagram-updater`, `diagram-generator` generate Mermaid syntax requiring reliable bracket handling. Route to local 13B. Existing mmdc validation retry loop (max 3 attempts) compensates for model imprecision. Requirements: Ollama, ~8GB RAM.
4. **Convention auditor on 13B** — currently Sonnet but analysis is grep-driven; model only aggregates tool output and applies threshold rules. Route to local 13B.

**Hard exclusion reminder:** reasoning agents (adversary, planner, uncle-bob, architect, code-reviewer, security-reviewer) stay on Claude. Only template-fill, schema-instantiation, and tool-output-aggregation tasks qualify.

Setup documentation: add `docs/guides/local-llm-setup.md` covering Ollama installation, model pull commands, MCP server configuration, and verification steps.

Reference: `docs/audits/token-optimization-2026-04-06.md` findings 2.4-2.9.

## Original Input

BL-121 audit Wave 5: Ollama MCP integration, cartography on 7B, diagrams on 13B, convention-auditor on 13B.

## Challenge Log

**Source:** BL-121 token optimization audit (2026-04-06). Pre-challenged during audit — each agent evaluated for reasoning requirement; doc-generator excluded (language-specific syntax needs).
