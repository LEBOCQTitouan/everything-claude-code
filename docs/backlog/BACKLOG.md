# Backlog Index

| ID | Title | Tier | Scope | Target | Status | Created |
|----|-------|------|-------|--------|--------|---------|
| BL-001 | Block auto-enable of MCP servers | 1 | LOW | direct edit | open | 2026-03-20 |
| BL-002 | Pin all MCP package versions | 1 | LOW | direct edit | open | 2026-03-20 |
| BL-003 | Prune stale local permissions | 1 | LOW | direct edit | open | 2026-03-20 |
| BL-004 | robert: read-only + memory + negative examples | 2 | MEDIUM | direct edit | open | 2026-03-20 |
| BL-005 | Update commands that call robert to handle his output | 2 | MEDIUM | direct edit | open | 2026-03-20 |
| BL-006 | spec-adversary: skills preload + negative examples | 2 | LOW | direct edit | open | 2026-03-20 |
| BL-007 | solution-adversary: skills preload + negative examples | 2 | LOW | direct edit | open | 2026-03-20 |
| BL-008 | drift-checker: skills preload | 2 | LOW | direct edit | open | 2026-03-20 |
| BL-009 | Add negative examples to planner agent | 2 | LOW | direct edit | open | 2026-03-20 |
| BL-010 | Create ubiquitous-language skill | 3 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-011 | Create grill-me skill | 3 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-012 | Create write-a-prd skill | 3 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-013 | Create interview-me skill | 3 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-014 | Create design-an-interface skill | 3 | HIGH | /plan dev | open | 2026-03-20 |
| BL-015 | Create request-refactor-plan skill | 3 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-016 | Create prd-to-plan skill | 3 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-017 | Create /catchup command | 4 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-019 | Create /spec command | 4 | MEDIUM | /plan dev | open | 2026-03-20 |
| BL-020 | Create /design command | 4 | LOW | direct edit | open | 2026-03-20 |
| BL-021 | Extract command reference tables from CLAUDE.md | 5 | LOW | direct edit | open | 2026-03-20 |
| BL-022 | Replace CLAUDE.md architecture block with pointer | 5 | LOW | direct edit | open | 2026-03-20 |
| BL-023 | Clean up stale workflow state | 5 | LOW | direct edit | open | 2026-03-20 |
| BL-024 | Add context:fork to heavy skills | 6 | LOW | direct edit | open | 2026-03-20 |
| BL-025 | Add memory:project to adversarial agents | 6 | LOW | direct edit | open | 2026-03-20 |
| BL-026 | Quarterly MCP version audit | 6 | LOW | process | open | 2026-03-20 |
| BL-027 | Cross-session memory system for actions, plans, and implementations | 7 | HIGH | /plan dev | open | 2026-03-21 |

## Dependency Graph

```
BL-004 → BL-005 (robert read-only requires command updates)
BL-010 → BL-004 (ubiquitous-language skill enables robert skill preload)
BL-012 → BL-016 (prd-to-plan consumes write-a-prd output)
BL-014 → BL-020 (/design command wraps design-an-interface skill)
BL-002 → BL-026 (quarterly audit requires initial pinning)
BL-017 → BL-023 (/catchup prevents stale state recurrence)
BL-027 → BL-017 (memory system feeds /catchup command)
BL-027 → BL-004 (memory system feeds robert negative examples)
BL-025 → BL-027 (per-agent memory flags complement cross-session log)
```

## Stats

- **Total:** 26
- **Open:** 26
- **By tier:** T1: 3 | T2: 6 | T3: 7 | T4: 3 | T5: 3 | T6: 3 | T7: 1
