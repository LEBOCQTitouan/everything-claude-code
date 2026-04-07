# Spec: BL-126 — 6 Token-Saving CLI Commands

## Problem Statement

The BL-121 token optimization audit identified 6 agent tasks that are purely mechanical (zero reasoning, deterministic output) yet consume LLM tokens: drift checking, module summary updates, doc coverage counting, diagram trigger detection, commit lint, and CLAUDE.md count validation. Each can be replaced with a compiled Rust CLI command that runs in milliseconds at zero token cost.

## Research Summary

- Web research skipped — internal CLI commands, no external dependencies needed.
- Existing patterns: `ecc validate`, `ecc analyze`, `ecc bypass` CLI groups serve as templates.
- Existing AC/PC parsers (`ecc-domain::spec::ac`, `ecc-domain::spec::pc`) can be reused for drift check.
- All I/O already abstracted behind port traits (FileSystem, ShellExecutor) — no new ports needed.
- Domain purity maintained: `ecc-domain` has zero I/O imports, all 6 commands follow this.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | 1 spec with 6 user stories | Shared structural pattern, avoids 6 adversary rounds | No |
| 2 | New CLI groups: ecc docs, ecc diagram, ecc commit | Namespace organization for related commands | No |
| 3 | Domain module: docs/ for US-002/003/006, drift/ for US-001 | Bounded context separation | No |
| 4 | All commands support --json flag for structured output | Agents consume structured output more reliably | No |
| 5 | Agent updates included per-US | Independent deliverability | No |
| 6 | Reuse existing AC/PC parsers for drift check | Avoid reimplementation | No |
| 7 | Keep enum variant counting (US-004) | Valuable heuristic, feasible with FileSystem port | No |

## User Stories

### US-001: ecc drift check
**As a** pipeline operator, **I want** `ecc drift check` to compute spec-vs-implementation drift, **so that** drift detection runs in milliseconds at zero token cost.

#### Acceptance Criteria
- AC-001.1: Given spec.md (or plan.md), solution.md, and implement-done.md, when ecc drift check runs, then AC/PC IDs are extracted and file change expectations from the design are loaded
- AC-001.2: Given extracted ACs and PCs, when set difference is computed, then unimplemented ACs are identified
- AC-001.3: Given analysis results, when drift level is classified, then output is NONE (0 unimplemented ACs, 0 unexpected files), LOW (<3 unexpected files, 0 unimplemented), MEDIUM (1-2 unimplemented ACs OR >3 unexpected files), HIGH (3+ unimplemented ACs)
- AC-001.4: Given classification, when output is written, then drift-report.md is produced at .claude/workflow/
- AC-001.5: Given --json flag, when run, then output is structured JSON
- AC-001.6: Given the CLI is built, when drift-checker agent is updated, then it calls ecc drift check
- AC-001.7: Given plan.md or implement-done.md is missing, when ecc drift check runs, then exit 1 with clear error message

#### Dependencies
- Depends on: none

### US-002: ecc docs update-module-summary
**As a** pipeline operator, **I want** `ecc docs update-module-summary` to locate markers and insert entries, **so that** module summary updates run without LLM.

#### Acceptance Criteria
- AC-002.1: Given --changed-files and --feature, when crate paths identified, then entries are formatted from template
- AC-002.2: Given MODULE-SUMMARIES.md with `<!-- IMPLEMENT-GENERATED -->` / `<!-- END IMPLEMENT-GENERATED -->` markers, when update runs, then entries inserted inside the marker block using format: `### <crate-name>\n<purpose>\n**Key Functions / Types:** ...\n**Spec Cross-Link:** ...\n**Design Rationale:** ...\n**Modified in:** ...`
- AC-002.3: Given existing entry for a crate, when update runs, then entry replaced (not duplicated)
- AC-002.4: Given --json flag, when run, then output is structured JSON
- AC-002.5: Given CLI built, when module-summary-updater agent updated, then it calls CLI for structural work
- AC-002.6: Given empty --changed-files list, when run, then output "No module changes" and exit 0

#### Dependencies
- Depends on: none

### US-003: ecc docs coverage
**As a** doc auditor, **I want** `ecc docs coverage --scope <path>` to count doc comments above pub items, **so that** coverage calculation runs without LLM.

#### Acceptance Criteria
- AC-003.1: Given Rust source directory, when coverage runs, then /// doc comments above pub items are detected
- AC-003.2: Given source files walked, when coverage computed, then per-module coverage percentages produced
- AC-003.3: Given --json flag, when run, then output is structured JSON
- AC-003.4: Given CLI built, when doc-reporter agent updated, then Step 1 calls CLI
- AC-003.5: Given --scope path does not exist, when run, then exit 1 with error message

#### Dependencies
- Depends on: none

### US-004: ecc diagram triggers
**As a** pipeline operator, **I want** `ecc diagram triggers --changed-files <list>` to evaluate heuristics, **so that** trigger detection runs deterministically.

