---
id: BL-114
title: "Upgrade rustyline 15 to 17"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: trial
tags: [deps, upgrade, cli]
---

## Context

rustyline has jumped from v15 to v17.0.2 (2 major versions behind). Used by the NanoClaw interactive REPL (`ecc claw`). Check release notes for breaking API changes.

## Prompt

Upgrade `rustyline` from `15` to `17` in `Cargo.toml` workspace dependencies. Review release notes for v16 and v17 breaking changes. Update any affected code in the NanoClaw REPL. Run full test suite to verify no regressions.

## Acceptance Criteria

- [ ] rustyline version bumped to 17 in workspace Cargo.toml
- [ ] Breaking API changes addressed
- [ ] NanoClaw REPL still functions correctly
- [ ] All tests pass
