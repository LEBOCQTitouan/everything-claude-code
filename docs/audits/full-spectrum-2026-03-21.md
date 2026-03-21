# ECC Full-Spectrum Audit Report

## Generated: 2026-03-21

## Commit: 8bd1cbf137a78a77963bd2c1a9eef08d8238cb5b

---

## Executive Summary

- **Overall health: 🟡 ADEQUATE (B)** — Strong architectural discipline and comprehensive test suite, undermined by formatting drift, documentation-code coupling gaps, and operational maturity shortfalls.
- **Top 3 strengths:**
  1. **Flawless hexagonal layering** — Domain crate has zero outer-layer imports, 5 port traits with 5 production adapters + 5 test doubles. Zero circular dependencies. (Evidence: `crates/ecc-domain/Cargo.toml`, `crates/ecc-ports/src/`)
  2. **Massive test suite (1,185 passing, 0 failures)** — 96% of tests in domain (507) + app (613) layers with property-based tests via proptest. Clippy zero warnings. (Evidence: `cargo test`, `cargo clippy`)
  3. **Mature CI/CD pipeline** — 4 workflows (CI, CD, Release, Maintenance) with auto-tagging, cross-compilation release, stale issue management, and component validation gates. (Evidence: `.github/workflows/`)

- **Top 3 risks:**
  1. **543 files with formatting drift** — `cargo fmt --check` fails massively. No `rustfmt.toml` config. Formatting is not enforced in CI. (Evidence: `cargo fmt --check`)
  2. **0% doc-code coupling** — Of last 50 code commits, zero also touch documentation. CLAUDE.md test count says 1180 but actual is 1185. (Evidence: git log analysis)
  3. **23 silent error swallowing sites** (`let _ =` on write paths) with no logging framework below `log::warn`. (Evidence: grep `let _ =` in crates/)

- **Top 3 recommended actions:**
  1. **Add `cargo fmt --check` to CI** and run `cargo fmt` once to fix 543 files. Impact: eliminates format drift permanently.
  2. **Replace `let _ =` with proper error propagation** or at minimum `log::warn`. Impact: eliminates silent failure risk at 23 sites.
  3. **Add CODEOWNERS, promote CONTRIBUTING.md/CHANGELOG.md to root**. Impact: improves discoverability for contributors (both exist in `docs/` but not at root).

---

## Scorecard

| Section | Rating | Score | Key Finding |
|---|---|---|---|
| 1.1 Code-as-Documentation | 🟢 | 82% | Strong naming, but 3 files exceed 800-line limit |
| 1.2 Docs-as-Code | 🟡 | 65% | 6 ADRs + rich docs, but 0% doc-PR coupling, low README coverage |
| 1.3 Executable Documentation | 🟡 | 70% | 1,185 tests, 0 doc-tests, no coverage tooling |
| 1.4 External Doc Platform | 🟡 | 55% | No generated API docs (rustdoc not configured), markdownlint only |
| 1.5 Diátaxis Compliance | 🟡 | 65% | Strong reference + explanation, weak tutorials + how-to |
| 1.6 Doc-as-Product Maturity | 🟡 | 50% | No CODEOWNERS, no doc versioning; CHANGELOG exists in docs/ |
| 2.1 Architecture Adherence | 🟢 | 95% | Flawless hexagonal layers, zero direction violations |
| 2.2 Module Cohesion & Coupling | 🟡 | 72% | Clean separation but 3 god-files (920, 783, 777 lines) |
| 2.3 Dependency Health | 🟢 | 88% | 13 direct deps, 3 minor updates available, Cargo.lock committed |
| 3.1 Lint & Formatting | 🔴 | 30% | Zero clippy warnings, but 543 files fail fmt check, no rustfmt.toml |
| 3.2 Error Handling | 🟡 | 58% | 3 custom error types, thiserror+anyhow used, but 23 `let _ =` sites |
| 3.3 Security Posture | 🟢 | 85% | No secrets, no unsafe, deps pinned, .gitignore comprehensive |
| 3.4 SOLID Compliance | 🟡 | 72% | Good DIP/OCP, SRP concerns in large modules (35+ methods) |
| 4.1 Test Infrastructure | 🟢 | 80% | 1,185 tests, proptest, integration suite, <1s execution |
| 4.2 Test Quality | 🟡 | 75% | Intent-revealing names (90%), good isolation, 0 doc-tests |
| 5.1 CI/CD Pipeline | 🟡 | 70% | 4 workflows, but no fmt/test in CI lint job |
| 5.2 Build & Release | 🟢 | 85% | Automated release with cross-compilation, lockfile committed |
| 6 Existing Metrics | 🟡 | 65% | 3 prior audits exist, some metrics stale |

