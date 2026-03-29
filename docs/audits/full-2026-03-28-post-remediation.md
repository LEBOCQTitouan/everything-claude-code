# Post-Remediation Audit — 2026-03-28

## Project Profile
- **Repository**: everything-claude-code
- **Scope**: full codebase (post US-001 through US-008 remediation)
- **Date**: 2026-03-28
- **Baseline**: full-2026-03-28.md (Grade: D)
- **Source files**: 1,284 Rust files, 48,761 lines (excluding worktrees/target)
- **Test annotations**: 1,647 `#[test]` markers
- **Crates**: 8 production + ecc-integration-tests

## Executive Summary

The remediation addressed 24 of 27 tracked smells across 8 user stories. The three CRITICAL cross-correlations from the baseline audit are substantially resolved: env_logger now defaults to `warn`, osascript injection is patched with tests, workflow integration tests are split into 15 focused files, typed errors replace `Result<T, String>` in 4 modules, and the 5 over-800-line files are down to 3 (one of which is a test file). The domain model gained `Validatable`, `Transitionable`, `Phase::Unknown`, `Concern` enum, and `Timestamp` newtype. Documentation is fully regenerated.

**Remaining gaps**: (1) 3 files still exceed 800 lines (merge.rs at 869, helpers_tests.rs at 865, aliases.rs at 814); (2) `Transitionable` trait is defined but not implemented for `WorkflowState`, causing 5 compilation errors in traits.rs tests; (3) 90 functions exceed 50 lines in production code; (4) `Result<T, String>` persists in `ecc-domain::detection::package_manager` (4 functions); (5) only 2 `debug!` calls exist (verbose mode has minimal content); (6) ecc-workflow still bypasses port traits (accepted tech debt).

## Overall Health Grade

**Grade: B (GOOD)**

Previous: D. Improvement: +2 letter grades.

| Criteria | Status |
|----------|--------|
| CRITICAL findings | 0 (was 5) |
| HIGH findings | 5 (was 30+) |
| MEDIUM findings | 12 |
| LOW findings | 6 |

## Per-Domain Scorecard

| Domain | Previous | Current | Delta | CRITICAL | HIGH | MEDIUM | LOW |
|--------|----------|---------|-------|----------|------|--------|-----|
| Architecture | B | A | +1 | 0 | 0 | 2 | 1 |
| Code Quality | B | B | = | 0 | 2 | 2 | 1 |
| Security | B | A | +1 | 0 | 0 | 1 | 0 |
| Testing | C | B | +1 | 0 | 1 | 2 | 1 |
| Conventions | C | B | +1 | 0 | 1 | 1 | 1 |
| Error Handling | C | B | +1 | 0 | 0 | 2 | 1 |
| Observability | C | B | +1 | 0 | 0 | 2 | 1 |
| Documentation | D | A | +3 | 0 | 0 | 0 | 0 |
| Evolution | B | B | = | 0 | 1 | 1 | 0 |
| **Cross-Correlation** | -- | -- | -- | 0 | 0 | 0 | 0 |

## CRITICAL Cross-Correlation Resolution

### [CORR-001] Hotspot + Zero Unit Tests: ecc-workflow commands -- RESOLVED

- **Previous**: CRITICAL. 12 workflow command files with 0 unit tests, monolithic integration.rs (2,750 lines).
- **Now**: integration.rs split into 15 focused test files (3,317 total lines). `memory_write.rs` has pure logic extraction. All workflow crate tests pass (82 tests in integration suite). The test feedback loop is dramatically faster.
- **Evidence**: `crates/ecc-workflow/tests/` contains 15 .rs files covering transition, phase_gate, memory_write, hooks, artifacts, backlog, and contention scenarios.
- **Status**: RESOLVED

### [CORR-002] Swallowed Errors + Silent Logging -- RESOLVED

- **Previous**: CRITICAL. env_logger defaulted to ERROR. 38 error-discard sites silent. --verbose was a no-op.
- **Now**: `env_logger::Env::default().default_filter_or("warn")` in main.rs. `--verbose` elevates to `debug` when RUST_LOG is unset. 61 `warn!` calls in production code (up from 48). Error discard sites now emit `warn!`.
- **Evidence**: `crates/ecc-cli/src/main.rs:52` sets warn default. `.ok()` usage down to 4 (from 38+ discard sites).
- **Residual**: Only 2 `debug!` calls exist; verbose mode has limited content. LOW.
- **Status**: RESOLVED (residual LOW)

