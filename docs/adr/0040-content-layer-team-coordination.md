# ADR-0040: Content-Layer Team Coordination over Rust Execution Engine

## Status
Accepted (2026-04-02)

## Context
ECC needs multi-agent team coordination. Two approaches were considered:
1. Build a generic team execution engine in Rust that reads manifests and orchestrates agent dispatch
2. Define team coordination as content-layer artifacts (Markdown manifests, skills) with minimal Rust validation

Claude Code already provides native Agent Teams primitives (shared task lists, mailbox messaging, teammate hooks). ECC already has wave dispatch, tasks.md, campaign.md, and phase-gated handoffs.

## Decision
Implement team coordination as content-layer artifacts:
- Team manifests (`teams/*.md`) with YAML frontmatter defining composition, roles, and coordination strategy
- Skills (`shared-state-protocol`, `task-handoff`) codifying existing coordination patterns
- Commands read manifests for agent configuration (with `ECC_LEGACY_DISPATCH=1` fallback)
- Rust adds only manifest validation (`ecc validate teams`) with strict agent cross-referencing

## Consequences
**Positive**: Zero new port traits. Zero new adapters. Follows existing content patterns. Low blast radius. Leverages Claude Code native runtime.

**Negative**: Coordination logic lives in natural language (Markdown), not type-checked Rust. Manifest schema is structural only — semantic validation limited to agent name cross-referencing and tool escalation checks.

**Deferred**: Generic team execution engine if demand materializes. Claude Code Agent Teams native integration (behind experimental flag).
