<!-- Generated: 2026-03-08 | Files scanned: 47 src + 23 tests | Token estimate: ~300 -->

# Codemap Index — Everything Claude Code (ECC)

## Maps

| File | Description |
|------|-------------|
| [architecture.md](architecture.md) | System diagram, data flow, build pipeline, boundaries |
| [backend.md](backend.md) | Library modules, hooks, install pipeline, CI validators |
| [data.md](data.md) | Data structures, storage format, configuration files |
| [dependencies.md](dependencies.md) | Runtime/dev deps, external tools, npm distribution |

## Quick Stats

- **Source:** 47 TypeScript files in `src/` (~5,000 LOC)
- **Tests:** 23 test files, 1089 assertions passing
- **Content:** 18 agents, 40+ commands, 71 skills, 5 rule groups
- **Runtime dep:** 1 (`omelette`)
- **Build:** `tsc` → CommonJS in `dist/`

## Entry Points

```
bin/ecc.js           → CLI entry (shell completion)
install.sh           → Bash orchestrator (install/init)
src/postinstall.ts   → npm postinstall health checks
src/install-orchestrator.ts → Node.js install pipeline
```
