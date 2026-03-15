<!-- Generated: 2026-03-15 | Crates: 6 | Files: 109 .rs -->

# Backend -- Crate Modules & Hook Handlers

## ecc-domain (pure business logic)

```
ansi.rs
  └─ ANSI color functions: bold, dim, red, green, yellow, cyan, white, magenta, gray, bg_cyan
  └─ strip_ansi() — remove escape codes
  └─ Respects NO_COLOR (checked at call site)

config/
  ├─ audit.rs — Severity, AuditFinding, AuditCheckResult, AuditReport
  │   └─ compute_audit_score(), is_ecc_managed_hook(), parse_frontmatter()
  ├─ clean.rs — CleanReport, format_clean_report()
  ├─ deny_rules.rs — DenyRule, ECC_DENY_RULES, ensure_deny_rules()
  ├─ detect.rs — scan existing Claude config directory
  ├─ gitignore.rs — ECC_GITIGNORE_ENTRIES, gitignore pattern management
  ├─ manifest.rs — EccManifest, Artifacts, create/update/diff manifests
  ├─ merge.rs — MergeReport, FileToReview, merge_hooks_pure(), remove_legacy_hooks()
  └─ validate.rs — frontmatter extraction, hook validation

detection/
  ├─ language.rs — Language enum, detection rules
  ├─ framework.rs — Framework enum, detection rules
  └─ package_manager.rs — PackageManager enum, lock file detection

diff/
  ├─ lcs.rs — LCS algorithm for line-level diffing
  └─ formatter.rs — unified diff formatting

session/
  ├─ aliases.rs — AliasesData, validate/resolve/list/cleanup aliases
  └─ manager.rs — session CRUD, metadata parsing

claw/
  ├─ command.rs — ClawCommand enum, parse_command()
  ├─ model.rs — ClawModel enum (Sonnet, Opus, Haiku)
  ├─ prompt.rs — system prompt construction
  ├─ turn.rs — Turn struct, parse_turns()
  ├─ metrics.rs — session metrics calculation
  ├─ search.rs — cross-session keyword search
  ├─ compact.rs — session compaction
  ├─ export.rs — session export (md/json/txt)
  └─ session_name.rs — session name validation

hook_runtime/
  └─ profiles.rs — profile parsing, is_hook_enabled()

paths.rs — path constants (claude_dir, claw_dir, session paths)
time.rs — timestamp formatting
```

## ecc-ports (trait definitions)

```
fs.rs        → FileSystem trait + FsError enum
shell.rs     → ShellExecutor trait + CommandOutput + ShellError
env.rs       → Environment trait + Platform enum
terminal.rs  → TerminalIO trait + TerminalError
stdin.rs     → StdinReader trait + StdinError
repl.rs      → ReplInput trait
```

## ecc-app (use cases)

```
install/
  ├─ mod.rs — InstallOptions, InstallSummary, install_global() (8-step flow)
  └─ helpers.rs — artifact collection, deny rules in settings, summary printing

merge/
  ├─ mod.rs — ReviewChoice, MergeOptions, merge_directory/skills/rules/hooks
  ├─ prompt.rs — interactive file review with diff display
  └─ helpers.rs — JSON read, recursive copy

audit.rs — AuditOptions, run_audit(), format_finding()
validate.rs — ValidateTarget, run_validate() (agents/commands/hooks/skills/rules/paths)
smart_merge.rs — SmartMergeResult, smart_merge() via claude -p
act_ci.rs — CheckStatus, Check, ValidationReport, validate_all()

hook/
  ├─ mod.rs — HookContext, HookResult, HookPorts, dispatch()
  └─ handlers/ — 20+ hook handler implementations
       ├─ Tier 1: passthrough hooks (check-console-log, git-push-reminder, etc.)
       ├─ Tier 2: external tool hooks (format, typecheck, quality-gate)
       └─ Tier 3: session I/O hooks (session-start, session-end, cost-tracker)

claw/
  ├─ mod.rs — ClawConfig, ClawState, run_repl()
  ├─ dispatch.rs — command dispatch
  ├─ handlers.rs — /help, /model, /metrics, /search, etc.
  ├─ claude_runner.rs — claude -p invocation
  ├─ skill_loader.rs — skill file loading
  └─ storage.rs — session persistence

detection/
  ├─ language.rs — filesystem-driven language detection
  ├─ framework.rs — filesystem-driven framework detection
  └─ package_manager.rs — lock file based PM detection

config/
  ├─ audit.rs — run_all_checks() orchestration
  ├─ clean.rs — clean_all(), clean_from_manifest()
  ├─ gitignore.rs — ensure_gitignore_entries(), find_tracked_ecc_files()
  ├─ manifest.rs — read_manifest(), write_manifest()
  └─ merge.rs — pre_scan_directory()

detect.rs — detect_and_report(), is_empty_setup()
session/ — session use cases (alias I/O, session manager)
version.rs — version string
```

## ecc-infra (production adapters)

```
os_fs.rs          → OsFileSystem (std::fs + walkdir)
process_executor.rs → ProcessExecutor (std::process::Command)
os_env.rs         → OsEnvironment (std::env)
std_terminal.rs   → StdTerminal (crossterm)
std_stdin.rs      → StdStdinReader (std::io::stdin)
rustyline_input.rs → RustylineInput (rustyline)
```

## ecc-test-support (test doubles)

```
in_memory_fs.rs      → InMemoryFileSystem (HashMap-backed, builder API)
mock_executor.rs     → MockExecutor (scripted command responses)
mock_env.rs          → MockEnvironment (configurable vars/home/platform)
buffered_terminal.rs → BufferedTerminal (captures output for assertions)
scripted_input.rs    → ScriptedInput (pre-loaded REPL lines)
```

## ecc-cli (binary)

```
main.rs → clap App definition, subcommand matching, port construction, delegation
```

## Agent Ecosystem (30 agents, Markdown with YAML frontmatter)

```
Orchestrators:  doc-orchestrator, arch-reviewer, audit-orchestrator
Reviewers:      code-reviewer, python-reviewer, go-reviewer, security-reviewer, database-reviewer, uncle-bob
Architects:     architect, architect-module
Builders:       build-error-resolver, go-build-resolver, tdd-guide, e2e-runner
Doc system:     doc-analyzer, doc-generator, doc-validator, doc-reporter, diagram-generator
Audit system:   evolution-analyst, test-auditor, observability-auditor, error-handling-auditor, convention-auditor
Utilities:      planner, requirements-analyst, refactor-cleaner, harness-optimizer, doc-updater
```
