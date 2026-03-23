# Solution: Graceful Mid-Session Exit + Implement Context Clear (BL-055, BL-054)

## Spec Reference
Concern: dev, Feature: Graceful mid-session exit when context gets heavy (BL-055)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | statusline/context-persist.sh | Create | SRP-compliant side-channel writer: reads context % from stdin JSON, sanitizes session ID, writes to user-private temp file via atomic mktemp+mv | US-001 (AC-001.1, AC-001.4-001.7) |
| 2 | skills/graceful-exit/read-context-percentage.sh | Create | Pure-function reader: reads temp file, validates numeric 0-100, returns percentage or "unknown". Sanitizes session ID to prevent path traversal | US-001 (AC-001.2-001.3, AC-001.8), US-002 (AC-002.2, AC-002.9) |
| 3 | skills/graceful-exit/SKILL.md | Create | Shared protocol: thresholds (75/85/95), state-dump contract with per-command Resumption Pointer formats, exit message template, re-entry guidance | US-002 (AC-002.1-002.8) |
| 4 | commands/implement.md | Modify | Phase 0: context clear gate with AskUserQuestion; Phases 3-7: context checkpoints between waves and at phase transitions | US-003 (AC-003.1-003.5), US-004 (AC-004.1-004.8) |
| 5 | commands/audit-full.md | Modify | Add re-entry logic: read partial results dir from campaign.md, pass completed domains to orchestrator, skip them. Cleanup partial dirs on success | US-005 (AC-005.3, AC-005.6, AC-005.8) |
| 6 | agents/audit-orchestrator.md | Modify | Context checkpoint after each domain agent return. Partial results to docs/audits/partial-<timestamp>/. Campaign.md update. No-campaign fallback to audit-resume.md | US-005 (AC-005.1-005.2, AC-005.4-005.5, AC-005.7, AC-005.9-005.10) |
| 7 | skills/strategic-compact/SKILL.md | Modify | Add row to Compaction Decision Guide: graceful-exit backstop at 85% | US-006 (AC-006.3) |
| 8 | skills/campaign-manifest/SKILL.md | Modify | Add checkpoint-triggered Resumption Pointer updates to Incremental Updates section | US-006 (AC-006.4) |
| 9 | docs/domain/glossary.md | Modify | Add "Graceful Exit" and "Context Checkpoint" entries | US-006 (AC-006.1) |
| 10 | docs/adr/0014-context-aware-graceful-exit.md | Create | ADR: two-threshold design, statusline side-channel, session ID sanitization, scope to /implement + /audit-full, audit re-entry pattern | US-006 (AC-006.2) |
| 11 | docs/adr/README.md | Modify | Add ADR 0014 to index | US-006 (AC-006.2) |
| 12 | CHANGELOG.md | Modify | Add BL-055 + BL-054 entry | US-006 (AC-006.5) |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | structural | context-persist.sh exists with atomic write + session ID sanitization | AC-001.1, AC-001.6, AC-001.7 | `test -f statusline/context-persist.sh && grep -c 'mktemp' statusline/context-persist.sh && grep -c 'CLAUDE_SESSION_ID' statusline/context-persist.sh && grep -c '[^a-zA-Z0-9_-]' statusline/context-persist.sh` | File exists, all greps >= 1 |
| PC-002 | structural | context-persist.sh uses PPID fallback | AC-001.6 | `grep 'PPID' statusline/context-persist.sh` | Match found |
| PC-003 | structural | context-persist.sh uses user-private runtime dir | AC-001.4, AC-001.5 | `grep 'ECC_RUNTIME_DIR' statusline/context-persist.sh` | Match found |
| PC-004 | integration | read-context-percentage.sh returns "unknown" for missing file | AC-001.3, AC-002.2, AC-002.7 | `chmod +x skills/graceful-exit/read-context-percentage.sh && DIR=$(mktemp -d) && ECC_RUNTIME_DIR="$DIR/ecc-test" CLAUDE_SESSION_ID=test-missing skills/graceful-exit/read-context-percentage.sh` | Output: "unknown" |
| PC-005 | integration | read-context-percentage.sh returns valid percentage | AC-001.2, AC-002.2 | `DIR=$(mktemp -d) && mkdir -p "$DIR/ecc-test" && echo "42" > "$DIR/ecc-test/ecc-context-test-valid.pct" && ECC_RUNTIME_DIR="$DIR/ecc-test" CLAUDE_SESSION_ID=test-valid skills/graceful-exit/read-context-percentage.sh` | Output: "42" |
| PC-006 | integration | read-context-percentage.sh returns "unknown" for garbage | AC-001.8 | `DIR=$(mktemp -d) && mkdir -p "$DIR/ecc-test" && echo "garbage" > "$DIR/ecc-test/ecc-context-test-garbage.pct" && ECC_RUNTIME_DIR="$DIR/ecc-test" CLAUDE_SESSION_ID=test-garbage skills/graceful-exit/read-context-percentage.sh` | Output: "unknown" |
| PC-007 | integration | read-context-percentage.sh returns "unknown" for out-of-range | AC-001.8 | `DIR=$(mktemp -d) && mkdir -p "$DIR/ecc-test" && echo "150" > "$DIR/ecc-test/ecc-context-test-range.pct" && ECC_RUNTIME_DIR="$DIR/ecc-test" CLAUDE_SESSION_ID=test-range skills/graceful-exit/read-context-percentage.sh` | Output: "unknown" |
| PC-008 | integration | read-context-percentage.sh sanitizes session ID (path traversal) | AC-001.6 | `DIR=$(mktemp -d) && mkdir -p "$DIR/ecc-test" && CLAUDE_SESSION_ID='../../etc/evil' ECC_RUNTIME_DIR="$DIR/ecc-test" skills/graceful-exit/read-context-percentage.sh` | Output: "unknown" |
| PC-009 | structural | graceful-exit SKILL.md has required frontmatter and sections | AC-002.1 | `test -f skills/graceful-exit/SKILL.md && grep -c 'name: graceful-exit' skills/graceful-exit/SKILL.md && grep -c '## Thresholds' skills/graceful-exit/SKILL.md && grep -c '## State-Dump Contract' skills/graceful-exit/SKILL.md && grep -c '## Exit Message' skills/graceful-exit/SKILL.md` | All >= 1 |
| PC-010 | structural | graceful-exit skill defines warn behavior | AC-002.3 | `grep -A3 '75%' skills/graceful-exit/SKILL.md \| grep -ci 'warn\|continue\|consider'` | >= 1 |
| PC-011 | structural | graceful-exit skill defines exit behavior | AC-002.4 | `grep -A3 '85%' skills/graceful-exit/SKILL.md \| grep -ci 'stop\|save\|exit'` | >= 1 |
| PC-012 | structural | graceful-exit skill defines hard ceiling | AC-002.8 | `grep -A3 '95%' skills/graceful-exit/SKILL.md \| grep -ci 'hard ceiling\|immediate'` | >= 1 |
| PC-013 | structural | graceful-exit skill defines /implement Resumption Pointer format | AC-002.5 | `grep -c 'Wave.*complete' skills/graceful-exit/SKILL.md` | >= 1 |
| PC-014 | structural | graceful-exit skill defines exit message with context %, campaign path, re-run | AC-002.6 | `grep -c 'campaign' skills/graceful-exit/SKILL.md && grep -c 're-run\|re-entry' skills/graceful-exit/SKILL.md` | Both >= 1 |
| PC-015 | structural | implement.md reads context percentage in Phase 0 | AC-003.1 | `grep -c 'read-context-percentage\|context.*percentage' commands/implement.md` | >= 1 |
| PC-016 | structural | implement.md has AskUserQuestion with clear/continue options | AC-003.2, AC-003.4 | `grep -c 'AskUserQuestion' commands/implement.md && grep -c 'Clear and restart' commands/implement.md` | Both >= 1 |
| PC-017 | structural | implement.md skips gate on unknown | AC-003.5 | `grep -ci 'unknown.*skip\|skip.*unknown\|unknown.*silently' commands/implement.md` | >= 1 |
| PC-018 | structural | implement.md has context checkpoints between waves | AC-004.1 | `grep -ci 'context checkpoint\|graceful-exit.*wave\|wave.*checkpoint' commands/implement.md` | >= 1 |
| PC-019 | structural | implement.md has context checkpoints at phase transitions | AC-004.2 | `grep -c 'graceful-exit' commands/implement.md` | >= 2 |
| PC-020 | structural | implement.md 85% exit saves tasks.md + campaign.md | AC-004.3 | `grep -A5 '85%' commands/implement.md \| grep -ci 'tasks.md\|campaign\|resumption'` | >= 1 |
| PC-021 | structural | implement.md references graceful-exit skill | AC-004.1, AC-004.7 | `grep -c 'skills/graceful-exit' commands/implement.md` | >= 1 |
| PC-022 | structural | audit-full.md has re-entry reading partial results | AC-005.3, AC-005.6 | `grep -c 'partial-' commands/audit-full.md && grep -c 're-entry\|resume' commands/audit-full.md` | Both >= 1 |
| PC-023 | structural | audit-full.md cleans up partial dirs on success | AC-005.8 | `grep -ci 'clean.*partial\|remove.*partial' commands/audit-full.md` | >= 1 |
| PC-024 | structural | audit-orchestrator has context checkpoint | AC-005.1, AC-005.7 | `grep -c 'context.*check\|read-context-percentage\|graceful-exit' agents/audit-orchestrator.md` | >= 1 |
| PC-025 | structural | audit-orchestrator writes partial results | AC-005.2, AC-005.9 | `grep -c 'partial-' agents/audit-orchestrator.md` | >= 1 |
| PC-026 | structural | audit-orchestrator has no-campaign fallback | AC-005.10 | `grep -c 'audit-resume.md' agents/audit-orchestrator.md` | >= 1 |
| PC-027 | structural | audit-orchestrator skips completed domains | AC-005.3 | `grep -ci 'skip.*completed\|completed.*skip\|incomplete.*domain' agents/audit-orchestrator.md` | >= 1 |
| PC-028 | structural | audit-orchestrator handles all-domains-done | AC-005.4 | `grep -ci 'correlation.*only\|all domains.*complete' agents/audit-orchestrator.md` | >= 1 |
| PC-029 | structural | strategic-compact references graceful-exit | AC-006.3 | `grep -c 'graceful-exit\|85%.*exit\|exit.*85%' skills/strategic-compact/SKILL.md` | >= 1 |
| PC-030 | structural | campaign-manifest documents checkpoint updates | AC-006.4 | `grep -ci 'graceful.exit\|context checkpoint' skills/campaign-manifest/SKILL.md` | >= 1 |
| PC-031 | structural | glossary has "Graceful Exit" | AC-006.1 | `grep -c 'Graceful Exit' docs/domain/glossary.md` | >= 1 |
| PC-032 | structural | glossary has "Context Checkpoint" | AC-006.1 | `grep -c 'Context Checkpoint' docs/domain/glossary.md` | >= 1 |
| PC-033 | structural | ADR 0014 exists with structure | AC-006.2 | `test -f docs/adr/0014-context-aware-graceful-exit.md && grep -c '## Status' docs/adr/0014-context-aware-graceful-exit.md && grep -c '## Decision' docs/adr/0014-context-aware-graceful-exit.md` | All pass |
| PC-034 | structural | ADR 0014 has threshold + side-channel content | AC-006.2 | `grep -c '75%' docs/adr/0014-context-aware-graceful-exit.md && grep -c '85%' docs/adr/0014-context-aware-graceful-exit.md && grep -c 'side-channel\|statusline' docs/adr/0014-context-aware-graceful-exit.md` | All >= 1 |
| PC-035 | structural | ADR 0014 in README index | AC-006.2 | `grep -c '0014' docs/adr/README.md` | >= 1 |
| PC-036 | structural | CHANGELOG has BL-055 | AC-006.5 | `grep -c 'BL-055' CHANGELOG.md` | >= 1 |
| PC-037 | lint | Markdown lint passes | All | `npm run lint` | exit 0 |
| PC-038 | build | Rust build passes | All | `cargo build` | exit 0 |
| PC-039 | build | Rust tests pass | All | `cargo test` | exit 0 |
| PC-040 | lint | Clippy clean | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-041 | structural | implement.md "Clear and restart" path instructs /compact | AC-003.3 | `grep -c '/compact' commands/implement.md` | >= 1 |
| PC-042 | structural | implement.md 75% warning displays and continues | AC-004.4 | `grep -ci '75%.*warn\|warn.*75%\|consider.*compact' commands/implement.md` | >= 1 |
| PC-043 | structural | implement.md below threshold continues silently | AC-004.5 | `grep -ci 'silently\|no action\|proceed.*normal' commands/implement.md` | >= 1 |
| PC-044 | structural | implement.md re-entry uses Resumption Pointer | AC-004.6 | `grep -ci 'resumption pointer\|campaign.*re-entry\|resume.*campaign' commands/implement.md` | >= 1 |
| PC-045 | structural | implement.md 85% threshold independent of 75% warning | AC-004.8 | `grep -ci 'independent\|regardless.*warning\|without.*prior.*warn' commands/implement.md` | >= 1 |
| PC-046 | structural | audit-orchestrator 75% warning | AC-005.5 | `grep -ci '75%.*warn\|warn.*75%' agents/audit-orchestrator.md` | >= 1 |
| PC-047 | integration | Round-trip: writer to temp file to reader | AC-001.1, AC-001.2 | `DIR=$(mktemp -d) && mkdir -p "$DIR/ecc-test" && chmod 700 "$DIR/ecc-test" && echo '{"context_window":{"used_percentage":67}}' \| ECC_RUNTIME_DIR="$DIR/ecc-test" CLAUDE_SESSION_ID=roundtrip bash statusline/context-persist.sh && ECC_RUNTIME_DIR="$DIR/ecc-test" CLAUDE_SESSION_ID=roundtrip skills/graceful-exit/read-context-percentage.sh` | Output: "67" |
| PC-048 | structural | Runtime dir env var consistent between writer and reader | AC-001.4 | `grep -o 'ECC_RUNTIME_DIR' statusline/context-persist.sh skills/graceful-exit/read-context-percentage.sh \| wc -l` | >= 2 (appears in both files) |

