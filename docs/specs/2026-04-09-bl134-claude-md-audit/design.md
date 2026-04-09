# Design: Audit CLAUDE.md for Inferable Content (BL-134)

## Revision
Revised 2026-04-09: Narrowed scope — only remove unused inferable sections.

## Spec Reference
\`docs/specs/2026-04-09-bl134-claude-md-audit/spec.md\`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | \`CLAUDE.md\` | Remove Doc Hierarchy, Dev Notes, Slash Commands listing, 2 inferable gotchas | Root | AC-001.1–AC-001.12 |

## Sections to Remove

1. **Slash Commands** listing (lines 55-57) — just a pointer to commands-reference.md, no directives
2. **Doc Hierarchy** (lines 74-76) — describes doc organization, agents find files by searching
3. **Development Notes** (lines 114-121) — repeats architecture info, states obvious facts
4. **Gotcha: hooks.json location** (line 93) — discoverable: \`ls hooks/hooks.json\`
5. **Gotcha: skill naming** (line 94) — discoverable from inspecting skills/ directory

## Sections to Keep

- Running Tests, CLI Commands, Architecture — pipeline-consumed
- Spec-Driven Pipeline, Command Workflows, Dual-Mode — policy rules
- All other Gotchas — non-inferable
- Glossary — domain vocabulary

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | verify | Doc Hierarchy removed | AC-001.1 | \`! grep -q '## Doc Hierarchy' CLAUDE.md\` | exit 0 |
| PC-002 | verify | Dev Notes removed | AC-001.2 | \`! grep -q '## Development Notes' CLAUDE.md\` | exit 0 |
| PC-003 | verify | Slash Commands listing removed | AC-001.3 | \`! grep -q 'Audit commands (\`/audit-full\`' CLAUDE.md\` | exit 0 |
| PC-004 | verify | hooks.json gotcha removed | AC-001.4 | \`! grep -q 'hooks.json.*lives in' CLAUDE.md\` | exit 0 |
| PC-005 | verify | skill naming gotcha removed | AC-001.5 | \`! grep -q 'Skill directory name must match' CLAUDE.md\` | exit 0 |
| PC-006 | verify | Running Tests preserved | AC-001.6 | \`grep -q '## Running Tests' CLAUDE.md\` | exit 0 |
| PC-007 | verify | CLI Commands preserved | AC-001.7 | \`grep -q '## CLI Commands' CLAUDE.md\` | exit 0 |
| PC-008 | verify | Architecture preserved | AC-001.8 | \`grep -q '## Architecture' CLAUDE.md\` | exit 0 |
| PC-009 | verify | Gotchas preserved | AC-001.9 | \`grep -q 'Brevity rule' CLAUDE.md\` | exit 0 |
| PC-010 | verify | Glossary preserved | AC-001.10 | \`grep -q 'Glossary:' CLAUDE.md\` | exit 0 |
| PC-011 | verify | Line count reduced >= 15 | AC-001.11 | \`test \$(wc -l < CLAUDE.md) -le 105\` | exit 0 |
| PC-012 | verify | ecc validate passes | AC-001.12 | \`ecc validate claude-md 2>&1; echo exit:\$?\` | exit:0 |

## Coverage Check

All 12 ACs covered by PC-001 through PC-012.

## Test Strategy

Single wave: edit CLAUDE.md, verify all 12 PCs.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | root | Add entry | BL-134 CLAUDE.md audit — removed unused inferable content | US-001 |

## SOLID Assessment

**PASS.** Doc-only change.

## Robert's Oath Check

**CLEAN.** Measured removal of clutter.

## Security Notes

**CLEAR.** No code changes.

## Rollback Plan

\`git revert\` the single CLAUDE.md commit.

## Bounded Contexts Affected

No bounded contexts affected.
