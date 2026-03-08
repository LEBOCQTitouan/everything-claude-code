<!-- Generated: 2026-03-08 | Files scanned: 47 src + 23 tests | Token estimate: ~900 -->

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
  │         │    ├─ merge.ts      → conflict resolution
  │         │    ├─ smart-merge.ts → LCS diff + Claude merge
  │         │    └─ gitignore.ts  → auto-manage .gitignore
  │         └─ cmd_init → project-level CLAUDE.md + hooks
  │
  ├─ hooks/ (hooks.json registry)
  │    └─ src/hooks/*.ts (28 hook implementations)
  │         └─ run-with-flags.ts (profile-gated execution)
  │
  └─ Content directories (copied to ~/.claude/)
       ├─ agents/    (18 specialized agents)
       ├─ commands/  (40+ slash commands)
       ├─ skills/    (71 skill directories)
       ├─ rules/     (common + language-specific)
       └─ contexts/  (3 context files)
```

## Key Boundaries

| Boundary | Description |
|----------|-------------|
| CLI → Bash | `bin/ecc.js` dispatches to `install.sh` for install/init |
| Bash → Node | `install.sh` delegates to `dist/install-orchestrator.js` |
| Hooks → Runtime | `hooks.json` maps events → `dist/hooks/*.js` scripts |
| Config → User | Files are merged into `~/.claude/` respecting user customizations |

## Build Pipeline

```
src/**/*.ts  →  tsc (tsconfig.build.json)  →  dist/**/*.js (CommonJS)
                                                  │
                                           npm publish (files: bin/, dist/, agents/, ...)
                                                  │
                                           npm install -g → postinstall.ts health checks
```

## Runtime Dependencies

- `omelette` — shell tab-completion
- Node.js >=18
- bash (for install.sh orchestration)
- Optional: `claude` CLI (for smart-merge)
