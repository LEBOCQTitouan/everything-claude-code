# Tasks: Graceful Mid-Session Exit + Implement Context Clear (BL-055, BL-054)

## Pass Conditions

- [ ] PC-001: context-persist.sh atomic write + session ID sanitization | pending@2026-03-23T10:00:00Z
- [ ] PC-002: context-persist.sh PPID fallback | pending@2026-03-23T10:00:00Z
- [ ] PC-003: context-persist.sh user-private runtime dir | pending@2026-03-23T10:00:00Z
- [ ] PC-004: read-context-percentage.sh "unknown" for missing file | pending@2026-03-23T10:00:00Z
- [ ] PC-005: read-context-percentage.sh valid percentage | pending@2026-03-23T10:00:00Z
- [ ] PC-006: read-context-percentage.sh "unknown" for garbage | pending@2026-03-23T10:00:00Z
- [ ] PC-007: read-context-percentage.sh "unknown" for out-of-range | pending@2026-03-23T10:00:00Z
- [ ] PC-008: read-context-percentage.sh sanitizes session ID | pending@2026-03-23T10:00:00Z
- [ ] PC-047: Round-trip writer to reader | pending@2026-03-23T10:00:00Z
- [ ] PC-048: Runtime dir env var consistent | pending@2026-03-23T10:00:00Z
- [ ] PC-009: graceful-exit SKILL.md frontmatter + sections | pending@2026-03-23T10:00:00Z
- [ ] PC-010: skill warn behavior | pending@2026-03-23T10:00:00Z
- [ ] PC-011: skill exit behavior | pending@2026-03-23T10:00:00Z
- [ ] PC-012: skill hard ceiling | pending@2026-03-23T10:00:00Z
- [ ] PC-013: skill /implement Resumption Pointer format | pending@2026-03-23T10:00:00Z
- [ ] PC-014: skill exit message | pending@2026-03-23T10:00:00Z
- [ ] PC-015: implement.md reads context percentage | pending@2026-03-23T10:00:00Z
- [ ] PC-016: implement.md AskUserQuestion clear/continue | pending@2026-03-23T10:00:00Z
- [ ] PC-017: implement.md skips gate on unknown | pending@2026-03-23T10:00:00Z
- [ ] PC-041: implement.md /compact instruction | pending@2026-03-23T10:00:00Z
- [ ] PC-018: implement.md wave checkpoints | pending@2026-03-23T10:00:00Z
- [ ] PC-019: implement.md phase checkpoints | pending@2026-03-23T10:00:00Z
- [ ] PC-020: implement.md 85% exit saves state | pending@2026-03-23T10:00:00Z
- [ ] PC-021: implement.md graceful-exit skill ref | pending@2026-03-23T10:00:00Z
- [ ] PC-042: implement.md 75% warning | pending@2026-03-23T10:00:00Z
- [ ] PC-043: implement.md below threshold silent | pending@2026-03-23T10:00:00Z
- [ ] PC-044: implement.md re-entry Resumption Pointer | pending@2026-03-23T10:00:00Z
- [ ] PC-045: implement.md 85% independent of 75% | pending@2026-03-23T10:00:00Z
- [ ] PC-022: audit-full.md re-entry partial results | pending@2026-03-23T10:00:00Z
- [ ] PC-023: audit-full.md cleanup partial dirs | pending@2026-03-23T10:00:00Z
- [ ] PC-024: audit-orchestrator context checkpoint | pending@2026-03-23T10:00:00Z
- [ ] PC-025: audit-orchestrator partial results | pending@2026-03-23T10:00:00Z
- [ ] PC-026: audit-orchestrator no-campaign fallback | pending@2026-03-23T10:00:00Z
- [ ] PC-027: audit-orchestrator skip completed | pending@2026-03-23T10:00:00Z
- [ ] PC-028: audit-orchestrator all-domains-done | pending@2026-03-23T10:00:00Z
- [ ] PC-046: audit-orchestrator 75% warning | pending@2026-03-23T10:00:00Z
- [ ] PC-029: strategic-compact graceful-exit ref | pending@2026-03-23T10:00:00Z
- [ ] PC-030: campaign-manifest checkpoint docs | pending@2026-03-23T10:00:00Z
- [ ] PC-031: glossary "Graceful Exit" | pending@2026-03-23T10:00:00Z
- [ ] PC-032: glossary "Context Checkpoint" | pending@2026-03-23T10:00:00Z
- [ ] PC-033: ADR 0014 structure | pending@2026-03-23T10:00:00Z
- [ ] PC-034: ADR 0014 content | pending@2026-03-23T10:00:00Z
- [ ] PC-035: ADR 0014 in README | pending@2026-03-23T10:00:00Z
- [ ] PC-036: CHANGELOG BL-055 | pending@2026-03-23T10:00:00Z
- [ ] PC-037: Markdown lint | pending@2026-03-23T10:00:00Z
- [ ] PC-038: Rust build | pending@2026-03-23T10:00:00Z
- [ ] PC-039: Rust tests | pending@2026-03-23T10:00:00Z
- [ ] PC-040: Clippy clean | pending@2026-03-23T10:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-23T10:00:00Z
- [ ] Code review | pending@2026-03-23T10:00:00Z
- [ ] Doc updates | pending@2026-03-23T10:00:00Z
- [ ] Write implement-done.md | pending@2026-03-23T10:00:00Z
