# ADR 0042: Context Pre-Hydration Pattern

## Status
Accepted

## Context
ECC commands (/spec, /design, /implement) waste early agent tokens on context discovery — reading files, detecting toolchain, checking workflow state. Stripe's Minions architecture demonstrates that deterministic context pre-fetching before the agent loop significantly reduces token waste and improves first-response quality.

## Decision
Implement a UserPromptSubmit hook (`pre:prompt:context-hydrate`) that deterministically pre-fetches context before ECC commands run. The hook detects project type, reads workflow state, and builds per-command context blocks (spec: git log + backlog count, design: spec summary + arch pointers, implement: design summary + test paths). Tool subsetting recommendations are included per command type. The hook uses the existing `HookResult::warn` pattern to inject context via stderr, following the `pre:prompt:context-inject` precedent. Registered with `async: true, timeout: 5` for non-blocking execution.

## Consequences
- Commands start with rich context — skip discovery phase, save tokens
- Per-command tool suggestions focus Claude on relevant tools
- Graceful degradation: fresh sessions still get project type + toolchain
- Pattern is extensible to new command types and non-Rust projects
- Hook is disableable via ECC_WORKFLOW_BYPASS or hooks.json removal
