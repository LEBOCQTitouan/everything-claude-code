# Tasks: Context-Aware Doc Generation at End of /implement (BL-056)

## Pass Conditions

- [x] PC-001: module-summary-updater name field | `head -10 agents/module-summary-updater.md | grep -c 'name: module-summary-updater'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:05:00Z
- [x] PC-002: module-summary-updater model haiku | `head -10 agents/module-summary-updater.md | grep -c 'model: haiku'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:05:00Z
- [x] PC-003: module-summary-updater tools list | `head -10 agents/module-summary-updater.md | grep -cE 'tools:.*Read.*Write.*Edit.*Grep.*Glob'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:05:00Z
- [x] PC-004: IMPLEMENT-GENERATED markers in agent | `grep -c 'IMPLEMENT-GENERATED' agents/module-summary-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:05:00Z
- [x] PC-005: Module defined as Rust crate | `grep -ci 'rust crate\|cargo workspace' agents/module-summary-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:05:00Z
- [x] PC-006: diagram-updater name field | `head -10 agents/diagram-updater.md | grep -c 'name: diagram-updater'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-007: diagram-updater model haiku | `head -10 agents/diagram-updater.md | grep -c 'model: haiku'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-008: diagram-updater tools list | `head -10 agents/diagram-updater.md | grep -cE 'tools:.*Read.*Write.*Edit.*Grep.*Glob'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-009: Header uses diagram-updater name | `grep -c 'diagram-updater' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-010: MUST NOT modify module-dependency-graph | `grep -ci 'must not modify.*module-dependency-graph\|never.*module-dependency-graph' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-011: MUST NOT register in CUSTOM.md | `grep -ci 'must not.*CUSTOM.md\|never.*CUSTOM.md' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-012: Concrete trigger heuristics | `grep -c '2+ crates' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:07:00Z
- [x] PC-013: implement.md has Phase 7.5 section | `grep -c 'Phase 7.5' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-014: Supplemental docs mentioned 3+ times | `grep -ci 'supplemental doc' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-015: TodoWrite has Supplemental docs | `grep -c 'Supplemental docs' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-016: Schema has ## Supplemental Docs | `grep -c '## Supplemental Docs' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-017: Table schema columns correct | `grep -c 'Subagent.*Status.*Output File.*Commit SHA.*Notes' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-018: Phase 8 checks Supplemental Docs | `sed -n '/Phase 8/,$ p' commands/implement.md | grep -c 'Supplemental Docs'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-019: Related Agents: module-summary-updater | `grep -c 'module-summary-updater' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-020: Related Agents: diagram-updater | `grep -c 'diagram-updater' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-021: implement.md under 800 lines | `test $(wc -l < commands/implement.md) -lt 800 && echo PASS` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-022: Parallel dispatch with allowedTools | `grep -c 'allowedTools' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-023: Cross-link fixup mentioned | `grep -ci 'cross-link fixup\|fixup pass' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-024: Non-blocking failure handling | `grep -ci 'non-blocking\|partial failure' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-025: Context checkpoint in Phase 7.5 | `grep -A5 'Phase 7.5' commands/implement.md | grep -ci 'context checkpoint\|graceful-exit'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:09:00Z
- [x] PC-026: Hook checks Supplemental Docs | `grep -c 'Supplemental Docs' .claude/hooks/doc-enforcement.sh` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:10:00Z
- [x] PC-027: Hook emits WARNING | `grep -A2 'Supplemental Docs' .claude/hooks/doc-enforcement.sh | grep -c 'WARNING'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:10:00Z
- [x] PC-028: Hook exits 0 on warning | `grep -A4 'Supplemental Docs' .claude/hooks/doc-enforcement.sh | grep -c 'exit 0'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:10:00Z
- [x] PC-029: Hook has ECC_WORKFLOW_BYPASS | `head -5 .claude/hooks/doc-enforcement.sh | grep -c 'ECC_WORKFLOW_BYPASS'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:10:00Z
- [x] PC-030: Hook has set -uo pipefail | `head -5 .claude/hooks/doc-enforcement.sh | grep -c 'set -uo pipefail'` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:10:00Z
- [x] PC-031: INDEX.md has Feature Implementation Diagrams | `grep -c '## Feature Implementation Diagrams' docs/diagrams/INDEX.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:11:00Z
- [x] PC-032: INDEX.md explains Phase 7.5 origin | `grep -ci 'Phase 7.5\|/implement' docs/diagrams/INDEX.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:11:00Z
- [x] PC-033: Section order correct | `awk '/Command Workflow/{a=1} /Feature Implementation/{if(a) b=1} /Coverage/{if(b) print "PASS"}' docs/diagrams/INDEX.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:11:00Z
- [x] PC-034: Coverage table includes new category | `grep -c 'Feature implementation' docs/diagrams/INDEX.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:11:00Z
- [x] PC-035: ADR 0015 exists | `test -f docs/adr/0015-feature-implementation-diagrams.md && echo PASS` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:12:00Z
- [x] PC-036: Glossary has Supplemental Docs | `grep -c '### Supplemental Docs' docs/domain/glossary.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:12:00Z
- [x] PC-037: CHANGELOG has BL-056 | `grep -c 'BL-056' CHANGELOG.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:13:00Z
- [x] PC-038: Markdown lint passes | `npx markdownlint-cli2 <files>` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:14:00Z
- [x] PC-039: Cargo clippy passes | `cargo clippy -- -D warnings` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:14:00Z
- [x] PC-040: Cargo build passes | `cargo build` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:14:00Z
- [x] PC-041: CUSTOM.md unchanged | `git diff --name-only docs/diagrams/CUSTOM.md | wc -l` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z
- [x] PC-042: All 4 diagram types in agent | `grep -c 'sequence\|flowchart\|dependency\|C4\|component' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z
- [x] PC-043: Format template in agent | `grep -c 'Generated by diagram-updater\|mermaid' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z
- [x] PC-044: All 3 trigger heuristics | `grep -c '2+ crates\|3+ variants\|new crate' agents/diagram-updater.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z
- [x] PC-045: Cross-link fixup with no-op and commit | `grep -c 'cross-link\|fixup\|no diagram' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z
- [x] PC-046: Supplemental Docs placement in schema | `awk '/ADRs Created/{a=1} /Supplemental Docs/{if(a) b=1} /Subagent Execution/{if(b) print "PASS"}' commands/implement.md` | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z
- [x] PC-047: Hook detects missing Supplemental Docs | integration test | pending@2026-03-23T13:00:00Z | done@2026-03-23T13:15:00Z

## Post-TDD

- [x] E2E tests | done@2026-03-23T13:16:00Z
- [x] Code review | done@2026-03-23T13:17:00Z
- [x] Doc updates | done@2026-03-23T13:18:00Z
- [x] Write implement-done.md | done@2026-03-23T13:19:00Z
