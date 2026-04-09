# Solution: Add cargo-llvm-cov Coverage Gate to CI

## Spec Reference
Concern: dev, Feature: Add cargo-llvm-cov coverage gate to CI

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `CLAUDE.md` | modify | Append "coverage gate" to Glossary bullet in Gotchas | AC-004.1, US-004 |
| 2 | `.github/workflows/ci.yml` | modify | Add `coverage` job with path filter, cache isolation, LCOV upload | AC-001.*, AC-002.*, AC-003.1, AC-005.1a |
| 3 | `CHANGELOG.md` | modify | Add entry for new CI coverage gate | US-001 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | structural | YAML parses without error | baseline | `python3 -c "import yaml,pathlib; yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); print('OK')"` | `OK` |
| PC-002 | structural | Job name is `Coverage Gate` | AC-005.1a | `grep 'name: Coverage Gate' .github/workflows/ci.yml` | match found |
| PC-003 | structural | Full llvm-cov command with --fail-under-functions 80 | AC-001.1 | `grep 'fail-under-functions 80' .github/workflows/ci.yml` | match found |
| PC-004 | structural | Command includes --exclude xtask, --exclude ecc-test-support, --workspace, --lcov | AC-001.1 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['coverage']['steps']; cmds=' '.join(s.get('run','') for s in steps); assert '--exclude xtask' in cmds and '--exclude ecc-test-support' in cmds and '--workspace' in cmds and '--lcov' in cmds; print('OK')"` | `OK` |
| PC-005 | structural | rust-toolchain step before install-action step | AC-001.0 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['coverage']['steps']; idx_rust=next(i for i,s in enumerate(steps) if 'dtolnay/rust-toolchain' in s.get('uses','')); idx_install=next(i for i,s in enumerate(steps) if 'taiki-e/install-action' in s.get('uses','')); assert idx_rust < idx_install; print('OK')"` | `OK` |
| PC-006 | structural | Artifact upload: name=coverage-report, path=lcov.info, retention=14, if=always() | AC-001.4 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['coverage']['steps']; up=[s for s in steps if 'upload-artifact' in s.get('uses','')][0]; assert up['with']['name']=='coverage-report'; assert 'lcov.info' in up['with']['path']; assert up['with']['retention-days']==14; assert up.get('if')=='always()'; print('OK')"` | `OK` |
| PC-007 | structural | All actions pinned to major versions, no @latest | AC-001.5 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); uses=[s.get('uses','') for s in w['jobs']['coverage']['steps'] if s.get('uses')]; bad=[u for u in uses if u.endswith('@latest')]; assert not bad, f'unpinned: {bad}'; print('OK')"` | `OK` |
| PC-008 | structural | Cache key has cargo-llvm-cov- prefix, restore-keys uses cargo-llvm-cov- | AC-003.1 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['coverage']['steps']; cache=[s for s in steps if 'actions/cache' in s.get('uses','')][0]; k=cache['with']['key']; rk=cache['with']['restore-keys']; assert 'cargo-llvm-cov-' in k; assert 'cargo-llvm-cov-' in rk; print('OK')"` | `OK` |
| PC-009 | structural | paths-filter with *.rs, Cargo.toml, Cargo.lock patterns | AC-002.1, AC-002.2 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['coverage']['steps']; pf=[s for s in steps if 'paths-filter' in s.get('uses','')][0]; filters=pf['with']['filters']; assert '.rs' in filters and 'Cargo.toml' in filters and 'Cargo.lock' in filters; print('OK')"` | `OK` |
| PC-010 | structural | merge_group bypass in coverage job step-level conditions | AC-002.3 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['coverage']['steps']; mg_steps=[s for s in steps if 'merge_group' in str(s.get('if',''))]; assert len(mg_steps)>=3, f'only {len(mg_steps)} steps have merge_group bypass'; print('OK')"` | `OK` |
| PC-011 | structural | timeout-minutes: 20 | Decision #6 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); assert w['jobs']['coverage']['timeout-minutes']==20; print('OK')"` | `OK` |
| PC-012 | structural | No continue-on-error: true on coverage job | Decision #10 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); coe=w['jobs']['coverage'].get('continue-on-error'); assert coe is not True; print('OK')"` | `OK` |
| PC-013 | structural | No needs: key on coverage job | constraint | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); assert 'needs' not in w['jobs']['coverage']; print('OK')"` | `OK` |
| PC-014 | structural | No job-level concurrency on coverage job | Decision #14 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); assert 'concurrency' not in w['jobs']['coverage']; print('OK')"` | `OK` |
| PC-015 | structural | Glossary contains "coverage gate" with CI job, function coverage, threshold | AC-004.1 | `grep 'coverage gate' CLAUDE.md` | match found |
| PC-016 | structural | --fail-under-functions flag present (flag presence only — AC-001.1) | AC-001.1 | `grep '\-\-fail-under-functions' .github/workflows/ci.yml` | match found |
| PC-017 | manual | Baseline measurement confirms >= 80% before merge | AC-001.6 | `cargo llvm-cov --workspace --exclude xtask --exclude ecc-test-support --fail-under-functions 80` | exit 0 |
| PC-018 | manual | Branch protection configured with Coverage Gate | AC-005.1b | `gh api repos/{owner}/{repo}/branches/main/protection --jq '.required_status_checks.contexts[]'` | `Coverage Gate` in output |
| PC-019 | build | Rust build passes | regression | `cargo build` | exit 0 |
| PC-020 | lint | Clippy passes | regression | `cargo clippy -- -D warnings` | exit 0 |
| PC-021 | structural | validate job cache key unchanged | constraint | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); steps=w['jobs']['validate']['steps']; cache=[s for s in steps if 'actions/cache' in s.get('uses','')][0]; k=cache['with']['key']; assert 'cargo-llvm-cov' not in k; print('OK')"` | `OK` |
| PC-022 | structural | Coverage job runs on ubuntu-latest | Decision #11 | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); assert w['jobs']['coverage']['runs-on']=='ubuntu-latest'; print('OK')"` | `OK` |
| PC-023 | structural | Coverage job has no job-level permissions (inherits workflow read-only) | security | `python3 -c "import yaml,pathlib; w=yaml.safe_load(pathlib.Path('.github/workflows/ci.yml').read_text()); assert 'permissions' not in w['jobs']['coverage']; print('OK')"` | `OK` |