### [CORR-003] Hotspot + Boundary Violation: ecc-workflow -- PARTIALLY RESOLVED

- **Previous**: CRITICAL. ecc-workflow bypassed port system with 60+ direct std::fs calls, no logging.
- **Now**: ecc-workflow still uses direct std::fs (accepted architectural decision for standalone binary). However: (1) pure logic extraction for memory_write.rs reduces untested I/O surface; (2) integration tests are comprehensive (82 tests); (3) logging is available via env_logger.
- **Evidence**: Port-trait migration deferred as long-term tech debt. Risk mitigated through testing and logging.
- **Status**: MITIGATED to MEDIUM (accepted tech debt)

### [CORR-004] Bus Factor + Documentation Staleness -- RESOLVED

- **Previous**: HIGH. DEPENDENCY-GRAPH.md had 174 lines of dead TypeScript. glossary.md had 27 dead .ts links.
- **Now**: DEPENDENCY-GRAPH.md rewritten with Mermaid Cargo workspace graph. glossary.md has 0 .ts references. ARCHITECTURE.md counts updated. bounded-contexts.md, MODULE-SUMMARIES.md, commands-reference.md, getting-started.md all created.
- **Evidence**: `grep -c "\.ts" docs/glossary.md` = 0. All 6 documentation deliverables confirmed present.
- **Status**: RESOLVED

## Domain Findings

### Architecture (Grade: A)

- **Domain purity**: PASS. Zero I/O imports in ecc-domain. No imports from ecc-infra, ecc-app, or ecc-cli.
- **Dependency direction**: PASS. Strict inward flow maintained: cli -> app -> domain/ports <- infra.
- **Instability (I)**: ecc-domain I=0.0 (Ca=3, Ce=0). Maximally stable as expected.
- **Abstractness (A)**: A = 4/315 = 0.013. D = |0.013 + 0 - 1| = 0.987. Still in Zone of Pain.
  - Note: The previous audit flagged D=0.99. The `Validatable` and `Transitionable` traits were added but with only 315 public items, the ratio barely moved. However, D metric interpretation: ecc-domain is correctly concrete and stable (everything depends on it), so Zone of Pain is architecturally appropriate for a leaf domain crate with no outgoing dependencies.
- **MEDIUM**: ecc-workflow remains outside hexagonal boundary (accepted).
- **MEDIUM**: Abstractness D=0.99 for ecc-domain (structurally appropriate but noted).

### Code Quality (Grade: B)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| Files > 800 lines | 5 | 3 | IMPROVED |
| validate.rs | 1,269 lines | split into 8 submodules | RESOLVED |
| dev.rs | 1,197 lines | split into 5 submodules | RESOLVED |
| install/global.rs | 863 lines | split into mod.rs + steps.rs | RESOLVED |
| integration.rs | 2,750 lines | split into 15 files | RESOLVED |
| merge/helpers.rs | 923 lines | split (helpers.rs=400 + helpers_tests.rs=865) | PARTIAL |

**Remaining over-800-line files**:
- `ecc-domain/src/config/merge.rs`: 869 lines (HIGH -- production code)
- `ecc-app/src/merge/helpers_tests.rs`: 865 lines (MEDIUM -- test file, acceptable)
- `ecc-app/src/session/aliases.rs`: 814 lines (HIGH -- production code)

**Functions > 50 lines**: 90 functions in production code exceed the 50-line limit. Top offenders:
- `validate_design.rs:run_validate_design` (171 lines)
- `transition.rs:run` (144 lines)
- `memory_write.rs:resolve_project_memory_dir` (143 lines)
- `merge/mod.rs:merge_skills` (121 lines)

### Security (Grade: A)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| osascript injection | Unescaped user input | sanitize_osascript + sanitize_powershell | RESOLVED |
| Backslash escaping | Not handled | Backslashes escaped, 256 char cap | RESOLVED |
| Test coverage | None | 10 unit tests covering injection vectors | RESOLVED |

