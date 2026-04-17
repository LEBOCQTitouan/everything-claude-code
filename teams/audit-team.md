---
name: audit-team
description: Full codebase audit team for /audit-full
coordination: parallel
agents:
  - name: evolution-analyst
    role: Git history analysis — hotspots, coupling, bus factor
    allowed-tool-set: readonly-analyzer-shell
  - name: arch-reviewer
    role: Architecture quality — layering, DDD, hexagonal compliance
    allowed-tool-set: readonly-analyzer-shell
  - name: component-auditor
    role: Component principles — instability, abstractness, main sequence
    allowed-tool-set: readonly-analyzer-shell
  - name: error-handling-auditor
    role: Error handling — swallowed errors, taxonomy, boundaries
    allowed-tool-set: readonly-analyzer-shell
  - name: convention-auditor
    role: Convention consistency — naming, patterns, config access
    allowed-tool-set: readonly-analyzer-shell
  - name: test-auditor
    role: Test architecture — classification, coupling, coverage
    allowed-tool-set: readonly-analyzer-shell
  - name: observability-auditor
    role: Observability — logging, metrics, correlation, health
    allowed-tool-set: readonly-analyzer-shell
---

# Audit Team

Team manifest for `/audit-full`. All domain auditors run in parallel, then
the orchestrator cross-correlates findings across domains.
