<!-- Generated: 2026-03-14 | Files scanned: 50 src + 32 tests | Token estimate: ~400 -->

# Codemap Index — Everything Claude Code (ECC)

## Maps

| File | Description |
|------|-------------|
| [architecture.md](architecture.md) | System diagram, data flow, build pipeline, boundaries |
| [backend.md](backend.md) | Library modules, hooks, install pipeline, CI validators |
| [data.md](data.md) | Data structures, storage format, configuration files |
| [dependencies.md](dependencies.md) | Runtime/dev deps, external tools, npm distribution |

## Quick Stats

- **Source:** 50 TypeScript files in `src/` (~5,800 LOC)
- **Tests:** 32 test files, 1401 assertions passing (single-process runner)
- **Content:** 30 agents, 6 commands (+ 41 archived), 67 skills, 5 rule groups
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
Orchestrators:  doc-orchestrator, arch-reviewer, audit-orchestrator
Reviewers:      code-reviewer, python-reviewer, go-reviewer, security-reviewer, database-reviewer, uncle-bob
Architects:     architect, architect-module
Builders:       build-error-resolver, go-build-resolver, tdd-guide, e2e-runner
Doc system:     doc-analyzer, doc-generator, doc-validator, doc-reporter, diagram-generator
Audit system:   evolution-analyst, test-auditor, observability-auditor, error-handling-auditor, convention-auditor
Utilities:      planner, requirements-analyst, refactor-cleaner, harness-optimizer, doc-updater
```
