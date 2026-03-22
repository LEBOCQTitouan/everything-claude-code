# Tasks: Campaign Manifest for Amnesiac Agents (BL-035)

## Pass Conditions

- [ ] PC-001: artifact-schemas skill frontmatter + 5 schemas | `grep -q "^name: artifact-schemas" skills/artifact-schemas/SKILL.md && grep -q "spec.md" skills/artifact-schemas/SKILL.md && grep -q "design.md" skills/artifact-schemas/SKILL.md && grep -q "tasks.md" skills/artifact-schemas/SKILL.md && grep -q "state.json" skills/artifact-schemas/SKILL.md && grep -q "campaign.md" skills/artifact-schemas/SKILL.md && grep -q "^origin: ECC" skills/artifact-schemas/SKILL.md && echo PASS` | pending@2026-03-22T22:58:00Z
- [ ] PC-002: campaign-manifest skill frontmatter + schema sections | `grep -q "^name: campaign-manifest" skills/campaign-manifest/SKILL.md && grep -q "^origin: ECC" skills/campaign-manifest/SKILL.md && grep -q "Status:" skills/campaign-manifest/SKILL.md && grep -q "Artifacts" skills/campaign-manifest/SKILL.md && grep -q "Grill-Me Decisions" skills/campaign-manifest/SKILL.md && grep -q "Adversary History" skills/campaign-manifest/SKILL.md && grep -q "Agent Outputs" skills/campaign-manifest/SKILL.md && grep -q "Commit Trail" skills/campaign-manifest/SKILL.md && grep -q "Resumption Pointer" skills/campaign-manifest/SKILL.md && echo PASS` | pending@2026-03-22T22:58:00Z
- [ ] PC-003: workflow-init.sh toolchain field | integration test | pending@2026-03-22T22:58:00Z
- [ ] PC-004: toolchain-persist.sh writes commands + jq fallback | integration test | pending@2026-03-22T22:58:00Z
- [ ] PC-005: phase-transition.sh campaign_path support | integration test | pending@2026-03-22T22:58:00Z
- [ ] PC-006: scope-check.sh reads design_path | `grep -q "design_path" .claude/hooks/scope-check.sh && grep -q "solution.md" .claude/hooks/scope-check.sh && echo PASS` | pending@2026-03-22T22:58:00Z
- [ ] PC-007: spec-pipeline-shared 5 campaign sections | `grep -q "Campaign Init" skills/spec-pipeline-shared/SKILL.md && grep -q "Grill-Me Disk Persistence" skills/spec-pipeline-shared/SKILL.md && grep -q "Draft Spec Persistence" skills/spec-pipeline-shared/SKILL.md && grep -q "Adversary History Tracking" skills/spec-pipeline-shared/SKILL.md && grep -q "Agent Output Tracking" skills/spec-pipeline-shared/SKILL.md && echo PASS` | pending@2026-03-22T22:58:00Z
- [ ] PC-008: spec-pipeline-shared Project Detection wording | `grep -q "toolchain-persist.sh" skills/spec-pipeline-shared/SKILL.md && echo PASS` | pending@2026-03-22T22:58:00Z
- [ ] PC-009: Remove "Store mentally" from all spec commands | `! grep -q "Store these commands mentally" commands/spec-dev.md && ! grep -q "Store these commands mentally" commands/spec-fix.md && ! grep -q "Store these commands mentally" commands/spec-refactor.md && echo PASS` | pending@2026-03-22T22:58:00Z
- [ ] PC-010: Spec commands reference shared campaign | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-011: Spec commands reference toolchain-persist | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-012: Spec commands reference draft spec persistence | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-013: design.md disk fallbacks | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-014: design.md campaign updates | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-015: wave-analysis skill extraction | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-016: wave-dispatch skill extraction | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-017: progress-tracking skill extraction | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-018: tasks-generation skill extraction | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-019: implement.md references skills + under 350 lines | grep + wc checks | pending@2026-03-22T22:58:00Z
- [ ] PC-020: implement.md Commit Trail campaign writes | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-021: implement.md Agent Outputs campaign writes | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-022: implement.md campaign re-entry orientation | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-023: strategic-compact campaign awareness | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-024: ADR 0013 | file + grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-025: Glossary entries | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-026: CHANGELOG entry | grep checks | pending@2026-03-22T22:58:00Z
- [ ] PC-027: Full test suite | `bash tests/test-campaign-manifest.sh` | pending@2026-03-22T22:58:00Z
- [ ] PC-028: Pipeline-summaries backward compat | `bash tests/test-pipeline-summaries.sh` | pending@2026-03-22T22:58:00Z
- [ ] PC-029: Wave-parallel backward compat | `bash tests/test-wave-parallel.sh` | pending@2026-03-22T22:58:00Z
- [ ] PC-030: Markdown lint | `npm run lint` | pending@2026-03-22T22:58:00Z
- [ ] PC-031: Rust build | `cargo build` | pending@2026-03-22T22:58:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-22T22:58:00Z
- [ ] Code review | pending@2026-03-22T22:58:00Z
- [ ] Doc updates | pending@2026-03-22T22:58:00Z
- [ ] Write implement-done.md | pending@2026-03-22T22:58:00Z
