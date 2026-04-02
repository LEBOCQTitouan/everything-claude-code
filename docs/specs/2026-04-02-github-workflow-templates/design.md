# Solution: BL-119 GitHub Workflow Templates for Claude Code Integration

## Spec Reference
Concern: dev, Feature: BL-119 GitHub workflow templates for Claude Code integration

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `workflow-templates/claude-pr-review.yml` | Create | PR review template using claude-code-action@v1 with least-privilege perms, concurrency, fork detection, API key check, env var customization | US-001, AC-001.1-001.8 |
| 2 | `workflow-templates/claude-pr-review-fork-safe.yml` | Create | Two-workflow fork-safe variant using workflow_run for OSS repos | US-001, AC-001.5 |
| 3 | `workflow-templates/claude-issue-triage.yml` | Create | Issue triage with default label taxonomy, LABEL_TAXONOMY env var, needs-triage fallback | US-002, AC-002.1-002.5 |
| 4 | `workflow-templates/claude-release-notes.yml` | Create | Release notes on v* tag push with categorization and 500-commit truncation | US-003, AC-003.1-003.5 |
| 5 | `workflow-templates/claude-ci-linter.yml` | Create | Convention linter with CONVENTION_RULES / .claude-lint-rules precedence | US-004, AC-004.1-004.4 |
| 6 | `commands/scaffold-workflows.md` | Create | Slash command: interactive template selection, verbatim copy, overwrite warning, dry-run | US-005, AC-005.1-005.4 |
| 7 | `skills/ci-cd-workflows/SKILL.md` | Create | Merged comprehensive CI/CD skill with 5 required H2 sections | US-006, AC-006.1, AC-006.3 |
| 8 | `skills/github-actions/SKILL.md` | Replace | Deprecated redirect to ci-cd-workflows with valid frontmatter, one release cycle | US-006, AC-006.2 |
| 9 | `rules/ecc/github-actions.md` | Modify | Add cross-reference to new ci-cd-workflows skill | US-006, AC-006.4 |
| 10 | `docs/adr/0037-workflow-templates-content-type.md` | Create | ADR for new content type decision | Decision #1 |
| 11 | `docs/adr/0038-scaffold-command-distribution.md` | Create | ADR for slash command distribution over ecc init | Decision #2 |
| 12 | `docs/adr/README.md` | Modify | Add ADR 0037, 0038 to index | Decision #1, #2 |
| 13 | `docs/domain/bounded-contexts.md` | Modify | Add "Workflow Template" concept to Configuration context | Spec Doc Impact |
| 14 | `CLAUDE.md` | Modify | Add /scaffold-workflows to side commands, mention workflow-templates/ | US-005, US-006 |
| 15 | `.github/workflows/ci.yml` | Modify | Add YAML validation step for workflow-templates | US-007, AC-007.3 |

