---
id: BL-115
title: "Upgrade toml 0.8 to 0.9"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: trial
tags: [deps, upgrade]
---

## Context

toml 0.9 released July 2025 with shared AST and improved test alignment between toml and toml_edit. Low-risk upgrade with parser improvements.

## Prompt

Upgrade `toml` from `0.8` to `0.9` in `Cargo.toml` workspace dependencies. This is a low-risk bump — verify all TOML parsing code still works. Run full test suite.

## Acceptance Criteria

- [ ] toml version bumped to 0.9 in workspace Cargo.toml
- [ ] All tests pass
- [ ] cargo clippy passes
