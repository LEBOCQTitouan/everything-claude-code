# Tasks: BL-146 Declarative Tool Manifest

## Pass Conditions

| PC | Phase | Description | Status |
|----|-------|-------------|--------|
| PC-001 | 1 | Parser round-trips canonical YAML | pending |
| PC-002 | 1 | Module doc declares VO | pending |
| PC-003 | 1 | Unknown tool in preset rejected | pending |
| PC-004 | 1 | Duplicate preset key rejected | pending |
| PC-005 | 1 | Duplicate atomic tool rejected | pending |
| PC-006 | 1 | Empty preset rejected | pending |
| PC-007 | 1 | Invalid preset names (7 fixtures) | pending |
| PC-008 | 1 | VALID_TOOLS absent from source | pending |
| PC-009 | 1 | No build references to VALID_TOOLS | pending |
| PC-010 | 1 | Tools superset of legacy | pending |
| PC-011 | 1 | ≥6 presets in manifest | pending |
| PC-012 | 1 | BOM-prefixed manifest parses | pending |
| PC-013 | 1 | Duplicate top-level keys rejected | pending |
| PC-014 | 1 | Oversized manifest rejected | pending |
| PC-070 | 1 | YAML anchors/aliases rejected | pending |
| PC-015 | 2 | Agent with tool-set validates | pending |
| PC-016 | 2 | Unknown preset named in error | pending |
| PC-017 | 2 | Union + WARN on outliers | pending |
| PC-018 | 2 | Legacy "Missing tools" preserved | pending |
| PC-019 | 2 | Command allowed-tool-set validates | pending |
| PC-020 | 2 | Resolver never panics (proptest) | pending |
| PC-021 | 2 | Array tool-set rejected | pending |
| PC-022 | 2 | Missing manifest single error | pending |
| PC-071 | 2 | tool-set value regex enforced | pending |
| PC-023 | 3 | Inline FakeTool fails | pending |
| PC-024 | 3 | Valid preset passes | pending |
| PC-025 | 3 | Manifest parse error once | pending |
| PC-026 | 3 | Meta-test real tree passes | pending |
| PC-027 | 3 | Teams WARN names preset+missing | pending |
| PC-028 | 3 | Path resolver canonical only | pending |
| PC-029 | 3 | Resolver signature no path | pending |
| PC-030 | 3 | Conventions validates allowed-tool-set | pending |
| PC-072 | 3 | Path resolver rejects parent walk + symlinks | pending |
| PC-031 | 4 | ecc validate agents exit 0 | pending |
| PC-032 | 4 | ecc validate conventions exit 0 | pending |
| PC-033 | 4 | Install writes expanded inline tools | pending |
| PC-034 | 4 | Full test suite clean | pending |
| PC-035 | 4 | Pre/post install byte-identical | pending |
| PC-073 | 4 | Atomic write + symlink reject | pending |
| PC-036 | 5 | Team validator cross-refs manifest | pending |
| PC-037 | 5 | 3 team files validate | pending |
| PC-038 | 5 | collect_agent_info resolves tool-set | pending |
| PC-039 | 5 | TeamAgent has allowed_tool_set | pending |
| PC-040 | 6 | Post-migration agents validate | pending |
| PC-041 | 6 | ≥48/59 covered, ≤10 presets | pending |
| PC-042 | 6 | comms-generator normalized | pending |
| PC-043 | 6 | web-scout frontmatter-only | pending |
| PC-044 | 6 | SHA-256 pre/post match | pending |
| PC-045 | 6 | Teams validator byte-identical | pending |
| PC-046 | 6 | Full suite clean | pending |
| PC-047 | 6 | 7 audit commands use preset | pending |
| PC-048 | 6 | 3 comms commands normalized | pending |
| PC-049 | 6 | create-component body unchanged | pending |
| PC-050 | 6 | implement.md validates | pending |
| PC-051 | 6 | Commands + conventions pass | pending |
| PC-052 | 6 | Skills migrated or agnostic | pending |
| PC-053 | 6 | Skill validator enforces tool-set | pending |
| PC-054 | 6 | Skill without tools valid | pending |
| PC-055 | 7 | Install propagation E2E | pending |
| PC-056 | 7 | Meta-test all 4 validators | pending |
| PC-057 | 7 | Non-recursion property test | pending |
| PC-058 | 7 | Research doc claim updated | pending |
| PC-074 | 7 | No tool-set in installed output | pending |
| PC-059 | 8 | ADR exists with cites | pending |
| PC-060 | 8 | Authoring guide exists | pending |
| PC-061 | 8 | CLAUDE.md glossary entry | pending |
| PC-062 | final | ecc-domain clippy clean | pending |
| PC-063 | final | Workspace clippy clean | pending |
| PC-064 | final | Workspace builds | pending |
| PC-065 | final | Formatting clean | pending |
| PC-066 | final | ecc validate agents | pending |
| PC-067 | final | ecc validate commands | pending |
| PC-068 | final | ecc validate teams | pending |
| PC-069 | final | ecc validate conventions | pending |

## Post-TDD

| Task | Status |
|------|--------|
| E2E tests | pending |
| Code review | pending |
| Doc updates | pending |
| Supplemental docs | pending |
| Write implement-done.md | pending |

## Status Trail

- 2026-04-17: tasks.md created, 74 PCs pending
