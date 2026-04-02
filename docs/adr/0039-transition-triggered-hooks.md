# ADR-0039: Replace command-embedded Bash calls with transition-triggered hooks

## Status
Accepted (2026-04-02)

## Context
Workflow state transitions were driven by Bash commands (`!ecc-workflow transition`) embedded in Markdown command files. Claude (an LLM) could skip, forget, or fail to execute these commands, causing workflow state to get stuck.

## Decision
Phase 1 (this implementation): Unify binaries so hooks and workflow share process context. `ecc workflow` delegates to `ecc-workflow` for behavioral parity.

Phase 2 (future): Hooks fire automatically on state transitions as registered Rust trait objects. Commands call `ecc workflow transition` which triggers pre/post hooks in-process.

## Consequences
- Phase 1: behavioral parity maintained; no new hook execution model yet
- Phase 2 will eliminate the "Claude forgot to call transition" failure mode
- Shell hooks remain as escape hatch, wrapped in Rust timeout/exit-code enforcement
- Transition hooks are deterministic (no env var bypass possible)