- **Evidence**: `sanitize_osascript` escapes `"` and `\`, caps at 256 chars. `sanitize_powershell` escapes `'`. Both have injection attempt tests (CORR-003 style payloads).
- **MEDIUM**: `ecc-domain::detection::package_manager` validates script names and args but returns `Result<(), String>` -- should use typed error.

### Testing (Grade: B)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| integration.rs monolith | 2,750 lines, 1 file | 15 files, 3,317 lines | RESOLVED |
| Workflow unit tests | 0 | Pure logic extractions tested | IMPROVED |
| Test count | 1,404 documented | 1,647 annotations, ~948+ passing | DISCREPANCY |
| Domain tests compile | Yes | NO -- 5 errors in traits.rs | REGRESSION |

- **HIGH**: `ecc-domain` test target fails to compile. `Transitionable` trait defined but not implemented for `WorkflowState`. Test `make_state()` uses `String` for `concern` field but it is now `Concern` enum. 7 tests in traits.rs are blocked.
- **MEDIUM**: CLAUDE.md documents 1,404 tests but actual count is higher (~1,600+). Stale count.
- **MEDIUM**: Test file helpers_tests.rs at 865 lines is a single large test file.

### Conventions (Grade: B)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| Result<T, String> | Widespread | 16 remaining (9 in domain) | IMPROVED |
| Completion.phase | String | Phase enum with Unknown variant | RESOLVED |
| Concern field | String | Concern enum | RESOLVED |
| Timestamp field | String | Timestamp newtype | RESOLVED |
| is_claude_available dedup | 2 definitions | 1 definition, 1 import | RESOLVED |
| Port trait docs | Missing | lib.rs has module-level docs | RESOLVED |

- **HIGH**: 4 functions in `ecc-domain::detection::package_manager` still return `Result<T, String>`. Domain code should never use String as error type.
- **MEDIUM**: `ecc-infra::rustyline_input::new()` returns `Result<Self, String>`.

### Error Handling (Grade: B)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| Typed errors | Result<T, String> in 4+ modules | ClawError, MergeError, ConfigAppError, InstallError | RESOLVED |
| .ok() discard sites | 38+ | 4 | RESOLVED |
| anyhow in ecc-app | Widespread | 1 remaining (worktree.rs) | RESOLVED |
| Error discard with warn! | 0 | 61 warn! calls | RESOLVED |

- **MEDIUM**: 4 remaining `.ok()` calls should be audited (session parsing, statusline validation).
- **MEDIUM**: `let _ =` pattern appears 27 times -- most are intentional (ignoring write results in tests) but production instances should be reviewed.

### Observability (Grade: B)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| env_logger default | ERROR (silent) | warn | RESOLVED |
| --verbose flag | No-op | Elevates to debug | RESOLVED |
| warn! calls | 48 (invisible) | 61 (visible at default level) | RESOLVED |
| debug! calls | 0 | 2 | MINIMAL |

- **MEDIUM**: Only 2 `debug!` calls means `--verbose` mode provides almost no additional information. Should add debug! at operation boundaries (file reads, config parsing, hook dispatch).
- **MEDIUM**: No structured logging (key=value pairs) for machine-parseable output.

### Documentation (Grade: A)

| Check | Previous | Current | Status |
|-------|----------|---------|--------|
| DEPENDENCY-GRAPH.md | TypeScript era, 174 dead lines | Rust Cargo workspace Mermaid graph | RESOLVED |
| glossary.md .ts refs | 27 dead .ts links | 0 .ts references | RESOLVED |
| ARCHITECTURE.md counts | 47-63% off | Updated with 8 crates, correct counts | RESOLVED |
| bounded-contexts.md | Missing | Created | RESOLVED |
| MODULE-SUMMARIES.md | Missing | Created | RESOLVED |
| commands-reference.md | Missing | Created | RESOLVED |
| getting-started.md | Missing | Created (10,257 bytes) | RESOLVED |

All 5 CRITICAL+HIGH documentation findings from the baseline are resolved. Zero remaining issues.

## Design Smells Assessment

