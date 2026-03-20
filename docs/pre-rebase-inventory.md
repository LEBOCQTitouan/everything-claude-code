# ECC Pre-Rebase Inventory

> Generated: 2026-03-19
> Branch: `main` (commit `cb5ebc7`)
> Status: clean working tree

---

## Table of Contents

1. [Project Identity](#1-project-identity)
2. [CLAUDE.md Architecture](#2-claudemd-architecture)
3. [Settings Surface](#3-settings-surface)
4. [Commands Inventory](#4-commands-inventory)
5. [Skills Inventory](#5-skills-inventory)
6. [Agents Inventory](#6-agents-inventory)
7. [Rules Inventory](#7-rules-inventory)
8. [Hooks Inventory](#8-hooks-inventory)
9. [MCP Configuration](#9-mcp-configuration)
10. [Source Code Map](#10-source-code-map)
11. [Git State](#11-git-state)
12. [Cross-Reference Matrix](#12-cross-reference-matrix)
13. [The Robert Agent](#13-the-robert-agent)
14. [Existing Workflow Patterns](#14-existing-workflow-patterns)
15. [.gitignore and Ignored Paths](#15-gitignore-and-ignored-paths)
16. [Orphans & Dangling References](#16-orphans--dangling-references)
17. [Preservation Checklist](#17-preservation-checklist)

---

## 1. Project Identity

**Name:** everything-claude-code (ECC)
**Version:** 4.0.0-alpha.1
**Edition:** Rust 2024
**License:** MIT
**Repository:** https://github.com/LEBOCQTitouan/everything-claude-code
**Binary:** `ecc` (from `crates/ecc-cli/src/main.rs`)
**Upstream:** https://github.com/affaan-m/everything-claude-code

### Workspace Members (7 crates)

| Crate | Description |
|-------|-------------|
| `ecc-domain` | Pure business logic — zero I/O dependencies |
| `ecc-ports` | Port trait definitions (FileSystem, ShellExecutor, Environment, TerminalIO) |
| `ecc-app` | Application use cases — orchestrates domain + ports |
| `ecc-infra` | Production adapters (OS filesystem, process executor, terminal) |
| `ecc-cli` | CLI binary entry point (`ecc` command) |
| `ecc-test-support` | Test doubles (InMemoryFileSystem, MockExecutor, MockEnvironment) |
| `ecc-integration-tests` | Binary-level integration tests |

### Dependencies (all)

**Workspace-level:**

| Dependency | Version | Purpose |
|-----------|---------|---------|
| serde | 1 (derive) | Serialization |
| serde_json | 1 | JSON handling |
| thiserror | 2 | Error handling |
| anyhow | 1 | Error handling |
| regex | 1 | Text processing |
| walkdir | 2 | Filesystem traversal |
| clap | 4 (derive, env) | CLI parsing |
| clap_complete | 4 | Shell completions |
| rustyline | 15 | Interactive REPL |
| crossterm | 0.28 | Terminal I/O |

**Per-crate additional:**

| Crate | Dependency | Version |
|-------|-----------|---------|
| ecc-app | log | 0.4 |
| ecc-cli | log | 0.4 |
| ecc-cli | env_logger | 0.11 |
| ecc-domain (dev) | proptest | 1 |
| ecc-integration-tests | tempfile | 3 |
| ecc-integration-tests | assert_cmd | 2 |
| ecc-integration-tests | predicates | 3 |
| ecc-integration-tests | shell-words | 1 |

### Build Configuration

```toml
[profile.release]
strip = true
lto = true
```

### Scripts (from CLAUDE.md, no package.json)

| Command | Description |
|---------|-------------|
| `cargo test` | Run all 1180 Rust tests |
| `cargo clippy -- -D warnings` | Lint with zero warnings |
| `cargo build --release` | Build release binary |
| `npm run lint` | Lint Markdown files with markdownlint |

---

## 2. CLAUDE.md Architecture

### Files Found: 3

#### 2.1 `CLAUDE.md` (root, 106 lines)

Project-level instructions. Contains: project overview, architecture tree, CLI commands table (`ecc version/install/init/audit/hook/validate/claw/completion`), slash commands table (9 commands), scripts table, gotchas (5 items including `ecc-domain` zero-I/O rule, hooks.json location, test count 1180), development notes (hexagonal architecture, file naming).

**Rules defined:**
1. `ecc-domain` must have zero I/O imports
2. Agent `model` field controls which Claude model runs the agent
3. `hooks.json` lives in `hooks/`, not the project root
4. Skill directory name must match the `name` field in its frontmatter
5. Test count (1180) must be updated after adding/removing tests
6. Command workflows in `commands/` are mandatory — follow every phase exactly

**File references:** None (@-imports).

#### 2.2 `crates/CLAUDE.md` (22 lines)

Crate-level architecture guide. Defines the strict dependency direction: `ecc-cli → ecc-app → ecc-ports ← ecc-infra`, `→ ecc-domain`. Documents each crate's responsibility and gotchas.

**Rules defined:**
1. `ecc-domain` depending on `ecc-infra` is a build-breaking violation
2. Adding new I/O capability requires port trait in `ecc-ports` first
3. All tests use `ecc-test-support` doubles — never real filesystem/executor
4. Integration tests live in `ecc-integration-tests`, not inside individual crates

#### 2.3 `examples/CLAUDE.md` (99 lines)

Generic template for new projects. Not project-specific — a reference example. Contains boilerplate rules for code organization, style, testing, security, patterns, env vars, and available ECC commands.

**Contradictions:** None found between the three files.

---

## 3. Settings Surface

### 3.1 `.claude/settings.json` — DOES NOT EXIST

### 3.2 `.claude/settings.local.json` — EXISTS

**Permissions (allow):**

```
Bash(npm:*), Bash(npx:*), Bash(node:*),
Bash(git status:*), Bash(git log:*), Bash(git diff:*),
Bash(git add:*), Bash(git commit:*), Bash(git branch:*),
Bash(git checkout:*), Bash(git stash:*),
Bash(ls:*), Bash(wc:*), Bash(sort:*), Bash(grep:*),
Bash(find tests:*),
WebFetch(domain:api.github.com), WebFetch(domain:raw.githubusercontent.com),
Read(//tmp/**), Read(//private/tmp/**),
Bash(cargo test:*), Bash(cargo clippy:*),
Bash(cargo build:*), Bash(cargo check:*), Bash(cargo run:*)
```

No hooks, env vars, or MCP servers in this file.

### 3.3 `~/.claude/settings.json` (global)

**Effort level:** `medium`

**Enabled plugins (8):**
- `claude-md-management@claude-plugins-official`
- `commit-commands@claude-plugins-official`
- `figma@claude-plugins-official`
- `github@claude-plugins-official`
- `pyright-lsp@claude-plugins-official`
- `rust-analyzer-lsp@claude-plugins-official`
- `security-guidance@claude-plugins-official`
- `typescript-lsp@claude-plugins-official`

**Deny rules:**
- `.env` files and variants (read/write)
- `~/.ssh/**`, `~/.aws/**`, `~/.gnupg/**`
- PEM and key files
- `rm -rf:*`, `chmod 777:*`
- `curl|sh` patterns
- Force push to main/master

**Status line:** Command-based, runs `bash ~/.claude/statusline-command.sh`

**Hooks:** 21 hook definitions across 10 lifecycle events (PreToolUse, PostToolUse, PostToolUseFailure, PreCompact, PostCompact, SessionStart, SessionEnd, Stop, SubagentStart, SubagentStop, ConfigChange, InstructionsLoaded, WorktreeCreate, WorktreeRemove, UserPromptSubmit).

---

## 4. Commands Inventory

### 9 Commands

| # | Path | Command | Uses $ARGS | Shell Interp | Agents Delegated |
|---|------|---------|-----------|-------------|------------------|
| 1 | `commands/plan.md` | `/plan` | Modes: feature/refactor/security | No | requirements-analyst, planner, tdd-guide, e2e-runner, architect, security-reviewer |
| 2 | `commands/build-fix.md` | `/build-fix` | No | Yes | None |
| 3 | `commands/verify.md` | `/verify` | Yes (quick/full/pre-commit/pre-pr/review/--fix/--path/--focus) | Yes | code-reviewer, arch-reviewer, go-reviewer, python-reviewer |
| 4 | `commands/e2e.md` | `/e2e` | --skip-plan | Yes | e2e-runner, code-reviewer |
| 5 | `commands/doc-suite.md` | `/doc-suite` | Yes (--scope/--phase/--base/--dry-run/--comments-only/--skip-plan) | Yes | doc-analyzer, doc-generator, diagram-generator, doc-validator, doc-reporter, doc-updater, doc-orchestrator |
| 6 | `commands/audit.md` | `/audit` | Yes (--scope/--domain/--boundary/--window/--diff/--skip-plan/--quick) | Yes | evolution-analyst, arch-reviewer, security-reviewer, test-auditor, observability-auditor, error-handling-auditor, convention-auditor, component-auditor |
| 7 | `commands/backlog.md` | `/backlog` | Subcommands: add/list/promote/archive/match | No | backlog-curator |
| 8 | `commands/uncle-bob-audit.md` | `/uncle-bob-audit` | No | No | robert |
| 9 | `commands/optimize.md` | `/optimize` | Yes (--scope/--check/--report-only/--skip-plan) | No | None (uses skills directly) |

**All commands reference the `prompt-optimizer` skill** (Phase 0 prompt refinement) except `/uncle-bob-audit`.

---

## 5. Skills Inventory

**Total: 97 skill directories**

### Skills with Supporting Files

| Skill | Supporting File | Lines | Description |
|-------|----------------|-------|-------------|
| api-reference-gen | references/bad-examples.md | 167 | API doc anti-patterns |
| architecture-gen | assets/c4-template.md | — | C4 diagram template |
| failure-modes | references/common-patterns.md | — | Common failure patterns |
| security-review | cloud-infrastructure-security.md | — | Cloud security patterns |
| skill-stocktake | scripts/scan.sh | — | Skill scanning script |
| skill-stocktake | scripts/quick-diff.sh | — | Changed skill detection |
| skill-stocktake | scripts/save-results.sh | — | Result persistence |
| strategic-compact | suggest-compact.sh | — | Compaction threshold tracker |
| symbol-extraction | references/language-patterns.md | — | Language-specific patterns |

### Complete Skill List (97)

1. agent-harness-construction
2. agentic-engineering
3. ai-first-engineering
4. api-design
5. api-reference-gen
6. architecture-gen
7. architecture-review
8. autonomous-loops
9. backend-patterns
10. backlog-management
11. behaviour-extraction
12. blueprint
13. changelog-gen
14. claude-workspace-optimization
15. clean-craft
16. clickhouse-io
17. coding-standards
18. component-principles
19. config-extraction
20. configure-ecc
21. content-hash-cache-pattern
22. continuous-agent-loop
23. convention-consistency
24. cost-aware-llm-pipeline
25. cpp-coding-standards
26. cpp-testing
27. csharp-patterns
28. csharp-testing
29. database-migrations
30. deep-research
31. dependency-docs
32. deployment-patterns
33. diagram-generation
34. django-patterns
35. django-security
36. django-tdd
37. django-verification
38. doc-analysis
39. doc-drift-detector
40. doc-gap-analyser
41. doc-guidelines
42. doc-quality-scoring
43. docker-patterns
44. e2e-testing
45. enterprise-agent-ops
46. error-handling-audit
47. eval-harness
48. evolutionary-analysis
49. example-extraction
50. failure-modes
51. foundation-models-on-device
52. frontend-patterns
53. git-narrative
54. golang-patterns
55. golang-testing
56. iterative-retrieval
57. java-coding-standards
58. jpa-patterns
59. json-patterns
60. kotlin-coroutines-flows
61. kotlin-exposed-patterns
62. kotlin-ktor-patterns
63. kotlin-patterns
64. kotlin-testing
65. nutrient-document-processing
66. observability-audit
67. plankton-code-quality
68. postgres-patterns
69. project-guidelines-example
70. prompt-optimizer
71. python-patterns
72. python-testing
73. readme-gen
74. regex-vs-llm-structured-text
75. runbook-gen
76. rust-patterns
77. rust-testing
78. search-first
79. security-review
80. security-scan
81. shell-patterns
82. shell-testing
83. skill-stocktake
84. springboot-patterns
85. springboot-security
86. springboot-tdd
87. springboot-verification
88. strategic-compact
89. swift-actor-persistence
90. swift-concurrency-6-2
91. swift-protocol-di-testing
92. swiftui-patterns
93. symbol-extraction
94. tdd-workflow
95. test-architecture
96. typescript-testing
97. yaml-patterns

---

## 6. Agents Inventory

**Total: 41 agent files**

### By Category

**Orchestrators (3):**

| Agent | Model | Description |
|-------|-------|-------------|
| audit-orchestrator | opus | Delegates to domain audit agents in parallel |
| doc-orchestrator | opus | Coordinates full documentation pipeline |
| requirements-analyst | opus | Decomposes raw input into User Stories |

**Architects (2):**

| Agent | Model | Description |
|-------|-------|-------------|
| architect | opus | Strategic — Hexagonal Architecture + DDD at system level |
| architect-module | opus | Module-level — internals of a single component |

**Clean Code (2):**

| Agent | Model | Description |
|-------|-------|-------------|
| uncle-bob | opus | Clean Architecture/SOLID consultant — diagnoses only |
| robert | opus | Professional conscience — reviews the review process |

**Reviewers (10):**

| Agent | Model | Language/Domain |
|-------|-------|-----------------|
| code-reviewer | opus | General code review |
| arch-reviewer | opus | Architecture quality |
| go-reviewer | opus | Go |
| python-reviewer | opus | Python |
| rust-reviewer | opus | Rust |
| typescript-reviewer | opus | TypeScript |
| java-reviewer | opus | Java |
| kotlin-reviewer | opus | Kotlin |
| cpp-reviewer | opus | C/C++ |
| csharp-reviewer | opus | C# |
| shell-reviewer | opus | Shell |
| database-reviewer | opus | PostgreSQL |

**Build Resolvers (3):**

| Agent | Model | Target |
|-------|-------|--------|
| build-error-resolver | sonnet | Generic build/TypeScript |
| go-build-resolver | sonnet | Go |
| kotlin-build-resolver | sonnet | Kotlin/Gradle |

**TDD & Testing (3):**

| Agent | Model | Description |
|-------|-------|-------------|
| tdd-guide | sonnet | TDD specialist — RED/GREEN/REFACTOR |
| e2e-runner | sonnet | E2E testing — Playwright |
| test-auditor | sonnet | Test architecture quality |

**Documentation (6):**

| Agent | Model | Description |
|-------|-------|-------------|
| doc-analyzer | opus | Codebase documentation analysis |
| doc-generator | haiku | Write doc comments, summaries, glossary |
| doc-validator | opus | Accuracy, quality scoring, drift detection |
| doc-reporter | haiku | Coverage percentages, staleness |
| doc-updater | haiku | Codemaps, READMEs |
| diagram-generator | haiku | Mermaid diagram generation |

**Audit Specialists (5):**

| Agent | Model | Description |
|-------|-------|-------------|
| evolution-analyst | opus | Git history mining — hotspots, coupling |
| observability-auditor | sonnet | Log levels, structured logging, metrics |
| error-handling-auditor | sonnet | Swallowed errors, taxonomy |
| convention-auditor | sonnet | Naming, pattern consistency |
| component-auditor | opus | 6 component principles, main sequence distance |

**Other (3):**

| Agent | Model | Description |
|-------|-------|-------------|
| planner | opus | Implementation planning |
| backlog-curator | sonnet | Backlog curation for /backlog command |
| refactor-cleaner | sonnet | Dead code cleanup |
| harness-optimizer | opus | Agent harness configuration |
| security-reviewer | opus | OWASP Top 10, vulnerability detection |

---

## 7. Rules Inventory

**Total: 76 rule files** (1 README + 10 common + 65 language-specific across 14 directories)

### Common Rules (no path globs — global)

| File | Summary |
|------|---------|
| `rules/common/coding-style.md` | Immutability, file organization (200-400 lines, 800 max), error handling, input validation, quality checklist |
| `rules/common/development-workflow.md` | MANDATORY: Research → Plan → TDD → E2E → Verify → Commit |
| `rules/common/git-workflow.md` | MANDATORY: Conventional commits, atomic commit cadence, PR workflow |
| `rules/common/testing.md` | MANDATORY: 80% coverage, unit/integration/E2E, TDD cycle |
| `rules/common/security.md` | Pre-commit security checklist, secret management, response protocol |
| `rules/common/patterns.md` | Skeleton projects, repository pattern, API response format |
| `rules/common/performance.md` | Model selection (haiku/sonnet/opus), context window management |
| `rules/common/hooks.md` | Hook types, auto-accept guidelines, TodoWrite best practices |
| `rules/common/agents.md` | Agent table, command→agent mapping, parallel execution |
| `rules/README.md` | Meta-documentation for the rules system |

### Language-Specific Rules (14 directories)

Each language directory contains 3-5 files (coding-style, hooks, patterns, security, testing) with path globs for activation.

| Directory | Glob Patterns | Files |
|-----------|---------------|-------|
| `golang/` | `**/*.go`, `**/go.mod`, `**/go.sum` | 5 |
| `python/` | `**/*.py`, `**/*.pyi` | 5 |
| `typescript/` | `**/*.ts`, `**/*.tsx`, `**/*.js`, `**/*.jsx` | 5 |
| `swift/` | `**/*.swift`, `**/Package.swift` | 5 |
| `rust/` | `**/*.rs`, `**/Cargo.toml`, `**/Cargo.lock` | 5 |
| `cpp/` | `**/*.c`, `**/*.cpp`, `**/*.h`, `**/*.hpp`, `**/*.cc`, `**/*.cxx`, `**/CMakeLists.txt` | 5 |
| `java/` | `**/*.java`, `**/pom.xml`, `**/build.gradle`, `**/build.gradle.kts` | 5 |
| `shell/` | `**/*.sh`, `**/*.bash`, `**/*.zsh` | 5 |
| `json/` | `**/*.json`, `**/*.jsonc` | 3 (no hooks/testing) |
| `yaml/` | `**/*.yml`, `**/*.yaml` | 3 (no hooks/testing) |
| `csharp/` | `**/*.cs`, `**/*.csproj`, `**/*.sln`, `**/Directory.Build.props` | 5 |
| `kotlin/` | `**/*.kt`, `**/*.kts`, `**/build.gradle.kts`, `**/settings.gradle.kts` | 5 |
| `perl/` | `**/*.pl`, `**/*.pm`, `**/*.t`, `**/*.psgi`, `**/*.cgi` | 5 |
| `php/` | `**/*.php`, `**/composer.json` (+ variants) | 5 |

---

## 8. Hooks Inventory

### Primary: `hooks/hooks.json`

Uses `$schema: https://json.schemastore.org/claude-code-settings.json`. All hooks invoke the `ecc-hook` binary (which delegates to `ecc hook <id>`).

**Hook Events Covered (16):**

| Event | Count | Example Hook IDs |
|-------|-------|------------------|
| PreToolUse | 7 | pre:bash:dev-server-block, pre:bash:tmux-reminder, pre:bash:git-push-reminder, pre:write:doc-file-warning, pre:edit-write:suggest-compact, pre:edit:boundary-crossing, pre:edit:stepdown-warning |
| PostToolUse | 10 | post:bash:pr-created, post:bash:build-complete, post:quality-gate, post:edit:format, post:edit:typecheck, post:edit:console-warn, post:edit:boy-scout-delta, post:edit:naming-review, post:edit:newspaper-check, post:edit-write:doc-coverage-reminder |
| PostToolUseFailure | 1 | post:failure:error-context |
| PreCompact | 1 | pre:compact |
| PostCompact | 1 | post:compact:state-save |
| SessionStart | 1 | session:start |
| SessionEnd | 1 | session:end:marker |
| Stop | 8 | stop:notify, stop:uncommitted-reminder, stop:check-console-log, stop:session-end, stop:evaluate-session, stop:cost-tracker, stop:oath-reflection, stop:craft-velocity |
| SubagentStart | 1 | subagent:start:log |
| SubagentStop | 1 | subagent:stop:log |
| UserPromptSubmit | 1 | pre:prompt:context-inject |
| ConfigChange | 1 | config:change:log |
| InstructionsLoaded | 1 | instructions:loaded:validate |
| WorktreeCreate | 1 | worktree:create:init |
| WorktreeRemove | 1 | stop:worktree-cleanup-reminder |

### Shell Scripts Referenced by Hooks

| Script | Purpose |
|--------|---------|
| `bin/ecc-hook` | Shim: `exec ecc hook "$@"` |
| `bin/ecc-shell-hook.sh` | Symlink resolver, delegates to run-with-flags-shell.sh |
| `scripts/hooks/run-with-flags-shell.sh` | Shell-based hook runner with profile support |
| `skills/strategic-compact/suggest-compact.sh` | Tracks tool count, suggests /compact |

### Other Shell Scripts (not hook-related)

| Script | Purpose |
|--------|---------|
| `install.sh` | 877-line install orchestrator (install/init/help/completion/version/update/uninstall) |
| `scripts/get-ecc.sh` | Curl-pipe installer — downloads from GitHub Releases |
| `scripts/uninstall-ecc.sh` | Uninstaller with --force and --keep-config |
| `scripts/bump-version.sh` | Semver version bumper for Cargo.toml |
| `statusline/statusline-command.sh` | Claude Code statusline formatter |
| `skills/skill-stocktake/scripts/scan.sh` | Skill scanning for stocktake |
| `skills/skill-stocktake/scripts/quick-diff.sh` | Changed skill detection |
| `skills/skill-stocktake/scripts/save-results.sh` | Stocktake result persistence |

---

## 9. MCP Configuration

### `.mcp.json` — DOES NOT EXIST in project root

### `mcp-configs/mcp-servers.json` — Reference catalog

16 MCP server configurations (all with placeholder API keys):

| Server | Transport | Notes |
|--------|-----------|-------|
| github | npx @modelcontextprotocol/server-github | Requires GITHUB_PERSONAL_ACCESS_TOKEN |
| firecrawl | npx firecrawl-mcp | Requires FIRECRAWL_API_KEY |
| supabase | npx @supabase/mcp-server-supabase | Requires --project-ref |
| memory | npx @modelcontextprotocol/server-memory | No auth needed |
| sequential-thinking | npx @modelcontextprotocol/server-sequential-thinking | No auth needed |
| vercel | HTTP https://mcp.vercel.com | — |
| railway | npx @railway/mcp-server | — |
| cloudflare-docs | HTTP https://docs.mcp.cloudflare.com/mcp | — |
| cloudflare-workers-builds | HTTP https://builds.mcp.cloudflare.com/mcp | — |
| cloudflare-workers-bindings | HTTP https://bindings.mcp.cloudflare.com/mcp | — |
| cloudflare-observability | HTTP https://observability.mcp.cloudflare.com/mcp | — |
| clickhouse | HTTP https://mcp.clickhouse.cloud/mcp | — |
| exa-web-search | npx exa-mcp-server | Requires EXA_API_KEY |
| context7 | npx @context7/mcp-server | — |
| magic | npx @magicuidesign/mcp@latest | — |
| filesystem | npx @modelcontextprotocol/server-filesystem | Needs path argument |

---

## 10. Source Code Map

### Directory Skeleton (depth 2)

```
.claude/            Agent memory, worktrees
agents/             41 agent definitions
bin/                ecc-hook, ecc-shell-hook.sh
commands/           9 slash command definitions
contexts/           3 context modes (dev, research, review)
crates/
  ecc-domain/       Pure business logic (zero I/O)
  ecc-ports/        Trait definitions
  ecc-app/          Application use cases
  ecc-infra/        Production adapters
  ecc-cli/          CLI binary
  ecc-test-support/ Test doubles
  ecc-integration-tests/  Integration tests
dist/               Legacy TypeScript compiled output (stale, gitignored)
docs/
  audits/           Audit reports
  CODEMAPS/         Token-lean architecture maps
  diagrams/         20 Mermaid diagrams
ecc-rs/             npm package bridge (target/, npm/)
examples/           10 CLAUDE.md templates + statusline config
hooks/              hooks.json + README.md
mcp-configs/        MCP server reference catalog
node_modules/       markdownlint dependencies
rules/              76 rule files (common/ + 14 language dirs)
schemas/            3 JSON schemas
scripts/            hooks/run-with-flags-shell.sh, bump-version.sh, get-ecc.sh, uninstall-ecc.sh
skills/             97 skill directories
statusline/         statusline-command.sh
target/             Rust build output (gitignored)
```

### Rust Source Files (113 .rs files)

**ecc-domain** (25 source files):

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root |
| `src/ansi.rs` | ANSI color formatting |
| `src/paths.rs` | Path resolution logic |
| `src/time.rs` | Time utilities |
| `src/claw/` | NanoClaw REPL domain (command, compact, export, metrics, model, prompt, search, session_name, turn) |
| `src/config/` | Configuration domain (audit, clean, deny_rules, detect, gitignore, hook_types, manifest, merge, statusline, validate) |
| `src/detection/` | Language/framework/package-manager detection |
| `src/diff/` | LCS-based diff (formatter, lcs) |
| `src/hook_runtime/` | Hook runtime (profiles) |
| `src/session/` | Session management (aliases, manager) |

**ecc-ports** (6 source files):

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root |
| `src/fs.rs` | FileSystem trait |
| `src/shell.rs` | ShellExecutor trait |
| `src/env.rs` | Environment trait |
| `src/terminal.rs` | TerminalIO trait |
| `src/repl.rs` | ReplInput trait |

**ecc-app** (49 source files):

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root |
| `src/act_ci.rs` | CI action support |
| `src/audit.rs` | Config audit use case |
| `src/detect.rs` | Detection use case |
| `src/smart_merge.rs` | Smart merge use case |
| `src/validate.rs` | Validation use case |
| `src/version.rs` | Version use case |
| `src/claw/` | NanoClaw app layer (claude_runner, dispatch, handlers, skill_loader, storage) |
| `src/config/` | Config use cases (audit/checks+hooks_diff, clean, detect, gitignore, manifest, merge) |
| `src/detection/` | Detection use cases (framework, language, package_manager) |
| `src/hook/` | Hook dispatching (handlers: tier1_simple, tier2_notify, tier2_tools, tier3_session) |
| `src/install/` | Install orchestration (helpers) |
| `src/merge/` | Merge use cases (helpers, prompt) |
| `src/session/` | Session use cases (aliases) |
| `tests/` | act_ci_integration.rs, ecc_root_integration.rs |

**ecc-infra** (6 source files):

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root |
| `src/os_env.rs` | OS environment adapter |
| `src/os_fs.rs` | OS filesystem adapter |
| `src/process_executor.rs` | Process execution adapter |
| `src/rustyline_input.rs` | Rustyline REPL adapter |
| `src/std_terminal.rs` | Standard terminal adapter |

**ecc-cli** (10 source files):

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point |
| `src/commands/mod.rs` | Command routing |
| `src/commands/audit.rs` | Audit CLI command |
| `src/commands/claw.rs` | NanoClaw CLI command |
| `src/commands/completion.rs` | Shell completion generation |
| `src/commands/hook.rs` | Hook CLI command |
| `src/commands/init.rs` | Init CLI command |
| `src/commands/install.rs` | Install CLI command |
| `src/commands/validate.rs` | Validate CLI command |
| `src/commands/version.rs` | Version CLI command |

**ecc-test-support** (6 source files):

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root |
| `src/in_memory_fs.rs` | In-memory filesystem double |
| `src/mock_executor.rs` | Mock shell executor |
| `src/mock_env.rs` | Mock environment |
| `src/buffered_terminal.rs` | Buffered terminal double |
| `src/scripted_input.rs` | Scripted REPL input |

**ecc-integration-tests** (10 test files):

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root |
| `tests/common/mod.rs` | Test helpers |
| `tests/audit_flow.rs` | Audit integration tests |
| `tests/clean_flow.rs` | Clean integration tests |
| `tests/cli_parsing.rs` | CLI parsing tests |
| `tests/hook_dispatch.rs` | Hook dispatch tests |
| `tests/init_flow.rs` | Init integration tests |
| `tests/install_flow.rs` | Install integration tests |
| `tests/validate_flow.rs` | Validate integration tests |
| `tests/version_completion.rs` | Version/completion tests |

### Documentation Files

| Path | Description |
|------|-------------|
| `docs/API-SURFACE.md` | Public Rust exports across 6 crates |
| `docs/ARCHITECTURE.md` | System architecture, diagram manifest |
| `docs/CHANGELOG.md` | Conventional commit changelog |
| `docs/CONTRIBUTING.md` | Contribution guidelines |
| `docs/DEPENDENCY-GRAPH.md` | Module dependency edges (stale — TypeScript era) |
| `docs/DOC-COVERAGE.md` | 88% overall, per-crate breakdown |
| `docs/DOC-QUALITY.md` | A- (8.6/10) quality score |
| `docs/GLOSSARY.md` | 14 domain + 11 infrastructure terms |
| `docs/MODULE-SUMMARIES.md` | Crate overview redirect |
| `docs/RUNBOOK.md` | Operational procedures |
| `docs/longform-guide.md` | Deep guide on token economics, memory, context |
| `docs/security-guide.md` | Attack vectors, sandboxing, AgentShield |
| `docs/shortform-guide.md` | Intro to skills, hooks, subagents, MCPs |
| `docs/token-optimization.md` | Model selection, settings recommendations |
| `docs/audits/2026-03-14-audit.md` | First codebase health audit (B+) |
| `docs/diagrams/INDEX.md` | Index of 18 diagrams |
| `docs/diagrams/CUSTOM.md` | Custom diagram registry (13 diagrams) |
| `docs/diagrams/*.md` | 18 individual Mermaid diagram files |
| `docs/CODEMAPS/INDEX.md` | Quick stats, entry points, crate tree |
| `docs/CODEMAPS/*.md` | architecture, backend, data, dependencies |

### Schemas

| File | Description |
|------|-------------|
| `schemas/hooks.schema.json` | JSON Schema for hooks configuration (21 event types) |
| `schemas/package-manager.schema.json` | JSON Schema for package manager config |
| `schemas/doc-manifest.schema.json` | JSON Schema for documentation manifest |

### Contexts

| File | Mode | Behavior |
|------|------|----------|
| `contexts/dev.md` | Active development | Write first, working > perfect |
| `contexts/research.md` | Exploration | Read widely, don't code prematurely |
| `contexts/review.md` | PR review | Read thoroughly, severity-prioritized findings |

### Examples

| File | Description |
|------|-------------|
| `examples/CLAUDE.md` | Generic project template |
| `examples/configs/statusline.json` | Statusline settings example |
| `examples/typescript-CLAUDE.md` | TypeScript/Node.js template |
| `examples/rust-api-CLAUDE.md` | Rust API (Axum + SQLx) template |
| `examples/saas-nextjs-CLAUDE.md` | Next.js + Supabase + Stripe template |
| `examples/python-CLAUDE.md` | Python 3.12+ template |
| `examples/go-microservice-CLAUDE.md` | Go microservice (gRPC + REST) template |
| `examples/django-api-CLAUDE.md` | Django + DRF template |
| `examples/swift-CLAUDE.md` | Swift 6+ template |
| `examples/user-CLAUDE.md` | Global ~/.claude/CLAUDE.md template |

---

## 11. Git State

### Last 20 Commits

```
cb5ebc7 docs: optimize CLAUDE.md files per /optimize audit
6135f98 docs: update CLAUDE.md with /optimize command
4d5f69b feat: add /optimize command for Claude workspace optimization
d46f938 feat: add claude-workspace-optimization skill with 11-check rubric
acc8bc7 feat: show git repo and directory in statusline instead of project name
7d6832e refactor: extract tier2_notify module for notification handlers
af3e3cf feat: expand VALID_HOOK_EVENTS to all 21 Claude Code event types
675f406 docs: update CLAUDE.md test count to 1180
07c19f1 test: add integration tests for new hook handlers
616f327 feat: wire 3 new handlers into dispatch and hooks.json
56c4782 feat: add config_change_log and worktree_create_init handlers
c889519 feat: add instructions_loaded_validate handler
d1fabe6 docs: update CLAUDE.md with accurate counts and missing entries
7457d4a fix: correct hookItem.type enum in hooks schema
eab20df feat: add statusline audit check
da30eb7 feat: wire statusline into install pipeline
55412fe feat: implement statusline app layer
f0e89ee feat: implement statusline domain logic
0b2d7cb test: add statusline domain tests
4957fc0 feat: add bundled statusline script
```

### All Branches

```
Local:
  ci/add-cd-pr-gate
  ci/upgrade-actions-node24
* main
  simplify-commands

Remote (origin):
  origin/HEAD -> origin/main
  origin/chore/audit-cleanup
  origin/ci/upgrade-actions-node24
  origin/docs/readme-tree-structure
  origin/main

Remote (upstream):
  upstream/HEAD -> upstream/main
  upstream/codex/fix-ci-followup
  upstream/codex/fix-ci-mainline
  upstream/codex/fix-main-windows-root-identity
  upstream/codex/fix-opencode-project-meta
  upstream/codex/orchestration-harness-skills
  upstream/codex/pr414-refresh-followup
  upstream/codex/release-1.8.0-core
  upstream/dev
  upstream/ecc-tools/everything-claude-code-1772664976069
  upstream/feat/skills-prompt-optimizer
  upstream/main
```

### All Tags (27)

```
v0.6.0, v1.0.0, v1.0.1, v1.0.1-beta.0, v1.0.10, v1.0.11, v1.0.12,
v1.0.13, v1.0.2, v1.0.3, v1.0.4, v1.0.5, v1.0.6, v1.0.7, v1.0.8,
v1.0.9, v1.1.0, v1.2.0, v1.3.0, v1.4.0, v1.4.1, v1.5.0, v1.6.0,
v1.7.0, v1.8.0, v2.0.0, v4.0.0-alpha.1
```

### Remotes

```
origin    https://github.com/LEBOCQTitouan/everything-claude-code.git
upstream  https://github.com/affaan-m/everything-claude-code.git
```

### Last 5 Commits Diff Stats

```
CLAUDE.md                                     |  17 +-
commands/optimize.md                          | 227 ++++++++++++++++++
crates/CLAUDE.md                              |  22 ++
skills/claude-workspace-optimization/SKILL.md | 321 ++++++++++++++++++++++++++
statusline/statusline-command.sh              |  33 +--
5 files changed, 605 insertions(+), 15 deletions(-)
```

---

## 12. Cross-Reference Matrix

### Commands → Agents

| Command | Agents Invoked |
|---------|---------------|
| `/plan` | requirements-analyst, planner, tdd-guide, e2e-runner, architect, security-reviewer |
| `/build-fix` | (none — self-contained) |
| `/verify` | code-reviewer, arch-reviewer, go-reviewer, python-reviewer |
| `/e2e` | e2e-runner, code-reviewer |
| `/doc-suite` | doc-analyzer, doc-generator, diagram-generator, doc-validator, doc-reporter, doc-updater, doc-orchestrator |
| `/audit` | evolution-analyst, arch-reviewer, security-reviewer, test-auditor, observability-auditor, error-handling-auditor, convention-auditor, component-auditor |
| `/backlog` | backlog-curator |
| `/uncle-bob-audit` | robert |
| `/optimize` | (none — uses skills directly) |

### Agents → Skills (from frontmatter `skills` field)

| Agent | Skills |
|-------|--------|
| robert | clean-craft, component-principles |
| uncle-bob | coding-standards |
| architect | architecture-review |
| architect-module | architecture-review |
| planner | tdd-workflow |
| tdd-guide | tdd-workflow |
| code-reviewer | coding-standards |
| arch-reviewer | architecture-review |
| security-reviewer | security-review |
| build-error-resolver | coding-standards |
| refactor-cleaner | coding-standards |
| e2e-runner | e2e-testing |
| go-reviewer | golang-patterns, golang-testing |
| go-build-resolver | golang-patterns |
| python-reviewer | python-patterns, python-testing |
| rust-reviewer | rust-patterns, rust-testing |
| typescript-reviewer | coding-standards, typescript-testing |
| java-reviewer | java-coding-standards |
| kotlin-reviewer | kotlin-patterns |
| kotlin-build-resolver | kotlin-patterns |
| cpp-reviewer | cpp-coding-standards, cpp-testing |
| csharp-reviewer | csharp-patterns, csharp-testing |
| shell-reviewer | shell-patterns, shell-testing |
| database-reviewer | postgres-patterns |
| test-auditor | test-architecture |
| error-handling-auditor | error-handling-audit |
| convention-auditor | convention-consistency |
| observability-auditor | observability-audit |
| harness-optimizer | agent-harness-construction |
| audit-orchestrator | architecture-review |
| evolution-analyst | evolutionary-analysis |
| doc-orchestrator | doc-guidelines |
| doc-analyzer | doc-analysis, symbol-extraction, behaviour-extraction |
| doc-generator | api-reference-gen, changelog-gen, readme-gen |
| doc-validator | doc-quality-scoring, doc-drift-detector, doc-gap-analyser |
| doc-reporter | doc-quality-scoring |
| doc-updater | doc-guidelines |
| diagram-generator | diagram-generation |
| requirements-analyst | blueprint |
| backlog-curator | backlog-management, prompt-optimizer |

### Agent Delegation Chains

| Agent | Delegates To |
|-------|-------------|
| architect | architect-module (via Agent tool) |
| architect-module | uncle-bob (via Agent tool) |
| code-reviewer | uncle-bob (via Agent tool) |
| arch-reviewer | architect, architect-module, uncle-bob (parallel via Agent tool) |
| audit-orchestrator | evolution-analyst, arch-reviewer, security-reviewer, test-auditor, observability-auditor, error-handling-auditor, convention-auditor, component-auditor (parallel) |
| doc-orchestrator | doc-analyzer, doc-generator, doc-validator, doc-reporter, diagram-generator, doc-updater (phased) |
| requirements-analyst | Explore sub-agent (via Agent tool) |

### Commands → Skills (direct)

| Command | Skills Referenced |
|---------|-----------------|
| `/plan` | prompt-optimizer |
| `/build-fix` | prompt-optimizer |
| `/verify` | prompt-optimizer |
| `/e2e` | prompt-optimizer |
| `/doc-suite` | prompt-optimizer, doc-guidelines, doc-quality-scoring |
| `/audit` | prompt-optimizer |
| `/backlog` | backlog-management, prompt-optimizer |
| `/optimize` | prompt-optimizer, claude-workspace-optimization |

### Hooks → Scripts

All hooks in `hooks/hooks.json` invoke `ecc-hook "<hook-id>"` which delegates to the Rust `ecc hook` command, which dispatches to handlers in `crates/ecc-app/src/hook/handlers/`.

---

## 13. The Robert Agent

### Complete Definition

```markdown
---
name: robert
description: Professional conscience meta-agent. Reviews the review process itself — evaluates work against the Programmer's Oath, audits ECC's own files for SRP/DRY violations, calculates rework ratio trends, and writes findings to docs/audits/robert-notes.md. Invoked as final phase of /plan, /verify, /audit, and standalone via /uncle-bob-audit.
tools: ["Read", "Grep", "Glob", "Bash", "Write"]
model: opus
skills: ["clean-craft", "component-principles"]
---

*"I will not produce code that I know to be defective."*

You are Robert — the professional conscience of this development session. You do not review application code (that is `uncle-bob`'s job). You review the **review process itself**: was the work done professionally? Were promises kept? Is the tooling clean?

You are invoked:
- As the final phase of `/plan`, `/verify`, and `/audit`
- Standalone via `/uncle-bob-audit`

---

## 1. Oath Check

Evaluate the current session's work against the 9 Programmer's Oath promises. For each relevant promise, write a one-line "oath note":

```
Oath 1 (no harmful code): CLEAN — no defective behavior or structure detected
Oath 2 (no mess): WARNING — 2 functions exceed 40 lines without justification
Oath 3 (proof): CLEAN — all new code has test coverage
Oath 4 (small releases): CLEAN — 6 atomic commits in this session
Oath 5 (fearless improvement): CLEAN — Boy Scout improvement in helpers.rs
Oath 6 (productivity): CLEAN — no throughput-decreasing changes
Oath 7 (easy substitution): WARNING — new service has no port interface
Oath 8 (honest estimates): N/A — no estimates given
Oath 9 (continuous learning): N/A — not applicable this session
```

Only evaluate promises that are relevant to the work done. Skip with "N/A" if not applicable.

Severity mapping:
- `CLEAN` — promise kept
- `WARNING` — minor deviation, note for improvement
- `VIOLATION` — promise broken, requires action

## 2. Self-Audit

Audit ECC's own agent, command, skill, and rule files for internal quality:

**SRP check**: Does each agent file have a single clear responsibility? Flag agents that try to do too much (> 400 lines, multiple unrelated review dimensions mixed together).

**DRY check**: Scan for duplicated instructions across agents/commands. Flag sections that are copy-pasted verbatim in 3+ files (candidates for extraction into a shared skill).

**Consistency check**: Verify that all agents follow the standard frontmatter format. Flag missing `skills` fields, inconsistent `model` choices, or missing `description` fields.

Report findings as:
```
Self-audit:
- [SELF-001] DRY violation: "Commit Cadence" section duplicated in 5 command files → extract to skill
- [SELF-002] SRP concern: planner.md at 450 lines with 8 sections → consider splitting
- [SELF-003] Consistency: 3 agents missing skills field in frontmatter
```

## 3. "Go Well" Metric

Parse the recent git log to calculate the rework ratio:

```bash
git log --oneline -50
```

Count commit types:
- `feat:` → forward progress
- `test:` → forward progress
- `fix:` → rework
- `chore:` → rework (unless `chore(scout):` which is forward progress)
- `refactor:` → neutral (planned improvement)
- `docs:` → forward progress

Calculate:
```
rework_ratio = rework_commits / total_commits
```

Report the ratio with interpretation:
```
"Go well" metric:
  Session commits: 12
  Forward: 8 (feat: 4, test: 3, docs: 1)
  Rework: 3 (fix: 3)
  Neutral: 1 (refactor: 1)
  Rework ratio: 0.25 (Normal — some rework expected)
```

If ratio > 0.40, flag trend concern and suggest investigating friction sources.

## 4. Output

Write findings to `docs/audits/robert-notes.md`:

```markdown
# Robert Notes — YYYY-MM-DD

## Oath Evaluation
[oath notes from section 1]

## Self-Audit
[findings from section 2, or "No issues found."]

## "Go Well" Metric
[rework ratio from section 3]

## Summary
[One-line summary: "N oath warnings, M self-audit findings, rework ratio X.XX"]
```

If no issues are found in any section, write:
```markdown
# Robert Notes — YYYY-MM-DD

All clean.
```

**Important**: Create the `docs/audits/` directory if it does not exist. Overwrite any existing `robert-notes.md` (it represents the latest session evaluation, not a historical record).

---

## Constraints

- Do NOT review application code — that is `uncle-bob`'s domain
- Do NOT produce implementation code — you only diagnose and report
- Do NOT modify any files other than `docs/audits/robert-notes.md`
- Keep the output concise — findings only, no filler
- If everything is clean, say "All clean." and stop
```

### Where Referenced

- Invoked by `/uncle-bob-audit` command (`commands/uncle-bob-audit.md`)
- Referenced as final phase of `/plan`, `/verify`, and `/audit` (in robert's own description)
- Skills: `clean-craft`, `component-principles`

### Rules Enforced

1. 9 Programmer's Oath promises (CLEAN/WARNING/VIOLATION)
2. SRP check on ECC's own files (>400 lines flagged)
3. DRY check across agents/commands (3+ duplicates flagged)
4. Consistency check on frontmatter format
5. Rework ratio calculation from git history (>0.40 = concern)

---

## 14. Existing Workflow Patterns

### Multi-Phase Workflows

All 9 commands define mandatory multi-phase workflows:

| Command | Phases |
|---------|--------|
| `/plan` | 0: Prompt refinement → 0.25: Backlog cross-ref → 0.5: Screaming Architecture → 1: User Stories → 2: Dependency analysis → 3: Plan per story → [confirm] → TDD execution → E2E → Verify |
| `/verify` | 0: Prompt → 1: Build → 2: Types → 3: Lint → 4: Tests+Coverage → 5: Dead code → 6: Code review → 7: Arch review → 7.5: Dependency direction → 8: Console.log audit → 8.5: Cross-phase correlation → 9: Git status |
| `/audit` | 0: Discovery → 1: Evolution (sequential) → 2: Domain audits (parallel) → 3: Cross-domain correlation → 4: Report → 5: Summary |
| `/doc-suite` | 0: Prompt → 1: Source sync → 2: Analyze → 3: Generate → 4: Diagrams → 5: Validate → 6: Report → 7: Codemaps → 8: README → 9: CLAUDE.md challenge |
| `/e2e` | 0: Plan → 1: Generate+Execute → 2: Code review → 3: Recap |
| `/optimize` | 0: Prompt → 1: Discovery → 2: CLAUDE.md audit → 3: Cross-reference → 4: Report → 5: Proposed changes → 6: Apply |

### Blocking Hooks

- `pre:bash:dev-server-block` — blocks dev servers outside tmux
- `pre:edit:boundary-crossing` — blocks domain-layer outward imports
- `pre:bash:git-push-reminder` — reminder before git push

### State Files

- `.claude/agent-memory/` — Agent memory persistence
- `.claude/worktrees/` — Worktree tracking
- `.claude/.ecc-manifest.json` — ECC installation manifest (gitignored)
- `.claude/plans/` — Autonomous loop plans (gitignored)
- `docs/backlog/` — Persistent backlog entries
- `docs/audits/` — Audit reports
- `docs/user-stories/` — Persisted user stories (from /plan)

### Contexts (Dynamic System Prompt Injection)

3 context modes in `contexts/`: `dev.md`, `research.md`, `review.md` — can be loaded via `claude --system-prompt`.

---

## 15. .gitignore and Ignored Paths

### Full `.gitignore`

```
# Environment files
.env
.env.local
.env.*.local
.secrets

# API keys
*.key
*.pem
secrets.json

# OS files
.DS_Store
Thumbs.db

# Editor files
.idea/
.vscode/
*.swp
*.swo

# Rust build output
target/

# Python
__pycache__/
*.pyc

# Task files (Claude Code teams)
tasks/

# Personal configs (if any)
personal/
private/

# MCP configs (may contain API keys)
.mcp.json

# Claude Code local instructions (personal, never commit)
CLAUDE.local.md

# Claude Code local settings (machine-specific, never commit)
.claude/settings.local.json

# Session templates (not committed)
examples/sessions/*.tmp

# Compiled TypeScript output (stale, migrated to Rust)
dist/

# Local drafts
marketing/

# ECC (Everything Claude Code) generated files
# ECC installation manifest
.claude/.ecc-manifest.json
# Generated architecture docs (regeneratable via /update-codemaps)
docs/CODEMAPS/
# Autonomous loop plans (ephemeral)
.claude/plans/
# End ECC generated files
```

**Coverage analysis:**
- `.claude/settings.local.json` — properly ignored
- `dist/` — properly ignored (stale TypeScript output)
- `.mcp.json` — properly ignored
- `CLAUDE.local.md` — properly ignored
- `target/` — properly ignored
- No `.eslintignore`, `.prettierignore`, or `.markdownlintignore` files exist

---

## 16. Orphans & Dangling References

### Potentially Orphaned Agents (not directly called by any command)

These agents exist but are not directly invoked by any of the 9 commands. They may be invoked indirectly via delegation chains or by users directly:

| Agent | Likely Usage |
|-------|-------------|
| `uncle-bob` | Called by code-reviewer, architect-module, arch-reviewer (delegation) |
| `architect` | Called by arch-reviewer, /plan refactor mode (delegation) |
| `architect-module` | Called by architect (delegation) |
| `refactor-cleaner` | User-invoked directly |
| `harness-optimizer` | User-invoked directly |
| `database-reviewer` | User-invoked directly |

**Verdict:** No true orphans — all agents are either delegated to or available for direct use.

### Potentially Orphaned Skills (not referenced by any agent frontmatter)

Many skills serve as knowledge resources loaded on-demand by Claude Code's skill system, not via agent frontmatter. The following 50+ skills have no direct agent reference but are available as contextual knowledge:

- ai-first-engineering, api-design, autonomous-loops, backend-patterns, blueprint, clickhouse-io, configure-ecc, content-hash-cache-pattern, continuous-agent-loop, cost-aware-llm-pipeline, database-migrations, deep-research, dependency-docs, deployment-patterns, docker-patterns, django-*, enterprise-agent-ops, eval-harness, failure-modes, foundation-models-on-device, frontend-patterns, git-narrative, iterative-retrieval, jpa-patterns, json-patterns, kotlin-coroutines-flows, kotlin-exposed-patterns, kotlin-ktor-patterns, nutrient-document-processing, plankton-code-quality, project-guidelines-example, regex-vs-llm-structured-text, runbook-gen, search-first, security-scan, springboot-*, strategic-compact, swift-*, swiftui-patterns, yaml-patterns

**Verdict:** These are knowledge skills, not orphans. They activate via Claude Code's skill matching system when relevant file types are edited.

### Stale Documentation

- `docs/DEPENDENCY-GRAPH.md` — Documents TypeScript module structure (pre-Rust era)
- `docs/API-SURFACE.md` — Partially TypeScript-era content
- `docs/GLOSSARY.md` — Some stale TypeScript file references

### Stale Artifacts

- `dist/` — Legacy compiled TypeScript output (gitignored but present locally)
- `ecc-rs/` — npm package bridge directory (purpose unclear relative to pure Rust binary)

---

## 17. Preservation Checklist

Every discrete artifact that must survive the rebase.

### Manifest & Config (5)

1. `Cargo.toml` (workspace root)
2. `Cargo.lock`
3. `.gitignore`
4. `.claude/settings.local.json`
5. `hooks/hooks.json`

### CLAUDE.md Files (3)

6. `CLAUDE.md` (root)
7. `crates/CLAUDE.md`
8. `examples/CLAUDE.md`

### Crate Cargo.toml Files (7)

9. `crates/ecc-domain/Cargo.toml`
10. `crates/ecc-ports/Cargo.toml`
11. `crates/ecc-app/Cargo.toml`
12. `crates/ecc-infra/Cargo.toml`
13. `crates/ecc-cli/Cargo.toml`
14. `crates/ecc-test-support/Cargo.toml`
15. `crates/ecc-integration-tests/Cargo.toml`

### Rust Source Files — ecc-domain (25)

16. `crates/ecc-domain/src/lib.rs`
17. `crates/ecc-domain/src/ansi.rs`
18. `crates/ecc-domain/src/paths.rs`
19. `crates/ecc-domain/src/time.rs`
20. `crates/ecc-domain/src/claw/mod.rs`
21. `crates/ecc-domain/src/claw/command.rs`
22. `crates/ecc-domain/src/claw/compact.rs`
23. `crates/ecc-domain/src/claw/export.rs`
24. `crates/ecc-domain/src/claw/metrics.rs`
25. `crates/ecc-domain/src/claw/model.rs`
26. `crates/ecc-domain/src/claw/prompt.rs`
27. `crates/ecc-domain/src/claw/search.rs`
28. `crates/ecc-domain/src/claw/session_name.rs`
29. `crates/ecc-domain/src/claw/turn.rs`
30. `crates/ecc-domain/src/config/mod.rs`
31. `crates/ecc-domain/src/config/audit.rs`
32. `crates/ecc-domain/src/config/clean.rs`
33. `crates/ecc-domain/src/config/deny_rules.rs`
34. `crates/ecc-domain/src/config/detect.rs`
35. `crates/ecc-domain/src/config/gitignore.rs`
36. `crates/ecc-domain/src/config/hook_types.rs`
37. `crates/ecc-domain/src/config/manifest.rs`
38. `crates/ecc-domain/src/config/merge.rs`
39. `crates/ecc-domain/src/config/statusline.rs`
40. `crates/ecc-domain/src/config/validate.rs`
41. `crates/ecc-domain/src/detection/mod.rs`
42. `crates/ecc-domain/src/detection/framework.rs`
43. `crates/ecc-domain/src/detection/language.rs`
44. `crates/ecc-domain/src/detection/package_manager.rs`
45. `crates/ecc-domain/src/diff/mod.rs`
46. `crates/ecc-domain/src/diff/formatter.rs`
47. `crates/ecc-domain/src/diff/lcs.rs`
48. `crates/ecc-domain/src/hook_runtime/mod.rs`
49. `crates/ecc-domain/src/hook_runtime/profiles.rs`
50. `crates/ecc-domain/src/session/mod.rs`
51. `crates/ecc-domain/src/session/aliases.rs`
52. `crates/ecc-domain/src/session/manager.rs`

### Rust Source Files — ecc-ports (6)

53. `crates/ecc-ports/src/lib.rs`
54. `crates/ecc-ports/src/fs.rs`
55. `crates/ecc-ports/src/shell.rs`
56. `crates/ecc-ports/src/env.rs`
57. `crates/ecc-ports/src/terminal.rs`
58. `crates/ecc-ports/src/repl.rs`

### Rust Source Files — ecc-app (49)

59. `crates/ecc-app/src/lib.rs`
60. `crates/ecc-app/src/act_ci.rs`
61. `crates/ecc-app/src/audit.rs`
62. `crates/ecc-app/src/detect.rs`
63. `crates/ecc-app/src/smart_merge.rs`
64. `crates/ecc-app/src/validate.rs`
65. `crates/ecc-app/src/version.rs`
66. `crates/ecc-app/src/claw/mod.rs`
67. `crates/ecc-app/src/claw/claude_runner.rs`
68. `crates/ecc-app/src/claw/dispatch.rs`
69. `crates/ecc-app/src/claw/handlers/mod.rs`
70. `crates/ecc-app/src/claw/handlers/display.rs`
71. `crates/ecc-app/src/claw/handlers/session.rs`
72. `crates/ecc-app/src/claw/skill_loader.rs`
73. `crates/ecc-app/src/claw/storage.rs`
74. `crates/ecc-app/src/config/mod.rs`
75. `crates/ecc-app/src/config/audit/mod.rs`
76. `crates/ecc-app/src/config/audit/checks.rs`
77. `crates/ecc-app/src/config/audit/hooks_diff.rs`
78. `crates/ecc-app/src/config/clean.rs`
79. `crates/ecc-app/src/config/detect.rs`
80. `crates/ecc-app/src/config/gitignore.rs`
81. `crates/ecc-app/src/config/manifest.rs`
82. `crates/ecc-app/src/config/merge.rs`
83. `crates/ecc-app/src/detection/mod.rs`
84. `crates/ecc-app/src/detection/framework.rs`
85. `crates/ecc-app/src/detection/language.rs`
86. `crates/ecc-app/src/detection/package_manager.rs`
87. `crates/ecc-app/src/hook/mod.rs`
88. `crates/ecc-app/src/hook/handlers/mod.rs`
89. `crates/ecc-app/src/hook/handlers/tier1_simple/mod.rs`
90. `crates/ecc-app/src/hook/handlers/tier1_simple/clean_craft_hooks.rs`
91. `crates/ecc-app/src/hook/handlers/tier1_simple/dev_hooks.rs`
92. `crates/ecc-app/src/hook/handlers/tier1_simple/doc_hooks.rs`
93. `crates/ecc-app/src/hook/handlers/tier1_simple/git_hooks.rs`
94. `crates/ecc-app/src/hook/handlers/tier1_simple/helpers.rs`
95. `crates/ecc-app/src/hook/handlers/tier1_simple/meta_hooks.rs`
96. `crates/ecc-app/src/hook/handlers/tier2_notify.rs`
97. `crates/ecc-app/src/hook/handlers/tier2_tools.rs`
98. `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs`
99. `crates/ecc-app/src/hook/handlers/tier3_session/helpers.rs`
100. `crates/ecc-app/src/install/mod.rs`
101. `crates/ecc-app/src/install/helpers.rs`
102. `crates/ecc-app/src/merge/mod.rs`
103. `crates/ecc-app/src/merge/helpers.rs`
104. `crates/ecc-app/src/merge/prompt.rs`
105. `crates/ecc-app/src/session/mod.rs`
106. `crates/ecc-app/src/session/aliases.rs`
107. `crates/ecc-app/tests/act_ci_integration.rs`
108. `crates/ecc-app/tests/ecc_root_integration.rs`

### Rust Source Files — ecc-infra (6)

109. `crates/ecc-infra/src/lib.rs`
110. `crates/ecc-infra/src/os_env.rs`
111. `crates/ecc-infra/src/os_fs.rs`
112. `crates/ecc-infra/src/process_executor.rs`
113. `crates/ecc-infra/src/rustyline_input.rs`
114. `crates/ecc-infra/src/std_terminal.rs`

### Rust Source Files — ecc-cli (10)

115. `crates/ecc-cli/src/main.rs`
116. `crates/ecc-cli/src/commands/mod.rs`
117. `crates/ecc-cli/src/commands/audit.rs`
118. `crates/ecc-cli/src/commands/claw.rs`
119. `crates/ecc-cli/src/commands/completion.rs`
120. `crates/ecc-cli/src/commands/hook.rs`
121. `crates/ecc-cli/src/commands/init.rs`
122. `crates/ecc-cli/src/commands/install.rs`
123. `crates/ecc-cli/src/commands/validate.rs`
124. `crates/ecc-cli/src/commands/version.rs`

### Rust Source Files — ecc-test-support (6)

125. `crates/ecc-test-support/src/lib.rs`
126. `crates/ecc-test-support/src/in_memory_fs.rs`
127. `crates/ecc-test-support/src/mock_executor.rs`
128. `crates/ecc-test-support/src/mock_env.rs`
129. `crates/ecc-test-support/src/buffered_terminal.rs`
130. `crates/ecc-test-support/src/scripted_input.rs`

### Rust Source Files — ecc-integration-tests (10)

131. `crates/ecc-integration-tests/src/lib.rs`
132. `crates/ecc-integration-tests/tests/common/mod.rs`
133. `crates/ecc-integration-tests/tests/audit_flow.rs`
134. `crates/ecc-integration-tests/tests/clean_flow.rs`
135. `crates/ecc-integration-tests/tests/cli_parsing.rs`
136. `crates/ecc-integration-tests/tests/hook_dispatch.rs`
137. `crates/ecc-integration-tests/tests/init_flow.rs`
138. `crates/ecc-integration-tests/tests/install_flow.rs`
139. `crates/ecc-integration-tests/tests/validate_flow.rs`
140. `crates/ecc-integration-tests/tests/version_completion.rs`

### Commands (9)

141. `commands/plan.md`
142. `commands/build-fix.md`
143. `commands/verify.md`
144. `commands/e2e.md`
145. `commands/doc-suite.md`
146. `commands/audit.md`
147. `commands/backlog.md`
148. `commands/uncle-bob-audit.md`
149. `commands/optimize.md`

### Agents (41)

150. `agents/robert.md`
151. `agents/uncle-bob.md`
152. `agents/architect.md`
153. `agents/architect-module.md`
154. `agents/planner.md`
155. `agents/tdd-guide.md`
156. `agents/code-reviewer.md`
157. `agents/arch-reviewer.md`
158. `agents/security-reviewer.md`
159. `agents/build-error-resolver.md`
160. `agents/refactor-cleaner.md`
161. `agents/e2e-runner.md`
162. `agents/go-reviewer.md`
163. `agents/go-build-resolver.md`
164. `agents/python-reviewer.md`
165. `agents/rust-reviewer.md`
166. `agents/typescript-reviewer.md`
167. `agents/java-reviewer.md`
168. `agents/kotlin-reviewer.md`
169. `agents/kotlin-build-resolver.md`
170. `agents/cpp-reviewer.md`
171. `agents/csharp-reviewer.md`
172. `agents/shell-reviewer.md`
173. `agents/database-reviewer.md`
174. `agents/test-auditor.md`
175. `agents/error-handling-auditor.md`
176. `agents/convention-auditor.md`
177. `agents/observability-auditor.md`
178. `agents/harness-optimizer.md`
179. `agents/component-auditor.md`
180. `agents/audit-orchestrator.md`
181. `agents/evolution-analyst.md`
182. `agents/doc-orchestrator.md`
183. `agents/doc-analyzer.md`
184. `agents/doc-generator.md`
185. `agents/doc-validator.md`
186. `agents/doc-reporter.md`
187. `agents/doc-updater.md`
188. `agents/diagram-generator.md`
189. `agents/requirements-analyst.md`
190. `agents/backlog-curator.md`

### Skills (97 SKILL.md + 9 supporting files = 106 files)

191–287. `skills/<name>/SKILL.md` × 97 (see complete list in Section 5)

288. `skills/api-reference-gen/references/bad-examples.md`
289. `skills/architecture-gen/assets/c4-template.md`
290. `skills/failure-modes/references/common-patterns.md`
291. `skills/security-review/cloud-infrastructure-security.md`
292. `skills/skill-stocktake/scripts/scan.sh`
293. `skills/skill-stocktake/scripts/quick-diff.sh`
294. `skills/skill-stocktake/scripts/save-results.sh`
295. `skills/strategic-compact/suggest-compact.sh`
296. `skills/symbol-extraction/references/language-patterns.md`

### Rules (76)

297. `rules/README.md`
298–307. `rules/common/*.md` × 10
308–312. `rules/golang/*.md` × 5
313–317. `rules/python/*.md` × 5
318–322. `rules/typescript/*.md` × 5
323–327. `rules/swift/*.md` × 5
328–332. `rules/rust/*.md` × 5
333–337. `rules/cpp/*.md` × 5
338–342. `rules/java/*.md` × 5
343–347. `rules/shell/*.md` × 5
348–350. `rules/json/*.md` × 3
351–353. `rules/yaml/*.md` × 3
354–358. `rules/csharp/*.md` × 5
359–363. `rules/kotlin/*.md` × 5
364–368. `rules/perl/*.md` × 5
369–373. `rules/php/*.md` × 5

### Hooks (2)

374. `hooks/hooks.json`
375. `hooks/README.md`

### Scripts (8)

376. `bin/ecc-hook`
377. `bin/ecc-shell-hook.sh`
378. `install.sh`
379. `scripts/hooks/run-with-flags-shell.sh`
380. `scripts/get-ecc.sh`
381. `scripts/uninstall-ecc.sh`
382. `scripts/bump-version.sh`
383. `statusline/statusline-command.sh`

### Schemas (3)

384. `schemas/hooks.schema.json`
385. `schemas/package-manager.schema.json`
386. `schemas/doc-manifest.schema.json`

### Contexts (3)

387. `contexts/dev.md`
388. `contexts/research.md`
389. `contexts/review.md`

### MCP Configs (1)

390. `mcp-configs/mcp-servers.json`

### Examples (10)

391. `examples/CLAUDE.md`
392. `examples/configs/statusline.json`
393. `examples/typescript-CLAUDE.md`
394. `examples/rust-api-CLAUDE.md`
395. `examples/saas-nextjs-CLAUDE.md`
396. `examples/python-CLAUDE.md`
397. `examples/go-microservice-CLAUDE.md`
398. `examples/django-api-CLAUDE.md`
399. `examples/swift-CLAUDE.md`
400. `examples/user-CLAUDE.md`

### Documentation (34)

401. `docs/API-SURFACE.md`
402. `docs/ARCHITECTURE.md`
403. `docs/CHANGELOG.md`
404. `docs/CONTRIBUTING.md`
405. `docs/DEPENDENCY-GRAPH.md`
406. `docs/DOC-COVERAGE.md`
407. `docs/DOC-QUALITY.md`
408. `docs/GLOSSARY.md`
409. `docs/MODULE-SUMMARIES.md`
410. `docs/RUNBOOK.md`
411. `docs/longform-guide.md`
412. `docs/security-guide.md`
413. `docs/shortform-guide.md`
414. `docs/token-optimization.md`
415. `docs/audits/2026-03-14-audit.md`
416. `docs/diagrams/INDEX.md`
417. `docs/diagrams/CUSTOM.md`
418. `docs/diagrams/module-dependency-graph.md`
419. `docs/diagrams/install-data-flow.md`
420. `docs/diagrams/hook-execution-flow.md`
421. `docs/diagrams/doc-suite-pipeline.md`
422. `docs/diagrams/build-pipeline.md`
423. `docs/diagrams/agent-orchestration.md`
424. `docs/diagrams/feature-development.md`
425. `docs/diagrams/tdd-workflow.md`
426. `docs/diagrams/security-review.md`
427. `docs/diagrams/refactoring.md`
428. `docs/diagrams/cmd-plan.md`
429. `docs/diagrams/cmd-verify.md`
430. `docs/diagrams/cmd-build-fix.md`
431. `docs/diagrams/cmd-e2e.md`
432. `docs/diagrams/cmd-doc-suite.md`
433. `docs/diagrams/cmd-audit.md`
434. `docs/diagrams/cmd-backlog.md`
435. `docs/diagrams/cmd-uncle-bob-audit.md`

---

**Total artifacts: 435**

Diff this checklist against post-rebase `git ls-files` to verify nothing was lost.
