---
id: BL-113
title: "Upgrade rusqlite 0.34 to 0.38"
scope: LOW
target: "direct edit"
status: open
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: trial
tags: [deps, upgrade, database]
---

## Context

rusqlite has progressed from 0.34.0 to 0.38.0 (4 minor versions behind). Breaking change in 0.38.0: Connection ownership check when registering closures as hooks. Brings SQLite 3.51.3 support and accumulated bug fixes.

## Prompt

Upgrade `rusqlite` from `0.34` to `0.38` in `Cargo.toml` workspace dependencies. Address the breaking change in 0.38.0 (Connection ownership check for hook registration). Run full test suite to verify no regressions. This is a LOW scope change — dependency version bump with one known breaking change.

## Acceptance Criteria

- [ ] rusqlite version bumped to 0.38 in workspace Cargo.toml
- [ ] Breaking change (Connection ownership check) addressed
- [ ] All tests pass
- [ ] cargo clippy passes
