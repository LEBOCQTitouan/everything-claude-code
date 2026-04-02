---
name: audit-team
description: Full codebase audit team for /audit-full
coordination: parallel
agents:
  - name: evolution-analyst
    role: Git history analysis — hotspots, coupling, bus factor
    allowed-tools: ["Read", "Bash", "Grep", "Glob"]
  - name: arch-reviewer
    role: Architecture quality — layering, DDD, hexagonal compliance
    allowed-tools: ["Read", "Grep", "Glob", "Bash"]
  - name: component-auditor
    role: Component principles — instability, abstractness, main sequence
    allowed-tools: ["Read", "Grep", "Glob", "Bash"]
  - name: error-handling-auditor
    role: Error handling — swallowed errors, taxonomy, boundaries
    allowed-tools: ["Read", "Bash", "Grep", "Glob"]
  - name: convention-auditor
    role: Convention consistency — naming, patterns, config access
    allowed-tools: ["Read", "Bash", "Grep", "Glob"]
  - name: test-auditor
    role: Test architecture — classification, coupling, coverage
    allowed-tools: ["Read", "Bash", "Grep", "Glob"]
  - name: observability-auditor
    role: Observability — logging, metrics, correlation, health
    allowed-tools: ["Read", "Bash", "Grep", "Glob"]
---

# Audit Team

Team manifest for `/audit-full`. All domain auditors run in parallel, then
the orchestrator cross-correlates findings across domains.