### Coverage Check
All 45 ACs covered by 48 PCs. Zero uncovered ACs.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Statusline side-channel | context-persist.sh | N/A | Verify context % written to temp file | ignored | Statusline persistence modified |
| 2 | Implement graceful exit | commands/implement.md | N/A | Verify mid-implementation exit saves state | ignored | Implement checkpoint logic modified |
| 3 | Audit-full resume | audit-orchestrator.md | N/A | Verify partial results persist and re-entry works | ignored | Audit checkpoint logic modified |

### E2E Activation Rules
No E2E tests to un-ignore — integration tests on read-context-percentage.sh (PC-004-008) and round-trip test (PC-047) cover executable code. Command changes are structural (Markdown).

## Test Strategy

TDD order (7 groups, dependency-ordered):

1. **Group 1: Infrastructure** (PC-001-008, PC-047-048) — context-persist.sh + read-context-percentage.sh + security + round-trip
2. **Group 2: Skill** (PC-009-014) — graceful-exit SKILL.md
3. **Group 3: Implement gate** (PC-015-017, PC-041) — /implement Phase 0 context clear gate
4. **Group 4: Implement checkpoints** (PC-018-021, PC-042-045) — /implement wave/phase checkpoints
5. **Group 5: Audit** (PC-022-028, PC-046) — audit-full + orchestrator
6. **Group 6: Docs** (PC-029-036) — glossary, ADR, skills, changelog
7. **Group 7: Verification** (PC-037-040) — lint, build, tests, clippy

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | docs/domain/glossary.md | Domain | Add entries | "Graceful Exit", "Context Checkpoint" | AC-006.1 |
| 2 | docs/adr/0014-context-aware-graceful-exit.md | ADR | Create | Two-threshold, side-channel, session ID sanitization, scope, audit re-entry | AC-006.2 |
| 3 | docs/adr/README.md | Index | Add row | ADR 0014 | AC-006.2 |
| 4 | skills/strategic-compact/SKILL.md | Skill | Update | Graceful-exit backstop row in Compaction Decision Guide | AC-006.3 |
| 5 | skills/campaign-manifest/SKILL.md | Skill | Update | Checkpoint Resumption Pointer updates | AC-006.4 |
| 6 | CHANGELOG.md | Project | Add entry | BL-055 + BL-054 | AC-006.5 |