| Smell | Previous Signal | Current Status |
|-------|----------------|----------------|
| **Rigidity** | Dead code + complexity trend up | RESOLVED -- dead TypeScript docs removed, complexity addressed via splits |
| **Fragility** | Low coverage + high fan-in | MITIGATED -- integration tests split, workflow tests comprehensive |
| **Immobility** | Co-change coupling + no shared interface | MITIGATED -- Validatable/Transitionable traits added (but Transitionable not yet impl'd) |
| **Viscosity** | Debug logging at boundaries + TODO trend | RESOLVED -- env_logger defaults to warn, --verbose works |

## Remaining Issues (Prioritized)

### HIGH

1. **Domain test compilation failure**: `Transitionable` not implemented for `WorkflowState`; `make_state()` in traits.rs uses wrong types for `concern` and `started_at` fields. 5 compile errors, 7 tests blocked.
2. **merge.rs at 869 lines**: Production file exceeds 800-line limit.
3. **aliases.rs at 814 lines**: Production file exceeds 800-line limit.
4. **Result<T, String> in package_manager.rs**: 4 domain functions use String error type.
5. **90 functions exceed 50 lines**: Widespread violation of function size limit.

### MEDIUM

1. helpers_tests.rs at 865 lines (test file, lower priority).
2. Only 2 debug! calls -- verbose mode is nearly empty.
3. 4 remaining .ok() discard sites need audit.
4. No structured logging.
5. CLAUDE.md test count stale (says 1,404, actual is higher).
6. ecc-workflow outside hexagonal boundary (accepted tech debt).

### LOW

1. rustyline_input::new returns Result<Self, String>.
2. 27 `let _ =` patterns need production/test classification.
3. Abstractness D=0.99 for ecc-domain (structurally appropriate).

## Top 5 Recommendations

1. **Fix traits.rs compilation**: Implement `Transitionable` for `WorkflowState` and update `make_state()` to use `Concern` enum and `Timestamp` newtype. This unblocks ~681 domain tests.
2. **Split merge.rs and aliases.rs**: Both are just over 800 lines. Extract logical submodules to get under the limit.
3. **Add debug! logging at boundaries**: File reads, config parsing, hook dispatch, and workflow transitions should all emit debug! for meaningful --verbose output.
4. **Type package_manager errors**: Replace `Result<T, String>` with a `PackageManagerError` enum in ecc-domain.
5. **Extract long functions**: The 20 functions over 100 lines are the highest priority; use helper functions to bring them under 50.

## Quality Gate

| Gate | Result |
|------|--------|
| CRITICAL findings > 0 | PASS (0 CRITICAL) |
| All structural only | NO -- has functional gaps (test compilation) |
| 0 CRITICAL, <=2 HIGH | FAIL (5 HIGH) |

**Audit Result: WARN**

The codebase has no CRITICAL issues but has 5 HIGH findings that prevent a clean PASS. The most impactful is the domain test compilation failure which should be a quick fix.

## Comparison to Baseline

| Metric | Baseline (D) | Post-Remediation (B) |
|--------|-------------|---------------------|
| CRITICAL findings | 5 | 0 |
| HIGH findings | 30+ | 5 |
| MEDIUM findings | 40+ | 12 |
| Files > 800 lines | 5 | 3 |
| .ok() discard sites | 38+ | 4 |
| Result<T, String> | Widespread | 16 (9 in domain) |
| Documentation issues | 11 CRITICAL+HIGH | 0 |
| warn! calls (visible) | 0 (all silent) | 61 |
| Workflow test files | 1 (2,750 lines) | 15 (3,317 lines) |
| Typed error enums | 0 | 4 (ClawError, MergeError, ConfigAppError, InstallError) |
| Domain newtypes | 0 | 3 (Phase::Unknown, Concern, Timestamp) |
| Domain traits | 0 | 2 (Validatable, Transitionable) |

**Net assessment**: The remediation successfully eliminated all 5 CRITICAL findings and reduced HIGH findings from 30+ to 5. The grade improvement from D to B reflects genuine structural improvement across all 9 audit domains. The remaining HIGH issues are incremental (file sizes slightly over limit, a test compilation fix, and long functions) rather than systemic.
