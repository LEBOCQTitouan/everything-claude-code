---
name: review-team
description: Multi-perspective code review team
coordination: sequential
agents:
  - name: arch-reviewer
    role: Architecture quality gate — layering and dependency direction
    allowed-tools: ["Read", "Grep", "Glob", "Bash"]
  - name: code-reviewer
    role: Code quality — SOLID, naming, complexity, maintainability
    allowed-tools: ["Read", "Grep", "Glob", "Bash"]
  - name: security-reviewer
    role: Security scan — OWASP top 10, injection, secrets, permissions
    allowed-tools: ["Read", "Write", "Edit", "Bash", "Grep", "Glob"]
---

# Review Team

Team manifest for multi-perspective review. Agents execute sequentially:
architecture review first, then code review, then security review. Each
reviewer's findings are available to the next via the shared state protocol.