### Coverage Check

All ACs covered:
- AC-001.0 → PC-005
- AC-001.1 → PC-003, PC-004
- AC-001.2 → PC-016 (flag presence) + operational verification (post-merge CI run confirms exit behavior)
- AC-001.3 → PC-016 (flag presence) + operational verification (post-merge CI run confirms exit behavior)
- AC-001.4 → PC-006
- AC-001.5 → PC-007
- AC-001.6 → PC-017 (manual)
- AC-002.1 → PC-009
- AC-002.2 → PC-009
- AC-002.3 → PC-010
- AC-003.1 → PC-008
- AC-004.1 → PC-015
- AC-005.1a → PC-002
- AC-005.1b → PC-018 (manual)

Zero uncovered ACs.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | No E2E boundaries affected | — | — | CI-only change, no port/adapter files touched | — | — |

### E2E Activation Rules

No E2E tests to activate — this change is entirely within `.github/workflows/` and `CLAUDE.md`.

## Test Strategy

TDD order (dependency-driven):

1. **PC-015** — Glossary entry (independent, zero coupling to ci.yml)
2. **PC-001, PC-002, PC-011, PC-012, PC-013, PC-014** — Job skeleton (name, timeout, no-needs, no-concurrency, no-continue-on-error)
3. **PC-005, PC-007, PC-008, PC-009, PC-010** — Step infrastructure (toolchain ordering, action pins, cache key, path filter, merge_group bypass)
4. **PC-003, PC-004, PC-006, PC-016** — Core coverage steps (llvm-cov command, flags, artifact upload)
5. **PC-019, PC-020, PC-021** — Regression checks (build, lint, validate-key untouched)
6. **PC-017, PC-018** — Post-merge operational (baseline measurement, branch protection)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CLAUDE.md` | Minor | Append to Glossary bullet | "coverage gate = CI job that enforces ≥80% workspace function coverage via --fail-under-functions; blocks PR merge if threshold not met" | AC-004.1 |
| 2 | `CHANGELOG.md` | Minor | Add entry | "ci: add cargo-llvm-cov coverage gate to CI pipeline" | US-001 |

No ADRs needed (all decisions marked "No").

## SOLID Assessment

**CLEAN** — uncle-bob found no violations. 1 MEDIUM prescription (explicit `continue-on-error: false`) already addressed in Decision #10. CI job follows SRP (one responsibility: measure and gate coverage), OCP (extends CI without modifying existing jobs), and DIP (no `needs:` coupling).

## Robert's Oath Check

**CLEAN** — 1 oath warning (timeout) already addressed in Decision #6 at 20 minutes. Rework ratio 0.14 (healthy). No harmful code, no mess, proof mechanism is the job itself.

## Security Notes

**CLEAR** — No CRITICAL or HIGH findings. All actions use pinned major versions. Permissions are `contents: read` only. No secrets, no external uploads. LCOV artifact contains file paths and line numbers (benign for OSS).

## Rollback Plan

Reverse dependency order:
1. Remove `coverage` job from `.github/workflows/ci.yml`
2. Remove glossary entry from `CLAUDE.md`
3. Remove `CHANGELOG.md` entry
4. Remove `Coverage Gate` from branch protection required checks (admin)

## Bounded Contexts Affected

No bounded contexts affected — CI-only change. No domain files modified.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 1 MEDIUM (explicit continue-on-error — already in Decision #10) |
| Robert | CLEAN | 1 WARNING (timeout — already in Decision #6) |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| AC Coverage | 82 | PASS | AC-001.2/001.3 reclassified as operational verification |
| Execution Order | 88 | PASS | Order defensible, minor wave redundancy |
| Fragility | 75 | PASS | PC-009 *.rs fix, PC-010 YAML-parsed fix applied |
| Rollback | 78 | PASS | Functional, branch protection note added |
| Architecture | 95 | PASS | CI-only, hexagonal rules N/A |
| Blast Radius | 90 | PASS | 3 files, all non-Rust |
| Missing PCs | 80 | PASS | PC-022 (runs-on), PC-023 (permissions) added |
| Doc Plan | 78 | PASS | CHANGELOG present, no ADRs needed |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | CLAUDE.md | modify | AC-004.1, US-004 |
| 2 | .github/workflows/ci.yml | modify | AC-001.*, AC-002.*, AC-003.1, AC-005.1a |
| 3 | CHANGELOG.md | modify | US-001 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-09-cargo-llvm-cov-ci-gate/design.md | Full design |