#### Acceptance Criteria
- AC-004.1: Given files spanning 2+ crate dirs, when heuristics evaluated, then "sequence" trigger emitted
- AC-004.2: Given file with enum >=3 variants, when heuristics evaluated, then "flowchart" trigger emitted
- AC-004.3: Given new crate directory, when heuristics evaluated, then "c4" trigger emitted
- AC-004.4: Given no heuristics fire, when run, then output is {"triggers":[]}
- AC-004.5: Given CLI built, when diagram-updater agent updated, then it calls CLI for trigger detection
- AC-004.6: Given --json flag, when run, then output is {"triggers": ["sequence"|"flowchart"|"c4"]}

#### Dependencies
- Depends on: none

### US-005: ecc commit lint --staged
**As a** developer, **I want** `ecc commit lint --staged` to detect multi-concern staged changes, **so that** atomic commit validation runs without LLM.

#### Acceptance Criteria
- AC-005.1: Given --staged flag, when run, then staged files obtained via git diff --cached --name-only
- AC-005.2: Given files spanning >1 top-level dir, when lint runs, then exit 2 with warning
- AC-005.3: Given files mixing src + docs, when lint runs, then exit 2 with warning
- AC-005.4: Given single-concern files, when lint runs, then exit 0
- AC-005.5: Given CLI built, when commit.md updated, then Phase 3 (concern detection) calls ecc commit lint
- AC-005.6: Given --json flag, when run, then output is {"concerns": [...], "verdict": "pass"|"warn"}
- AC-005.7: Given no staged files, when run, then exit 0 with no output

#### Dependencies
- Depends on: none

### US-006: ecc validate claude-md --counts
**As a** doc maintainer, **I want** `ecc validate claude-md --counts` to cross-check numeric claims, **so that** count drift detection runs without LLM.

#### Acceptance Criteria
- AC-006.1: Given CLAUDE.md claim "997 tests", when validation runs, then actual count obtained and compared
- AC-006.2: Given claim "9 crates", when run, then workspace crate count verified
- AC-006.3: Given mismatches, when run, then exit 1 with mismatch report
- AC-006.4: Given all match, when run, then exit 0 with "All counts valid"
- AC-006.5: Given --json flag, when run, then output is structured JSON
- AC-006.6: Given CLI built, when doc-validator agent updated, then count drift calls CLI
- AC-006.7: Given CLAUDE.md does not exist, when run, then exit 1 with error
- AC-006.8: Given cargo test not available, when run, then skip test count and warn

#### Dependencies
- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| crates/ecc-domain/src/drift/ | Domain | New: AC/PC cross-ref, scope detection, drift level |
| crates/ecc-domain/src/docs/ | Domain | New: module_summary, coverage, diagram_triggers, claude_md |
| crates/ecc-domain/src/analyze/commit_lint.rs | Domain | New: staged file concern detection |
| crates/ecc-app/src/ | App | 6 new use case modules |
| crates/ecc-cli/src/commands/ | CLI | 3 new subcommand groups + 1 validate variant |
| agents/*.md, commands/*.md | Content | 6 agent/command updates |

## Constraints

- ecc-domain has zero I/O imports (purity enforced)
- No new port traits needed (FileSystem + ShellExecutor sufficient)
- Agents updated, not deleted
- All commands support --json flag for structured output
- Reuse existing AC/PC parsers from ecc-domain::spec
- Exit codes: 0 = success, 1 = error/mismatch found, 2 = lint warning (commit lint only)
- JSON schemas (top-level fields): drift={level,unimplemented_acs,unexpected_files,missing_files}, docs-coverage={modules:[{name,total,documented,pct}]}, claude-md={claims:[{text,claimed,actual,match}]}, diagram-triggers={triggers:[string]}, commit-lint={concerns:[string],verdict:string}, docs-update={updated_crates:[string]}

## Non-Requirements

- Full agent replacement (agents retain LLM judgment roles)
- Custom output templates
- Interactive mode
- ADR (standard CLI extension)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem | Uses existing | Read source files, MODULE-SUMMARIES, CLAUDE.md |
| ShellExecutor | Uses existing | Run git diff, cargo test --list |
| TerminalIO | Uses existing | Output results |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New commands | CLAUDE.md | CLAUDE.md | Add 6 new CLI commands |
| Changelog | CHANGELOG | CHANGELOG.md | Add BL-126 entry |

## Open Questions

None.

## Doc Preview

### CLAUDE.md changes
Add to CLI Commands:
```
ecc drift check [--json]         Compute spec-vs-implementation drift
ecc docs update-module-summary   Update MODULE-SUMMARIES.md entries
ecc docs coverage --scope <path> Doc comment coverage per module
ecc diagram triggers             Evaluate diagram generation heuristics
ecc commit lint --staged         Validate atomic commit concerns
ecc validate claude-md --counts  Cross-check CLAUDE.md numeric claims
```
