---
name: implement-team
description: TDD implementation team for /implement Phase 3
coordination: wave-dispatch
max-concurrent: 2
agents:
  - name: tdd-executor
    role: Implements one Pass Condition per invocation using RED-GREEN-REFACTOR
    allowed-tools: ["Read", "Write", "Edit", "MultiEdit", "Bash", "Grep", "Glob"]
  - name: pc-evaluator
    role: Post-PC self-evaluation — checks AC satisfaction, regressions, spec achievability
    allowed-tools: ["Read", "Grep", "Glob"]
  - name: code-reviewer
    role: Reviews implementation quality after TDD loop completes
    allowed-tool-set: readonly-analyzer-shell
  - name: module-summary-updater
    role: Updates MODULE-SUMMARIES.md with per-crate entries
    allowed-tools: ["Read", "Write", "Edit", "Grep", "Glob"]
  - name: diagram-updater
    role: Generates Mermaid diagrams for cross-module flows
    allowed-tools: ["Read", "Write", "Edit", "Grep", "Glob"]
---

# Implement Team

Team manifest for `/implement` Phase 3 (TDD Loop). The orchestrator dispatches
`tdd-executor` agents in waves controlled by `max-concurrent`. After TDD completes,
`code-reviewer` runs sequentially, then `module-summary-updater` and `diagram-updater`
run in parallel for supplemental docs.

## Coordination Flow

1. **Wave dispatch**: `tdd-executor` agents execute PCs in parallel waves (max 2 concurrent)
2. **Sequential review**: `code-reviewer` reviews all changes after waves complete
3. **Parallel docs**: `module-summary-updater` + `diagram-updater` run concurrently
