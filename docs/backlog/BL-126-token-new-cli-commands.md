---
id: BL-126
title: "Token optimization wave 3 — new CLI commands replacing agent work"
status: open
created: 2026-04-06
promoted_to: ""
tags: [token-optimization, rust-cli, agents, cost]
scope: HIGH
target_command: /spec-dev
dependencies: [BL-121, BL-124]
---

## Optimized Prompt

Implement 6 new Rust CLI commands from the BL-121 audit (Wave 3). Each replaces a purely mechanical agent task (zero reasoning) with a compiled binary:

1. `ecc drift check` — parse plan.md + implement-done.md, extract AC/PC IDs via regex, compute set difference, classify drift level (NONE/LOW/MEDIUM/HIGH by % coverage), write drift-report.md. Replaces `agents/drift-checker.md`.
2. `ecc docs update-module-summary --changed-files <list> --feature <name> --spec-ref <id>` — locate HTML markers in MODULE-SUMMARIES.md, insert/update structured entry from template. Replaces `agents/module-summary-updater.md`.
3. `ecc docs coverage --scope <path>` — walk source files, count doc comments (`///` for Rust, `/** */` for JS/TS, `"""` for Python) above `pub` items, output per-module coverage table. Replaces doc-reporter Step 1.
4. `ecc diagram triggers --changed-files <list>` — evaluate 3 heuristics (cross-module file span, enum variant count >5, new crate directory), output JSON `{triggers: ["sequence"|"flowchart"|"c4"]}`. Replaces diagram-updater trigger detection.
5. `ecc commit lint --staged` — `git diff --cached --name-only`, flag files spanning >1 top-level directory or mixing src+docs+agents, exit 2 with warning message. Replaces commit.md atomic concern detection.
6. `ecc validate claude-md --counts` — regex-match numeric claims in CLAUDE.md (e.g., "2449 tests", "9 crates"), cross-check via `cargo test -- --list | wc -l`, `find crates/ -maxdepth 1 -type d | wc -l`, etc. Report mismatches. Replaces doc-validator count drift.

After building each command, update the corresponding agent/command markdown to call the CLI and interpret output rather than reimplementing.

**Hexagonal placement:** All 6 commands live in `ecc-cli` crate, with domain logic in `ecc-domain` (pure, no I/O) and file I/O behind port traits.

Reference: `docs/audits/token-optimization-2026-04-06.md` findings 1.1, 1.2, 1.6, 1.7, 1.10, 1.11.

## Original Input

BL-121 audit Wave 3: 6 new CLI commands (drift check, module-summary update, doc coverage, diagram triggers, commit lint, CLAUDE.md count validation).

## Challenge Log

**Source:** BL-121 token optimization audit (2026-04-06). Pre-challenged during audit — each finding validated as zero-reasoning, purely deterministic work.
