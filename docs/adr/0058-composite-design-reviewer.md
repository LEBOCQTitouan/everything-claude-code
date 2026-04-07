# ADR 0058: Composite Design Reviewer

## Status

Accepted

## Context

The `/design` command launches three sequential read-only review agents — `uncle-bob` (SOLID/Clean Architecture), `robert` (Programmer's Oath), and `security-reviewer` (design-level security scan). Each runs in a separate subagent context with identical tool access (`[Read, Grep, Glob, Bash]`) and no data dependency between them. This triples the context overhead for design review.

## Decision

Merge the three review passes into a single `design-reviewer` composite agent that evaluates all three dimensions in one context. The output uses labeled sections (`## SOLID Assessment`, `## Oath Evaluation`, `## Security Notes`) so findings remain distinguishable by dimension.

The original agents (`uncle-bob`, `robert`, `security-reviewer`) remain in the repository for standalone use — `/review` still invokes `robert` directly, and other commands may reference individual agents.

## Consequences

- **Positive**: Reduces `/design` from 3 subagent contexts to 1 (2 fewer API roundtrips, ~40% less design-phase token spend)
- **Positive**: Unified report is easier to review than 3 separate outputs
- **Negative**: A single agent failure blocks all three dimensions (no partial results)
- **Mitigation**: On composite agent failure, fall back to sequential 3-agent pattern
