---
id: BL-091
title: "ECC diagnostics — tiered verbosity with tracing, ecc status command, zero model cost"
scope: HIGH
target: "/spec dev"
status: implemented
tags: [observability, diagnostics, tracing, cli, developer-experience]
created: 2026-03-28
related: [BL-053]
---

# BL-091: ECC Diagnostics — Tiered Verbosity + Status Command

## Problem

ECC hooks fire silently — users can't see what's happening under the hood. When things work, there's no confirmation. When things break, the error context is minimal. There's no way to see the current workflow phase, loaded hooks, installed components, or binary versions without manually inspecting files.

## Proposed Solution

### 1. Tiered Verbosity via `tracing` Crate

Replace ad-hoc stderr output with structured `tracing` instrumentation across all ecc-hook and ecc-workflow commands:

| Level | Flag | Env Var | What You See |
|-------|------|---------|-------------|
| Quiet | `-q` | `ECC_LOG=error` | Only blocking errors |
| Default | (none) | `ECC_LOG=warn` | Current behavior — warnings + blocks |
| Verbose | `-v` | `ECC_LOG=info` | Phase transitions, hook decisions, key events |
| Debug | `-vv` | `ECC_LOG=debug` | Config resolution, file paths, lock acquisition |
| Trace | `-vvv` | `ECC_LOG=trace` | Full hook stdin/stdout, JSON parsing |

Implementation: `tracing` + `tracing-subscriber` with `EnvFilter`. `ECC_LOG` env var follows `RUST_LOG` conventions (per-module filtering: `ECC_LOG=ecc_domain=trace`). CLI `-v` flags via `clap::ArgAction::Count`. All output to stderr — zero model cost.

### 2. `ecc status` Command (Extended)

Extend the existing `ecc` CLI with a broader `status` subcommand:

```
$ ecc status
ECC v4.2.0 | ecc-workflow v4.2.0
Phase: implement | Feature: BL-068
Started: 2026-03-28T15:00:00Z
Artifacts: spec ✓ design ✓ tasks ✓
Hooks: 24 installed (3 WorktreeCreate → legacy, will be cleaned on next install)
Components: 51 agents, 42 skills, 15 commands, 14 rules
Config: ~/.claude/settings.json (last merged: 2026-03-27)
```

Zero model cost — pure Rust reading files and counting entries.

### 3. Persistent Verbosity via `ecc config`

Set default verbosity level via CLI:
```
ecc config set log-level info    # Persist to ~/.ecc/config.toml
ecc config set log-level warn    # Reset to default
```

Reads from config file, overridable by `ECC_LOG` env var or `-v` flags. Precedence: CLI flag > env var > config file > default (warn).

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Statusline vs other display? | Not statusline — already packed. Use stderr + ecc status | User |
| 2 | When to show info? | Every hook event, not just session start | User |
| 3 | Always-on or opt-in? | Tiered: default=warn (current), -v=info, -vv=debug. Set via env/CLI/config | User + Research |
| 4 | Which Rust crate? | tracing + tracing-subscriber (modern standard, structured, async-aware) | Research |

## Dependencies

- None blocking. Enhances all existing hooks and commands.

## Impact

- **Zero model cost**: All output is Rust stderr, never enters Claude's context window
- **Debugging**: `-vv` gives full visibility into hook decisions, state reads, lock acquisition
- **Onboarding**: New users set `ECC_LOG=info` to understand what ECC does
- **Production**: Default `warn` maintains current silent behavior

## Ready-to-Paste Prompt

```
/spec dev

Add tiered diagnostics to ECC with zero model cost:

1. Integrate `tracing` + `tracing-subscriber` with `EnvFilter` into ecc-hook
   and ecc-workflow binaries. Levels: error/warn/info/debug/trace.
   CLI: -v/-vv/-vvv/-q flags. Env: ECC_LOG=<filter>.
   All output to stderr.

2. Add `tracing::info!` instrumentation to key hook handlers: phase-gate
   decisions, transition events, memory write results, session start/end.

3. Extend `ecc status` to show: binary versions, workflow phase/feature,
   artifacts status, installed component counts, hook count, config path.

4. Add `ecc config set log-level <level>` for persistent verbosity preference.
   Precedence: CLI flag > env var > config > default(warn).

See BL-091 for full analysis.
```