## SOLID Assessment
uncle-bob: NEEDS WORK → addressed. SRP fix (separate context-persist.sh from statusline), rename to read-context-percentage.sh. DIP correct (commands depend on skill protocol). OCP correct (additive via skill extension point).

## Robert's Oath Check
robert: CLEAN with 1 warning (implement.md size — additions should be concise, reference skill for protocol details).

## Security Notes
| Finding | Severity | Mitigation |
|---------|----------|------------|
| Path traversal via SESSION_ID | CRITICAL | Sanitize: strip non-alphanumeric except dash/underscore |
| Temp file in world-readable /tmp | HIGH | User-private dir: ECC_RUNTIME_DIR with chmod 700 |
| Partial audit dir timestamp source | MEDIUM | Generated internally by script |
| Numeric validation strictness | MEDIUM | Strict regex + range check 0-100 |

## Rollback Plan
Reverse dependency order:
12. CHANGELOG.md — remove entry
11. docs/adr/README.md — remove row
10. docs/adr/0014-context-aware-graceful-exit.md — delete
9. docs/domain/glossary.md — remove entries
8. skills/campaign-manifest/SKILL.md — revert
7. skills/strategic-compact/SKILL.md — revert
6. agents/audit-orchestrator.md — revert
5. commands/audit-full.md — revert
4. commands/implement.md — revert
3. skills/graceful-exit/SKILL.md — delete
2. skills/graceful-exit/read-context-percentage.sh — delete
1. statusline/context-persist.sh — delete

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | NEEDS WORK → addressed (SRP fix, rename) | 2 prescriptions |
| Robert | CLEAN | 1 warning (implement.md size) |
| Security | 1 CRITICAL + 1 HIGH → mitigated | 4 findings |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS (R2 fix) | 8 PCs added for uncovered ACs |
| Order | PASS | 7 TDD groups in dependency order |
| Fragility | PASS (R2 fix) | Env var naming consistency PC added |
| Rollback | PASS | 12-step reverse dependency order |
| Architecture | PASS | Content-layer only, no Rust changes |
| Blast radius | PASS | 12 files, scoped to 2 commands |
| Missing PCs | PASS (R2 fix) | Round-trip integration test added |
| Doc plan | PASS | 6 doc actions including ADR 0014 |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | statusline/context-persist.sh | Create | US-001 |
| 2 | skills/graceful-exit/read-context-percentage.sh | Create | US-001, US-002 |
| 3 | skills/graceful-exit/SKILL.md | Create | US-002 |
| 4 | commands/implement.md | Modify | US-003, US-004 |
| 5 | commands/audit-full.md | Modify | US-005 |
| 6 | agents/audit-orchestrator.md | Modify | US-005 |
| 7 | skills/strategic-compact/SKILL.md | Modify | US-006 |
| 8 | skills/campaign-manifest/SKILL.md | Modify | US-006 |
| 9 | docs/domain/glossary.md | Modify | US-006 |
| 10 | docs/adr/0014-context-aware-graceful-exit.md | Create | US-006 |
| 11 | docs/adr/README.md | Modify | US-006 |
| 12 | CHANGELOG.md | Modify | US-006 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-23-graceful-mid-session-exit/design.md | Full design |