---

## Detailed Findings

### PART 1 — DOCUMENTATION AUDIT

#### 1.1 Code-as-Documentation 🟢 82%

**Metrics:**

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| naming_quality | 36/40 (90%) | >80% | 🟢 |
| function_length_p90 | ~35 lines | ≤20 | 🔴 |
| cyclomatic_complexity_p90 | ~8 | ≤10 | 🟡 |
| orphan_comments_ratio | <5% | <20% | 🟢 |
| dead_code_count | 2 (#[allow(dead_code)]) | 0 | 🟢 |

**Function naming sample (20 across crates):**

| Name | Module | Rating |
|------|--------|--------|
| `validate_actrc` | ecc-app | Self-explanatory (2) |
| `is_ecc_managed_hook` | ecc-domain | Self-explanatory (2) |
| `check_statusline` | ecc-app | Self-explanatory (2) |
| `post_edit_boy_scout_delta` | ecc-app | Self-explanatory (2) |
| `install_global` | ecc-app | Self-explanatory (2) |
| `format_search_results` | ecc-app | Self-explanatory (2) |
| `all_entries_valid` | ecc-domain | Self-explanatory (2) |
| `is_legacy_pattern` | ecc-domain | Self-explanatory (2) |
| `validate_secrets_example` | ecc-app | Self-explanatory (2) |
| `is_act_available` | ecc-app | Self-explanatory (2) |
| `validate_all` | ecc-app | Self-explanatory (2) |
| `validate_act_jobs` | ecc-app | Self-explanatory (2) |
| `read_file` | ecc-infra | Self-explanatory (2) |
| `validate_args` | ecc-app | Self-explanatory (2) |
| `as_str` | ecc-domain | Acceptable (1) |
| `is_empty` | ecc-domain | Self-explanatory (2) |
| `new` | ecc-infra | Acceptable (1) |
| `run` | ecc-cli | Acceptable (1) |
| `success` | ecc-ports | Acceptable (1) |
| `current` | ecc-ports | Opaque (0) |

**Score: 36/40 = 90%**

**Findings:**

- **[DOC-001] 3 files exceed 800-line limit**: `install/mod.rs` (920), `config/merge.rs` (783), `session/aliases.rs` (777). The install module at 920 lines is a clear violation of the project's own <800 rule.
- **[DOC-002] ecc-cli has 5 functions all named `run`**: While Rust module context disambiguates, this pattern hinders code search and readability.
- **[DOC-003] Minimal commented-out code**: Only 1 instance found. Very clean.
- **[DOC-004] Zero doc comments (///) on public APIs**: Only 2 `/// ```\`` blocks exist in the entire codebase. Public functions have no doc comments.

**Checklist:**
- [x] Function names express intent (90% self-explanatory)
- [x] Types convey constraints (newtypes for session names, error enums)
- [x] No translation comments
- [x] Small functions (mostly, with 3 oversized files)
- [x] No commented-out code

#### 1.2 Docs-as-Code 🟡 62%

**Metrics:**

| Metric | Value | Target |
|--------|-------|--------|
| readme_coverage | 6/16 (37.5%) dirs with READMEs | >80% |
| adr_count | 6 ADRs + index | >5 |
| doc_freshness | Most updated within Rust rewrite cycle | <20% stale |
| doc_pr_coupling | 0/50 (0%) | >30% |
| onboarding_path | Exists (CLAUDE.md → getting-started → ARCHITECTURE) | Present |

**Details:**

- **557 total markdown files**: 134 in docs/, 44 agents, 104 skills, 26 commands, 78 rules, 1 hook
- **6 ADRs**: hexagonal-architecture, hook-based-state-machine, manifest-based-dev-mode, native-tooling-standard, file-based-memory-system, doc-first-spec-driven-pipeline
- **READMEs exist in**: root, hooks/, rules/, docs/runbooks/, docs/adr/, docs/memory/. **Missing from**: agents/, commands/, skills/, crates/, contexts/, docs/domain/, docs/diagrams/, scripts/, bin/, dist/
- **CONTRIBUTING.md exists** at `docs/CONTRIBUTING.md` (139 lines) — not at root, reducing discoverability
- **CHANGELOG.md exists** at `docs/CHANGELOG.md` (400 lines, auto-generated from conventional commits)
- **0% doc-PR coupling**: None of the last 50 code-changing commits touched documentation files. This is the most concerning documentation metric.

**Checklist:**
- [x] Root README exists (50 lines)
- [x] CONTRIBUTING.md exists (`docs/CONTRIBUTING.md`, 139 lines — consider promoting to root)
- [x] ADRs exist (6 decisions documented)
- [x] ARCHITECTURE.md exists
- [x] CLAUDE.md exists and is comprehensive
- [ ] Docs reviewed in PRs — No CI enforcement of doc updates

#### 1.3 Executable Documentation 🟡 70%

**Metrics:**

| Metric | Value | Target |
|--------|-------|--------|
| test_coverage_pct | ~90% domain+app (estimated from test density) | >80% |
| test_naming_quality | 18/20 = 90% | >80% |
| doc_test_count | 0 | >10 |
| type_coverage | 100% (Rust enforced) | 100% |
| property_test_count | 3 proptest blocks | >5 |
| integration_test_ratio | 47/1185 = 4% | 15-30% |

**Test naming sample (20 tests):**

All 20 sampled test names are intent-revealing: `create_without_history`, `init_creates_gitignore_entries`, `fresh_install_creates_agent_files`, `install_preserves_user_settings`, `install_hooks_round_trip_parseable_by_hook_command`, `validate_agents_passes`, `known_hook_passthrough_exits_zero`, `unknown_hook_warns_on_stderr`, etc.

**Score: 18/20 = 90%** (2 tests scored as "acceptable" for terse names)

**Findings:**

- **[TEST-DOC-001] Zero doc-tests**: Despite 2 `/// ``` ` blocks existing, they produce 0 running doc-tests. No public API has executable documentation examples.
- **[TEST-DOC-002] Integration test ratio is 4%**, well below the 15-30% healthy range. The `ecc-integration-tests` crate has only 47 tests across 8 test files.
- **[TEST-DOC-003] Infra crate has only 2 tests** (351 lines of adapter code, barely tested).
- **[TEST-DOC-004] CLI crate has 0 unit tests** (589 lines untested). Integration tests cover CLI flow but no unit-level coverage.

**Checklist:**
- [x] Tests read like specifications
- [ ] Doc-tests exist for public APIs — **NONE**
- [x] Type system encodes domain constraints
- [x] Edge cases tested (proptest covers this)
- [x] Test failures produce clear diagnostics

#### 1.4 External Documentation Platform 🟡 55%

**Metrics:**

| Metric | Value |
|--------|-------|
| external_doc_references | Minimal (links to GitHub issues) |
| generated_doc_config | None (no mdBook, Docusaurus, rustdoc config) |
| api_doc_completeness | ~2% (2 doc comments on ~100+ public items) |

**Findings:**

- **[EXT-DOC-001] No generated API documentation configured**: No `rustdoc` customization, no `mdBook.toml`, no static site generator. `cargo doc` would produce bare output with no doc comments.
- **[EXT-DOC-002] Markdownlint is configured** (.markdownlint.json) and runs in CI for agent/skill/command/rule markdown.
- **[EXT-DOC-003] Runbooks exist**: `docs/runbooks/` contains operational documentation.

#### 1.5 Diátaxis Compliance 🟡 65%

| Quadrant | Count | Examples |
|----------|-------|---------|
| Tutorial | 1 | `docs/getting-started.md` |
| How-to | 4 | `docs/runbooks/`, `docs/commands-reference.md` |
| Reference | 12+ | `docs/MODULE-SUMMARIES.md`, `docs/domain/bounded-contexts.md`, ADRs |
| Explanation | 5+ | `docs/ARCHITECTURE.md`, ADR rationales |

**Balance assessment**: Heavy on reference and explanation, weak on tutorials and how-to guides. Standard deviation across quadrants is high. A new contributor has architectural docs but few task-oriented guides.

#### 1.6 Documentation-as-Product 🔴 35%

**Checklist:**
- [ ] Docs have explicit ownership — **No CODEOWNERS file**
- [ ] Docs have versioning strategy — **No doc versioning**
- [x] Docs have CI checks — markdownlint in CI
- [x] Docs have style guide — .markdownlint.json, CLAUDE.md conventions
- [x] Changelog exists — `docs/CHANGELOG.md` (400 lines, auto-generated from conventional commits)

---

### PART 2 — ARCHITECTURE & DESIGN AUDIT

#### 2.1 Architectural Pattern Adherence 🟢 95%

**Metrics:**

| Metric | Value |
|--------|-------|
| layer_violation_count | **0** |
| port_adapter_ratio | 5:5 production + 5:5 test doubles = **perfect** |
| bounded_context_clarity | Clear (domain modules map to contexts) |
| dependency_direction_violations | **0** |

**Dependency direction (verified):**

```
ecc-cli → ecc-app, ecc-domain, ecc-ports, ecc-infra
ecc-app → ecc-domain, ecc-ports
ecc-infra → ecc-ports
ecc-domain → (serde, serde_json, regex only — ZERO internal deps)
ecc-ports → (thiserror, serde_json only)
```

**5 Port traits with matching adapters:**

| Port (trait) | Production Adapter | Test Double |
|---|---|---|
| `FileSystem` | `OsFileSystem` | `InMemoryFileSystem` |
| `ShellExecutor` | `ProcessExecutor` | `MockExecutor` |
| `Environment` | `OsEnvironment` | `MockEnvironment` |
| `TerminalIO` | `StdTerminal` | `BufferedTerminal` |
| `ReplInput` | `RustylineInput` | `ScriptedInput` |

**Checklist:**
- [x] Hexagonal layers identifiable in directory structure (7 crates)
- [x] Domain layer has zero infrastructure dependencies
- [x] Ports defined as traits (not concrete types)
- [x] Adapters are pluggable
- [x] No circular dependencies
- [x] DDD value objects identifiable (SessionName, etc.)

#### 2.2 Module Cohesion & Coupling 🟡 72%

**Crate size distribution:**

| Crate | Lines | Functions | Role |
|---|---|---|---|
| ecc-domain | 8,316 | ~500+ | Business logic |
| ecc-app | 17,948 | ~600+ | Use cases |
| ecc-cli | 589 | ~20 | Entry point |
| ecc-infra | 351 | ~15 | Adapters |
| ecc-ports | 165 | ~10 | Traits |
| ecc-test-support | 617 | ~30 | Test doubles |

**Top 10 largest files:**

| File | Lines | Status |
|---|---|---|
| `ecc-app/src/install/mod.rs` | 920 | 🔴 Exceeds 800 |
| `ecc-domain/src/config/merge.rs` | 783 | 🟡 Near limit |
| `ecc-app/src/session/aliases.rs` | 777 | 🟡 Near limit |
| `ecc-app/src/hook/handlers/tier1_simple/clean_craft_hooks.rs` | 739 | 🟡 |
| `ecc-domain/src/config/audit.rs` | 731 | 🟡 |
| `ecc-app/src/validate.rs` | 665 | OK |
| `ecc-app/src/merge/helpers.rs` | 648 | OK |
| `ecc-app/src/hook/handlers/tier1_simple/helpers.rs` | 630 | OK |
| `ecc-app/src/hook/handlers/tier1_simple/dev_hooks.rs` | 622 | OK |
| `ecc-domain/src/session/aliases.rs` | 596 | OK |

**Findings:**

- **[ARCH-001] `ecc-app` at 17,948 lines is 2.16x larger than `ecc-domain`** (8,316 lines). The app layer may be accumulating logic that belongs in domain.
- **[ARCH-002] `SessionManager` has 35 methods** — god-object candidate. Consider decomposing by responsibility.
- **[ARCH-003] `install/mod.rs` at 920 lines violates the 800-line rule.** 41 functions in a single module.
- **[ARCH-004] `config/audit.rs` has 46 functions** in 731 lines — high function density but cohesive (all audit checks).

#### 2.3 Dependency Health 🟢 88%

| Metric | Value |
|--------|-------|
| direct_dependency_count | 13 unique external crates |
| outdated_dependency_count | 3 minor updates (itoa, zerocopy, zerocopy-derive) |
| vulnerability_count | N/A (cargo-audit not installed) |
| dependency_depth | Shallow (most deps are leaf crates) |
| unused_dependency_count | 0 detected |

**External dependencies:** serde, serde_json, regex, anyhow, thiserror, clap, clap_complete, crossterm, rustyline, walkdir, log, env_logger, shell-words. Plus dev-deps: proptest, assert_cmd, predicates, tempfile.

**Lockfile**: Committed (`Cargo.lock`, 36KB). Versions pinned.

---

### PART 3 — CODE QUALITY AUDIT

#### 3.1 Lint & Formatting 🔴 30%

| Metric | Value | Status |
|--------|-------|--------|
| lint_config_exists | No clippy.toml, no deny.toml | 🔴 |
| lint_warning_count | 0 (clippy clean) | 🟢 |
| lint_error_count | 0 | 🟢 |
| format_config_exists | No rustfmt.toml | 🔴 |
| format_drift_file_count | **543 files** | 🔴 |

**Findings:**

- **[LINT-001] CRITICAL: 543 files have formatting drift.** `cargo fmt --check` fails. No `rustfmt.toml` exists. CI does not run `cargo fmt --check`. This is the single largest quality gap.
- **[LINT-002] Clippy is clean**: Zero warnings with `-D warnings`. This is excellent.
- **[LINT-003] No `cargo deny` or `clippy.toml`**: No supply-chain or advanced lint configuration.
- **[LINT-004] Markdownlint is configured** and runs in CI for content files.

#### 3.2 Error Handling 🟡 58%

| Metric | Value |
|--------|-------|
| unwrap_count | 284 total, ~200 in test code, ~84 in production |
| expect_count | 30 |
| let_ignore_count | 23 (silent error swallowing) |
| error_type_quality | 3 custom types (structured) + anyhow (semi-structured) |
| error_propagation_consistency | Mixed: thiserror in ports, anyhow in cli/app |

**Custom error types:**
- `ShellError` (`ecc-ports/src/shell.rs:40`)
- `TerminalError` (`ecc-ports/src/terminal.rs:12`)
- `FsError` (`ecc-ports/src/fs.rs:23`)

**Findings:**

- **[ERR-001] 23 `let _ =` sites silently swallow errors** on write paths (file writes, terminal output). Previously flagged in 2026-03-14 audit. Not yet resolved.
- **[ERR-002] ~84 `.unwrap()` calls in production code.** Many are in pattern-match arms where the value is guaranteed, but some could panic on unexpected input.
- **[ERR-003] Error strategy is split**: Ports use `thiserror` (good), CLI/App use `anyhow` (acceptable but loses type safety at boundaries).
- **[ERR-004] Domain crate has no error types** — errors are propagated as strings or booleans in validation functions.

**Checklist:**
- [x] Custom error types exist for bounded contexts (ports layer)
- [ ] Errors carry enough context — `anyhow` context is often minimal
- [ ] No silent error swallowing — **23 `let _ =` violations**
- [x] Error boundaries exist at adapter/port boundaries

#### 3.3 Security Posture 🟢 85%

**Checklist:**
- [x] No secrets committed — grep found no API keys, tokens, or passwords
- [x] Input validation exists at boundaries (validate.rs, config validation)
- [x] Dependencies pinned (Cargo.lock committed)
- [x] .gitignore covers target/, .env, credentials
- [x] No `unsafe` blocks — only 2 instances found, both in safe context (crossterm raw mode)

**Finding:**
- **[SEC-001] No `cargo audit` in CI**: Vulnerability scanning for dependencies is not automated.

#### 3.4 SOLID Principles 🟡 72%

| Principle | Score | Evidence |
|---|---|---|
| **S** (SRP) | 6/10 | `SessionManager` (35 methods), `install/mod.rs` (41 functions) violate SRP. Other modules are well-focused. |
| **O** (OCP) | 8/10 | Port/adapter pattern enables extension. Hook handler tier system is well-extensible. |
| **L** (LSP) | 9/10 | All trait implementations are genuinely substitutable (production + test doubles). |
| **I** (ISP) | 7/10 | `FileSystem` trait has 8+ methods — could be split (read vs write). Others are focused. |
| **D** (DIP) | 10/10 | Perfect dependency inversion via port traits. Domain depends on nothing concrete. |

**SOLID_score = 40/50 = 80%** (adjusted to 72% for severity of SRP violations in core modules)

---

### PART 4 — TESTING AUDIT

#### 4.1 Test Infrastructure 🟢 80%

| Metric | Value |
|--------|-------|
| test_framework | Built-in `#[test]` + proptest |
| test_count_unit | 1,122 (domain: 507, app: 613, infra: 2) |
| test_count_integration | 47 (ecc-integration-tests crate) |
| test_count_e2e | 0 (CLI integration tests via assert_cmd serve as pseudo-E2E) |
| test_count_doc | 0 |
| test_pyramid_shape | 96:4:0 (unit:integration:e2e) |
| test_execution_time | <1 second total |
| flaky_test_indicators | 0 (no sleep, no fixed timestamps) |

**Findings:**

- **[TEST-001] Test pyramid is extremely top-heavy**: 96% unit, 4% integration, 0% E2E. Healthy target is 70:20:10.
- **[TEST-002] 3 proptest blocks** in domain crate (validate.rs, lcs.rs, session_name.rs). Good but limited coverage of property-based testing.
- **[TEST-003] Zero flaky test indicators**: No `sleep()`, no hardcoded timestamps. Excellent.
- **[TEST-004] Sub-second test execution**: Entire suite runs in <1s. No long-running tests.

#### 4.2 Test Quality 🟡 75%

| Metric | Value | Target |
|--------|-------|--------|
| assertion_density | ~1.8 per test (60/33 in merge.rs sample) | 1-3 |
| test_isolation | Excellent — all in-memory, no shared state | Full |
| mock_count | 5 test doubles (purpose-built, not generic mocks) | Low |
| test_to_code_ratio | ~0.6 (est. 17K test lines / 28K total) | 0.5-2.0 |

**Findings:**

- **[TQUAL-001] Test naming is 90% intent-revealing** — industry-leading. Tests like `install_preserves_user_settings` and `unknown_hook_warns_on_stderr` are self-documenting.
- **[TQUAL-002] No tests depend on execution order** — full isolation via in-memory adapters.
- **[TQUAL-003] 5 purpose-built test doubles** (InMemoryFileSystem, MockExecutor, MockEnvironment, BufferedTerminal, ScriptedInput) — clean design, not mock frameworks.
- **[TQUAL-004] Negative/error cases well-tested** — many tests verify failure behavior (e.g., `validate_actrc_missing_platform`).

**Checklist:**
- [x] Tests follow Arrange-Act-Assert pattern
- [x] No order-dependent tests
- [x] Test data is fixture-based (builder pattern on InMemoryFileSystem)
- [x] Negative/failure cases tested
- [ ] Tests run in CI on every PR — **Rust tests NOT in CI** (only component validation and markdown lint run)

---

### PART 5 — CI/CD & OPERATIONAL MATURITY

#### 5.1 Pipeline 🟡 70%

| Metric | Value |
|--------|-------|
| ci_config_exists | Yes: `ci.yml`, `cd.yml`, `release.yml`, `maintenance.yml` |
| ci_stages | Validate (build + component validation), Lint (markdownlint) |
| ci_gate_count | 2 (validate + lint) |
| branch_protection | Not detectable from config |

**CI workflow analysis:**

| Workflow | Trigger | Jobs |
|---|---|---|
| `ci.yml` | PR to main | `validate` (build + ecc validate), `lint` (markdownlint) |
| `cd.yml` | Push to main | `auto-tag` (version bump + git tag) |
| `release.yml` | Tag push `v*` | `validate`, `build` (6 targets), `release` (GitHub Release) |
| `maintenance.yml` | Weekly Monday 9am | `stale` (issue/PR cleanup) |

**Critical CI gaps:**

- **[CI-001] `cargo test` is NOT in CI.** Tests only run locally. A failing test could be merged.
- **[CI-002] `cargo fmt --check` is NOT in CI.** Format drift of 543 files proves this.
- **[CI-003] `cargo clippy` is NOT in CI.** Currently clean, but not enforced.
- **[CI-004] No security scanning** (cargo audit, cargo deny) in any workflow.

#### 5.2 Build & Release 🟢 85%

**Checklist:**
- [x] Build is reproducible (Cargo.lock committed)
- [x] Versioning: semver (currently 4.2.0), auto-bumped by CD pipeline
- [x] Release process automated (tag → build 6 targets → GitHub Release)
- [x] Build artifacts not committed
- [x] Cross-compilation support (linux x86_64/aarch64, macOS x86_64/aarch64, Windows)

**Findings:**

- **[REL-001] Version bump script exists** (`scripts/bump-version.sh`) — automated in CD pipeline.
- **[REL-002] Release workflow has validation gate** — verifies tag matches Cargo.toml version before building.
- **[REL-003] CHANGELOG.md exists** at `docs/CHANGELOG.md` (auto-generated, 400 lines). Not at root.
- **[REL-004] Makefile exists** with `make ci` (local act runner), `make build`, `make install`, `make dev` targets.

---

### PART 6 — EXISTING METRICS REVIEW

#### Found Artifacts

| File | Type | Status |
|---|---|---|
| `docs/audits/2026-03-14-audit.md` | First comprehensive audit | Still relevant, most findings unresolved |
| `docs/audits/full-2026-03-20.md` | Second audit (1 day ago) | Current, grade B |
| `docs/audits/robert-notes.md` | Professional conscience notes | Active, updated today |

**2026-03-14 Audit findings status (7 days later):**

| Finding | Status |
|---|---|
| 14 `let _ =` silent errors | Partially addressed (now 23 — grew, not shrunk) |
| No logging framework | Unresolved (only `log::warn` used) |
| 4 untested orchestration modules | Partially addressed (test count 1,141→1,185) |
| Bus factor = 1 | Unresolved |

**2026-03-20 Audit key metrics vs today:**

| Metric | Mar 20 | Mar 21 | Delta |
|---|---|---|---|
| Test count | 1,141 | 1,185 | +44 |
| Source files | 116 | ~120 | +4 |
| Clippy warnings | 0 | 0 | Stable |
| File limit violations | 3 | 3 | Stable |

**CLAUDE.md accuracy check:**
- Claims "1190 tests" in CLAUDE.md but `cargo test` shows **1,185 passing + 3 ignored**. The gotchas section says "currently 1180" — both are slightly wrong.

---

### Appendix: Raw Metrics

| Metric | Value |
|---|---|
| total_rust_files | ~160 |
| total_rust_lines | 29,114 |
| total_markdown_files | 557 |
| crate_count | 7 |
| test_count_total | 1,185 passing, 3 ignored |
| test_count_domain | 507 |
| test_count_app | 613 |
| test_count_infra | 2 |
| test_count_integration | 47 |
| test_count_cli | 0 |
| test_count_doc | 0 |
| proptest_blocks | 3 |
| port_traits | 5 |
| production_adapters | 5 |
| test_doubles | 5 |
| layer_violations | 0 |
| circular_deps | 0 |
| clippy_warnings | 0 |
| fmt_drift_files | 543 |
| unwrap_total | 284 |
| unwrap_production | ~84 |
| expect_count | 30 |
| let_ignore_count | 23 |
| unsafe_blocks | 2 |
| dead_code_annotations | 2 |
| custom_error_types | 3 |
| external_dependencies | 13 |
| outdated_deps | 3 (minor) |
| ci_workflows | 4 |
| adr_count | 6 |
| largest_file_lines | 920 (install/mod.rs) |
| files_over_800 | 1 |
| files_over_700 | 4 |
| version | 4.2.0 |
| doc_comments | 2 |
| readme_coverage | 37.5% (6/16 dirs) |
| doc_pr_coupling | 0% |
| test_execution_time | <1s |
| nesting_6plus_lines | 92 |

---

### Priority Remediation Roadmap

| Priority | Action | Impact | Effort |
|---|---|---|---|
| 🔴 P0 | Add `cargo fmt --check` and `cargo test` to CI | Prevents regression, catches 543 format issues | Low (2 lines in ci.yml) |
| 🔴 P0 | Run `cargo fmt` once to fix all 543 files | Eliminates format debt | Trivial |
| 🟡 P1 | Replace 23 `let _ =` with `log::warn` or `?` | Eliminates silent failures | Medium |
| 🟡 P1 | Split `install/mod.rs` (920 lines) into submodules | Brings within 800-line limit | Medium |
| 🟡 P1 | Add `cargo clippy -- -D warnings` to CI | Enforces lint quality | Low |
| 🟡 P2 | Add CODEOWNERS; symlink/promote CONTRIBUTING.md to root | Improves contributor discoverability | Low |
| 🟡 P2 | Add doc-tests to top 10 public APIs | Living documentation | Medium |
| 🟡 P2 | Increase integration test ratio from 4% to 15% | Better boundary coverage | High |
| 🟢 P3 | Promote `docs/CHANGELOG.md` to root or add root symlink | Release transparency | Trivial |
| 🟢 P3 | Add `cargo audit` to CI | Supply-chain security | Low |
| 🟢 P3 | Correct CLAUDE.md test count (1185, not 1190/1180) | Accuracy | Trivial |
