<!-- Generated: 2026-03-09 | Files scanned: 48 src + 27 tests | Token estimate: ~950 -->

# Architecture Overview

## System Type

CLI tool (`@lebocqtitouan/ecc`) — global npm package providing Claude Code configuration management.

## Data Flow

```
User CLI
  │
  ├─ bin/ecc.js (entry, shell completion via omelette)
  │    └─ install.sh (bash orchestrator)
  │         ├─ cmd_install → install-orchestrator.ts
  │         │    ├─ detect.ts     → scan existing setup
  │         │    ├─ manifest.ts   → track ECC artifacts
  │         │    ├─ merge.ts      → interactive diff review + conflict resolution
  │         │    ├─ smart-merge.ts → LCS diff + Claude merge + contentsDiffer
  │         │    └─ gitignore.ts  → auto-manage .gitignore
  │         └─ cmd_init → project-level CLAUDE.md + hooks
  │
  ├─ hooks/ (hooks.json registry)
  │    └─ src/hooks/*.ts (23 hook implementations)
  │         └─ run-with-flags.ts (profile-gated execution)
  │
  ├─ Doc system agents (6-agent pipeline)
  │    └─ doc-orchestrator → doc-analyzer → doc-generator → doc-validator → doc-reporter
  │                                       → diagram-generator (reads CUSTOM.md registry)
  │
  └─ Content directories (copied to ~/.claude/)
       ├─ agents/    (25 specialized agents)
       ├─ commands/  (47 slash commands)
       ├─ skills/    (70 skill directories)
       ├─ rules/     (common + language-specific)
       └─ contexts/  (3 context files)
```

## Key Boundaries

| Boundary | Description |
|----------|-------------|
| CLI → Bash | `bin/ecc.js` dispatches to `install.sh` for install/init |
| Bash → Node | `install.sh` delegates to `dist/install-orchestrator.js` |
| Hooks → Runtime | `hooks.json` maps events → `dist/hooks/*.js` scripts |
| Config → User | Files are merged into `~/.claude/` with interactive diff review |
| Doc Suite → Agents | `/doc-suite` orchestrates 5 specialized doc agents in parallel |
| Custom Registry → Diagrams | `docs/diagrams/CUSTOM.md` declares diagrams for regeneration |

## Build Pipeline

```
src/**/*.ts  →  tsc (tsconfig.build.json)  →  dist/**/*.js (CommonJS)
                                                  │
                                           npm publish (files: bin/, dist/, agents/, ...)
                                                  │
                                           npm install -g → postinstall.ts health checks
```

## Test Architecture

```
tests/harness.js       → shared test()/describe()/getResults() harness
tests/run-all.js       → single-process runner (require, no subprocess per file)
tests/**/*.test.js     → 27 test files exporting runTests() (1272 assertions)
                         env snapshot/restore + require.cache cleanup between files
```

## Runtime Dependencies

- `omelette` — shell tab-completion
- Node.js >=18
- bash (for install.sh orchestration)
- Optional: `claude` CLI (for smart-merge)