**Note**: `skills/github-actions-rust/SKILL.md` was verified to contain no reference to the `github-actions` skill -- no change needed (AC-006.4 satisfied by File #9 only).

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | PR review template parses as valid YAML | AC-001.1 | `python3 -c "import yaml; yaml.safe_load(open('workflow-templates/claude-pr-review.yml'))"` | Exit 0 |
| PC-002 | unit | PR review has required top-level keys | AC-001.1, AC-007.1 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-pr-review.yml')); assert all(k in d for k in ['name','on','permissions','jobs'])"` | Exit 0 |
| PC-003 | unit | PR review triggers on pull_request opened+synchronize | AC-001.1 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-pr-review.yml')); pr=d['on']['pull_request']; assert 'opened' in pr.get('types',[]) and 'synchronize' in pr.get('types',[])"` | Exit 0 |
| PC-004 | unit | PR review uses anthropics/claude-code-action@v1 | AC-001.1 | `grep -q 'anthropics/claude-code-action@v1' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-005 | unit | PR review has Read,Grep,Glob allowed tools | AC-001.2 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-pr-review.yml')); s=str(d); assert 'Read' in s and 'Grep' in s and 'Glob' in s"` | Exit 0 |
| PC-006 | unit | PR review permissions: contents read, pull-requests write only | AC-001.3 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-pr-review.yml')); p=d['permissions']; assert p.get('contents')=='read' and p.get('pull-requests')=='write' and len(p)==2"` | Exit 0 |
| PC-007 | unit | PR review does NOT use pull_request_target | AC-001.3 | `! grep -q 'pull_request_target' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-008 | unit | PR review has fork detection step | AC-001.4 | `grep -q 'github.event.pull_request.head.repo.fork\|head.repo.full_name.*!=.*base.repo.full_name' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-009 | unit | PR review has concurrency group with PR number | AC-001.8 | `grep -q 'concurrency' workflow-templates/claude-pr-review.yml && grep -q 'pull_request.number\|event.number' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-010 | unit | PR review has API key check step | AC-001.7 | `grep -q 'ANTHROPIC_API_KEY' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-011 | unit | PR review has CLAUDE_MODEL env var documented | AC-001.6 | `grep -q 'CLAUDE_MODEL' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-012 | unit | ECC version header in PR review template | AC-007.4 | `head -3 workflow-templates/claude-pr-review.yml \| grep -q '# ECC Template v'` | Exit 0 |
| PC-013 | lint | Fork-safe variant parses as valid YAML | AC-001.5 | `python3 -c "import yaml; yaml.safe_load(open('workflow-templates/claude-pr-review-fork-safe.yml'))"` | Exit 0 |
| PC-014 | unit | Fork-safe variant uses workflow_run trigger | AC-001.5 | `grep -q 'workflow_run' workflow-templates/claude-pr-review-fork-safe.yml` | Exit 0 |
| PC-015 | unit | Fork-safe variant does NOT use pull_request_target | AC-001.5 | `! grep -q 'pull_request_target' workflow-templates/claude-pr-review-fork-safe.yml` | Exit 0 |
| PC-016 | lint | Issue triage parses as valid YAML | AC-002.1 | `python3 -c "import yaml; yaml.safe_load(open('workflow-templates/claude-issue-triage.yml'))"` | Exit 0 |
| PC-017 | unit | Issue triage triggers on issues opened | AC-002.1 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-issue-triage.yml')); assert 'opened' in d['on']['issues']['types']"` | Exit 0 |
| PC-018 | unit | Issue triage has default label taxonomy | AC-002.2 | `grep -q 'bug' workflow-templates/claude-issue-triage.yml && grep -q 'feature' workflow-templates/claude-issue-triage.yml && grep -q 'question' workflow-templates/claude-issue-triage.yml && grep -q 'security' workflow-templates/claude-issue-triage.yml` | Exit 0 |
| PC-019 | unit | Issue triage permissions: issues write, contents read | AC-002.3 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-issue-triage.yml')); p=d['permissions']; assert p.get('issues')=='write' and p.get('contents')=='read' and len(p)==2"` | Exit 0 |
| PC-020 | unit | Issue triage has LABEL_TAXONOMY env var | AC-002.4 | `grep -q 'LABEL_TAXONOMY' workflow-templates/claude-issue-triage.yml` | Exit 0 |
| PC-021 | unit | Issue triage has needs-triage fallback | AC-002.5 | `grep -q 'needs-triage' workflow-templates/claude-issue-triage.yml` | Exit 0 |
| PC-022 | unit | ECC version header in issue triage | AC-007.4 | `head -3 workflow-templates/claude-issue-triage.yml \| grep -q '# ECC Template v'` | Exit 0 |
| PC-023 | lint | Release notes parses as valid YAML | AC-003.1 | `python3 -c "import yaml; yaml.safe_load(open('workflow-templates/claude-release-notes.yml'))"` | Exit 0 |
| PC-024 | unit | Release notes triggers on v* tag push | AC-003.1 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-release-notes.yml')); tags=d['on']['push']['tags']; assert any('v' in t for t in tags)"` | Exit 0 |
| PC-025 | unit | Release notes permissions: contents write only | AC-003.4 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-release-notes.yml')); p=d['permissions']; assert p.get('contents')=='write' and len(p)==1"` | Exit 0 |
| PC-026 | unit | Release notes has 500-commit truncation | AC-003.5 | `grep -qi 'truncat.*500\|limit.*500\|max.*500' workflow-templates/claude-release-notes.yml` | Exit 0 |
| PC-027 | unit | Release notes has categorization prompt | AC-003.2 | `grep -qi 'features\|fixes\|breaking' workflow-templates/claude-release-notes.yml` | Exit 0 |
| PC-028 | unit | ECC version header in release notes | AC-007.4 | `head -3 workflow-templates/claude-release-notes.yml \| grep -q '# ECC Template v'` | Exit 0 |
| PC-029 | lint | CI linter parses as valid YAML | AC-004.1 | `python3 -c "import yaml; yaml.safe_load(open('workflow-templates/claude-ci-linter.yml'))"` | Exit 0 |
| PC-030 | unit | CI linter triggers on pull_request | AC-004.1 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-ci-linter.yml')); assert 'pull_request' in d['on']"` | Exit 0 |
| PC-031 | unit | CI linter permissions: contents read, pull-requests write | AC-004.4 | `python3 -c "import yaml; d=yaml.safe_load(open('workflow-templates/claude-ci-linter.yml')); p=d['permissions']; assert p.get('contents')=='read' and p.get('pull-requests')=='write' and len(p)==2"` | Exit 0 |
| PC-032 | unit | CI linter has CONVENTION_RULES env var | AC-004.3 | `grep -q 'CONVENTION_RULES' workflow-templates/claude-ci-linter.yml` | Exit 0 |
| PC-033 | unit | CI linter has .claude-lint-rules precedence | AC-004.3 | `grep -q 'claude-lint-rules' workflow-templates/claude-ci-linter.yml` | Exit 0 |
| PC-034 | unit | ECC version header in CI linter | AC-007.4 | `head -3 workflow-templates/claude-ci-linter.yml \| grep -q '# ECC Template v'` | Exit 0 |
| PC-035 | unit | All templates use pinned action versions (no @latest) | AC-007.1 | `! grep -rn '@latest' workflow-templates/` | Exit 0 |
| PC-036 | unit | All templates have permissions block | AC-007.1 | `for f in workflow-templates/*.yml; do python3 -c "import yaml; d=yaml.safe_load(open('$f')); assert 'permissions' in d" \|\| exit 1; done` | Exit 0 |
| PC-037 | lint | Scaffold-workflows has valid frontmatter | AC-005.1 | `python3 -c "content=open('commands/scaffold-workflows.md').read(); assert 'description' in content.split('---')[1]"` | Exit 0 |
| PC-038 | unit | Scaffold-workflows has allowed-tools | AC-005.1 | `python3 -c "content=open('commands/scaffold-workflows.md').read(); assert 'allowed-tools' in content.split('---')[1]"` | Exit 0 |
| PC-039 | unit | Scaffold-workflows references AskUserQuestion | AC-005.1 | `grep -q 'AskUserQuestion' commands/scaffold-workflows.md` | Exit 0 |
| PC-040 | unit | Scaffold-workflows has dry-run support | AC-005.4 | `grep -q 'dry-run\|dry_run\|--dry-run' commands/scaffold-workflows.md` | Exit 0 |
| PC-041 | unit | Scaffold-workflows has overwrite warning | AC-005.3 | `grep -qi 'overwrite\|already exist' commands/scaffold-workflows.md` | Exit 0 |
| PC-042 | unit | Scaffold-workflows references .github/workflows/ | AC-005.2 | `grep -q '.github/workflows' commands/scaffold-workflows.md` | Exit 0 |
| PC-043 | integration | ecc validate commands passes | AC-005.1 | `cargo build --release 2>/dev/null && ./target/release/ecc validate commands` | Exit 0 |
| PC-044 | lint | New skill has valid frontmatter | AC-006.3 | `python3 -c "import yaml; content=open('skills/ci-cd-workflows/SKILL.md').read(); fm=content.split('---')[1]; d=yaml.safe_load(fm); assert d['name']=='ci-cd-workflows' and 'description' in d and d['origin']=='ECC'"` | Exit 0 |
| PC-045 | unit | New skill has 5 required H2 sections | AC-006.1 | `for s in 'General CI/CD Patterns' 'Claude Code Templates' 'Security' 'Fork Safety' 'Customization'; do grep -q "## $s" skills/ci-cd-workflows/SKILL.md \|\| { echo "Missing: $s"; exit 1; }; done` | Exit 0 |
| PC-046 | unit | Old skill is deprecated redirect with valid frontmatter | AC-006.2 | `grep -q 'ci-cd-workflows' skills/github-actions/SKILL.md && grep -qi 'moved\|deprecated' skills/github-actions/SKILL.md && python3 -c "import yaml; content=open('skills/github-actions/SKILL.md').read(); fm=content.split('---')[1]; d=yaml.safe_load(fm); assert 'name' in d and 'description' in d"` | Exit 0 |
| PC-047 | integration | ecc validate skills passes | AC-006.3 | `cargo build --release 2>/dev/null && ./target/release/ecc validate skills` | Exit 0 |
| PC-048 | unit | rules/ecc/github-actions.md references ci-cd-workflows | AC-006.4 | `grep -q 'ci-cd-workflows' rules/ecc/github-actions.md` | Exit 0 |
| PC-049 | unit | ADR 0037 exists with correct structure | Decision #1 | `test -f docs/adr/0037-workflow-templates-content-type.md && grep -q '## Status' docs/adr/0037-workflow-templates-content-type.md && grep -q '## Decision' docs/adr/0037-workflow-templates-content-type.md` | Exit 0 |
| PC-050 | unit | ADR 0038 exists with correct structure | Decision #2 | `test -f docs/adr/0038-scaffold-command-distribution.md && grep -q '## Status' docs/adr/0038-scaffold-command-distribution.md && grep -q '## Decision' docs/adr/0038-scaffold-command-distribution.md` | Exit 0 |
| PC-051 | unit | ADR README includes 0037 and 0038 | Decision #1, #2 | `grep -q '0037' docs/adr/README.md && grep -q '0038' docs/adr/README.md` | Exit 0 |
| PC-052 | unit | Bounded contexts has workflow template concept | Spec Doc Impact | `grep -qi 'workflow.template' docs/domain/bounded-contexts.md` | Exit 0 |
| PC-053 | unit | CLAUDE.md mentions /scaffold-workflows | Spec Doc Impact | `grep -q 'scaffold-workflows' CLAUDE.md` | Exit 0 |
| PC-054 | unit | CLAUDE.md mentions workflow-templates directory | Spec Doc Impact | `grep -q 'workflow-templates' CLAUDE.md` | Exit 0 |
| PC-055 | unit | CI workflow validates workflow-templates | AC-007.3 | `grep -q 'workflow-templates\|workflow.templates' .github/workflows/ci.yml` | Exit 0 |
| PC-056 | build | Cargo build succeeds | All | `cargo build --release` | Exit 0 |
| PC-057 | lint | Cargo clippy passes | All | `cargo clippy -- -D warnings` | Exit 0 |
| PC-058 | integration | Full cargo test passes | All | `cargo test` | Exit 0 |
| PC-059 | unit | No write-all permissions in any template | AC-001.3, AC-007.1 | `! grep -rn 'write-all' workflow-templates/` | Exit 0 |
| PC-060 | unit | secrets.ANTHROPIC_API_KEY in PR review | AC-001.3 | `grep -q 'secrets.ANTHROPIC_API_KEY' workflow-templates/claude-pr-review.yml` | Exit 0 |
| PC-061 | unit | Release notes references release creation | AC-003.3 | `grep -qi 'release.*create\|release.*body\|gh release' workflow-templates/claude-release-notes.yml` | Exit 0 |
| PC-062 | unit | CI linter instructs inline comment posting | AC-004.2 | `grep -qi 'inline\|comment\|annotation\|review' workflow-templates/claude-ci-linter.yml` | Exit 0 |

### Coverage Check

All 34 ACs covered (AC-007.2 is local-only by spec design):

| AC | Covering PCs |
|---|---|
| AC-001.1 | PC-001, PC-002, PC-003, PC-004 |
| AC-001.2 | PC-005 |
| AC-001.3 | PC-006, PC-007, PC-059, PC-060 |
| AC-001.4 | PC-008 |
| AC-001.5 | PC-013, PC-014, PC-015 |
| AC-001.6 | PC-011 |
| AC-001.7 | PC-010 |
| AC-001.8 | PC-009 |
| AC-002.1 | PC-016, PC-017 |
| AC-002.2 | PC-018 |
| AC-002.3 | PC-019 |
| AC-002.4 | PC-020 |
| AC-002.5 | PC-021 |
| AC-003.1 | PC-023, PC-024 |
| AC-003.2 | PC-027 |
| AC-003.3 | PC-061 |
| AC-003.4 | PC-025 |
| AC-003.5 | PC-026 |
| AC-004.1 | PC-029, PC-030 |
| AC-004.2 | PC-062 |
| AC-004.3 | PC-032, PC-033 |
| AC-004.4 | PC-031 |
| AC-005.1 | PC-037, PC-038, PC-039, PC-043 |
| AC-005.2 | PC-042 |
| AC-005.3 | PC-041 |
| AC-005.4 | PC-040 |
| AC-006.1 | PC-045 |
| AC-006.2 | PC-046 |
| AC-006.3 | PC-044, PC-047 |
| AC-006.4 | PC-048 |
| AC-007.1 | PC-002, PC-035, PC-036 |
| AC-007.2 | Local-only gate (not CI) |
| AC-007.3 | PC-055 |
| AC-007.4 | PC-012, PC-022, PC-028, PC-034 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Content install pipeline | release tarball | N/A | Verify workflow-templates/ in tarball | ignored | Release packaging modified |
| 2 | /scaffold-workflows | Claude Code | N/A | Verify command writes to .github/workflows/ | ignored | Command .md modified |
| 3 | CI pipeline | ci.yml | N/A | Verify YAML validation step runs | ignored | ci.yml modified |

### E2E Activation Rules
No E2E tests to un-ignore -- content-only changes, no Rust adapter modifications.

## Test Strategy

TDD order (dependency-first):
1. **PC-001 to PC-036, PC-059, PC-060, PC-061, PC-062** -- Phase 1: Workflow templates (foundation)
2. **PC-037 to PC-043** -- Phase 2: Slash command (depends on templates)
3. **PC-044 to PC-048** -- Phase 3: Skill merge (documents templates)
4. **PC-049 to PC-054** -- Phase 4: Documentation (documents decisions)
5. **PC-055 to PC-058** -- Phase 5: CI + final gates (validates everything)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0037-workflow-templates-content-type.md` | MEDIUM | Create | ADR: workflow-templates/ as new content type | Decision #1 |
| 2 | `docs/adr/0038-scaffold-command-distribution.md` | MEDIUM | Create | ADR: /scaffold-workflows over ecc init | Decision #2 |
| 3 | `docs/adr/README.md` | LOW | Modify | Add 0037, 0038 entries | Decision #1, #2 |
| 4 | `docs/domain/bounded-contexts.md` | LOW | Modify | Add "Workflow Template" concept | US-006 |
| 5 | `CLAUDE.md` | HIGH | Modify | Add /scaffold-workflows, workflow-templates/ | US-005, US-006 |
| 6 | `CHANGELOG.md` | LOW | Modify | BL-119 feature entry | All |

## SOLID Assessment

**CLEAN** (uncle-bob) -- 1 MEDIUM (file table cleanup applied), 2 LOW (grep fragility acknowledged, layer labels noted). SRP, OCP, clean dependency ordering all satisfied.

## Robert's Oath Check

**CLEAN** -- All 5 oath dimensions satisfied. 62 pass conditions. 5-phase incremental delivery. Responsible deprecated redirect migration.

## Security Notes

**CLEAR** -- 2 LOW observations. Least-privilege permissions with exact-count enforcement. No pull_request_target. Read-only tool restriction. API key via secrets only. Major-version pinning consistent with project conventions.

## Rollback Plan

Reverse dependency order:
1. Revert `.github/workflows/ci.yml` changes
2. Revert `CLAUDE.md` changes
3. Revert `docs/domain/bounded-contexts.md` changes
4. Revert `docs/adr/README.md`, delete `docs/adr/0038-scaffold-command-distribution.md`
5. Delete `docs/adr/0037-workflow-templates-content-type.md`
6. Revert `rules/ecc/github-actions.md` changes
7. Restore `skills/github-actions/SKILL.md` from git (original content, not redirect)
8. Delete `skills/ci-cd-workflows/` directory
9. Delete `commands/scaffold-workflows.md`
10. Delete `workflow-templates/` directory

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 1 MEDIUM, 2 LOW |
| Robert | CLEAN | 0 |
| Security | CLEAR | 2 LOW |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Coverage | 88 | 88 | PASS | All 34 ACs covered, PC-061/062 added for gaps |
| Order | 92 | 92 | PASS | Clean 5-phase dependency chain |
| Fragility | 72 | 85 | PASS | PC-005 fixed to YAML parsing, PC-026 tightened |
| Rollback | 90 | 90 | PASS | Correct reverse dependency order |
| Architecture | 95 | 95 | PASS | Content-only, respects hexagonal architecture |
| Blast Radius | 85 | 85 | PASS | 15 files, no unnecessary changes |
| Missing PCs | 78 | 87 | PASS | PC-046 now validates redirect frontmatter |
| Doc Plan | 88 | 88 | PASS | Complete with ADRs, CHANGELOG |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `workflow-templates/claude-pr-review.yml` | Create | US-001 |
| 2 | `workflow-templates/claude-pr-review-fork-safe.yml` | Create | US-001, AC-001.5 |
| 3 | `workflow-templates/claude-issue-triage.yml` | Create | US-002 |
| 4 | `workflow-templates/claude-release-notes.yml` | Create | US-003 |
| 5 | `workflow-templates/claude-ci-linter.yml` | Create | US-004 |
| 6 | `commands/scaffold-workflows.md` | Create | US-005 |
| 7 | `skills/ci-cd-workflows/SKILL.md` | Create | US-006 |
| 8 | `skills/github-actions/SKILL.md` | Replace | US-006, AC-006.2 |
| 9 | `rules/ecc/github-actions.md` | Modify | US-006, AC-006.4 |
| 10 | `docs/adr/0037-workflow-templates-content-type.md` | Create | Decision #1 |
| 11 | `docs/adr/0038-scaffold-command-distribution.md` | Create | Decision #2 |
| 12 | `docs/adr/README.md` | Modify | Decision #1, #2 |
| 13 | `docs/domain/bounded-contexts.md` | Modify | Spec Doc Impact |
| 14 | `CLAUDE.md` | Modify | US-005, US-006 |
| 15 | `.github/workflows/ci.yml` | Modify | US-007, AC-007.3 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-github-workflow-templates/design.md` | Full design + Phase Summary |
