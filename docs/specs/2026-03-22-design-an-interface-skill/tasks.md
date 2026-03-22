# Tasks: Create design-an-interface skill + agent (BL-014)

## Pass Conditions

- [ ] PC-001: Skill file exists | `test -f skills/design-an-interface/SKILL.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-002: Skill frontmatter has name | `grep -q '^name: design-an-interface' skills/design-an-interface/SKILL.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-003: Skill frontmatter has description | `grep -q '^description:' skills/design-an-interface/SKILL.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-004: Skill frontmatter has origin: ECC | `grep -q '^origin: ECC' skills/design-an-interface/SKILL.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-005: Skill under 500 words | `wc -w < skills/design-an-interface/SKILL.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-006: Skill has trigger phrases | `grep -q 'design an interface' skills/design-an-interface/SKILL.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-007: Skill has 4 constraints | `grep -cE 'minimize method count|maximize flexibility|optimize.*common case|named paradigm' skills/design-an-interface/SKILL.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-008: Skill has 5 dimensions | `grep -cE 'simplicity|general.purpose.*specialized|implementation efficiency|depth|ease of correct use' skills/design-an-interface/SKILL.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-009: Skill has 3 anti-patterns | `grep -cE 'DO NOT.*similar|DO NOT.*skip|DO NOT.*implement' skills/design-an-interface/SKILL.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-010: Skill references interface-designer agent | `grep -q 'interface-designer' skills/design-an-interface/SKILL.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-011: Agent file exists | `test -f agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-012: Agent has name field | `grep -q '^name: interface-designer' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-013: Agent has model: opus | `grep -q '^model: opus' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-014: Agent has required tools | `grep -E '^tools:' agents/interface-designer.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-015: Agent has skills reference | `grep -q 'skills:.*design-an-interface' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-016: Agent mentions parallel sub-agents | `grep -qi 'parallel' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-017: Agent specifies allowedTools | `grep -q 'allowedTools' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-018: Agent has 4 constraints | `grep -cE 'minimize method count|maximize flexibility|optimize.*common case|named paradigm' agents/interface-designer.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-019: Agent has optional 5th constraint | `grep -qi 'optional.*constraint|5th constraint' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-020: Agent output format complete | `grep -cE 'signature|usage example|hides internally|tradeoffs' agents/interface-designer.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-021: Agent detects language | `grep -qi 'Cargo.toml|package.json|go.mod' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-022: Agent prompts for module | `grep -qi 'prompt.*module|ask.*module|specify.*module' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-023: Agent has TodoWrite with degradation | `grep -q 'TodoWrite' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-024: Agent multi-language asks user | `grep -qi 'multiple.*language|ask.*language|which language' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-025: Agent convergence defined | `grep -qi 'structural pattern.*method.*overlap|method.*overlap.*structural' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-026: Agent re-spawn on convergence | `grep -qi 're-spawn|retry.*converg' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-027: Agent handles sub-agent failure | `grep -qi 'fail.*timeout|proceed.*available' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-028: Agent minimum 2 designs | `grep -qE 'minimum.*(2|two)' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-029: Agent 5 comparison dimensions | `grep -cE 'simplicity|general.purpose.*specialized|implementation efficiency|depth|ease of correct use' agents/interface-designer.md` | pending@2026-03-22T12:45:00Z
- [ ] PC-030: Agent never skip comparison | `grep -qi 'never skip|DO NOT skip|must.*compar' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-031: Agent uses AskUserQuestion | `grep -q 'AskUserQuestion' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-032: Agent graceful degradation AskUserQuestion | `grep -qi 'graceful|fallback|unavailable' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-033: Agent outputs to docs/designs/ | `grep -q 'docs/designs/' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-034: Agent creates directory if missing | `grep -qi 'create.*dir|directory.*not exist' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-035: Agent numeric suffix | `grep -qi 'numeric.*suffix|already exists' agents/interface-designer.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-036: /design references interface-designer | `grep -q 'interface-designer' commands/design.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-037: /design reference is optional | `grep -qi 'optional.*interface-designer|interface-designer.*optional' commands/design.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-038: Glossary has Interface Designer | `grep -q 'Interface Designer' docs/domain/glossary.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-039: CHANGELOG has BL-014 | `grep -q 'BL-014' CHANGELOG.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-040: ADR 0008 exists | `test -f docs/adr/0008-designs-directory-convention.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-041: ADR has correct structure | `grep -q '## Status' docs/adr/0008-designs-directory-convention.md && echo PASS` | pending@2026-03-22T12:45:00Z
- [ ] PC-042: Lint passes | `cargo clippy -- -D warnings` | pending@2026-03-22T12:45:00Z
- [ ] PC-043: Build passes | `cargo build` | pending@2026-03-22T12:45:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-22T12:45:00Z
- [ ] Code review | pending@2026-03-22T12:45:00Z
- [ ] Doc updates | pending@2026-03-22T12:45:00Z
- [ ] Write implement-done.md | pending@2026-03-22T12:45:00Z
