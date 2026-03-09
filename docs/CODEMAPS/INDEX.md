<!-- Generated: 2026-03-09 | Files scanned: 48 src + 27 tests | Token estimate: ~350 -->

# Codemap Index — Everything Claude Code (ECC)

## Maps

| File | Description |
|------|-------------|
| [architecture.md](architecture.md) | System diagram, data flow, build pipeline, boundaries |
| [backend.md](backend.md) | Library modules, hooks, install pipeline, CI validators |
| [data.md](data.md) | Data structures, storage format, configuration files |
| [dependencies.md](dependencies.md) | Runtime/dev deps, external tools, npm distribution |

## Quick Stats

- **Source:** 48 TypeScript files in `src/` (~5,200 LOC)
- **Tests:** 27 test files, 1272 assertions passing (single-process runner)
- **Content:** 25 agents, 47 commands, 70 skills, 5 rule groups
- **Runtime dep:** 1 (`omelette`)
- **Build:** `tsc` → CommonJS in `dist/`

## Entry Points

```
bin/ecc.js           → CLI entry (shell completion)
install.sh           → Bash orchestrator (install/init)
src/postinstall.ts   → npm postinstall health checks
src/install-orchestrator.ts → Node.js install pipeline
```

## Agent Ecosystem

```
Orchestrators:  doc-orchestrator, arch-reviewer, loop-operator
Reviewers:      code-reviewer, python-reviewer, go-reviewer, security-reviewer, database-reviewer, uncle-bob
Architects:     architect, architect-module
Builders:       build-error-resolver, go-build-resolver, tdd-guide, e2e-runner
Doc system:     doc-analyzer, doc-generator, doc-validator, doc-reporter, diagram-generator
Utilities:      planner, refactor-cleaner, harness-optimizer, doc-updater, chief-of-staff
```
