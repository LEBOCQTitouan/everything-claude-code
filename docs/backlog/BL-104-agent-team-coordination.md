---
id: BL-104
title: "Multi-agent team coordination — shared state and task handoff"
scope: HIGH
target: "/spec-dev"
status: implemented
tags: [agents, orchestration, teams, coordination]
created: 2026-03-29
related: [BL-065, BL-093]
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-104: Multi-Agent Team Coordination

## Problem

ECC supports parallel subagents but lacks coordinated team patterns with shared state and task handoff. Claude Code introduced Agent Teams (Feb 2026) for collaborative multi-agent squads. ACM TOSEM survey (124 papers) confirms viability of role-aligned agent teams (ALMAS framework).

## Proposed Solution

Add agent team coordination capabilities:
- Shared task list between team members (beyond TodoWrite)
- Role-based agent teams (architect + developer + reviewer)
- Task handoff protocols between agents
- Team-level progress tracking and reporting

## Ready-to-Paste Prompt

```
/spec-dev Add multi-agent team coordination to ECC:

1. Team Definition Format
   - Define team composition in a team manifest (Markdown + frontmatter)
   - Specify roles, responsibilities, and handoff protocols per team member
   - Support pre-defined teams: "review-team", "implement-team", "audit-team"

2. Shared State Protocol
   - Shared task list accessible by all team members
   - Artifact exchange format between agents (structured JSON)
   - Progress aggregation across team members

3. Task Handoff
   - Define handoff triggers (phase completion, quality gate pass)
   - Support both sequential handoff (architect → developer) and
     parallel fan-out (multiple reviewers)

4. Integration
   - Extend /implement to use implement-team by default
   - Extend /audit-full to use audit-team by default

Reference: Claude Code Agent Teams (Feb 2026), ALMAS framework (ACM TOSEM)
Source: docs/audits/web-radar-2026-03-29-r2.md
```
