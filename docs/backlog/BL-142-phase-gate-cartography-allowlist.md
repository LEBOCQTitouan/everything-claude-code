---
id: BL-142
title: "Add docs/cartography/ to phase-gate allowlist"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-04-09"
source: "session finding — cartography processing blocked during active workflow phases"
tags: [phase-gate, cartography, fix]
---

## Context

The phase-gate hook (`ecc-workflow phase-gate`) blocks Write/Edit to paths not in the `allowed_prefixes` list during spec/design phases. `docs/cartography/` is missing from this list, so cartography delta processing fails when any workflow is active. Other `docs/` paths (specs, audits, backlog, adr, plans) are already allowed.

## Prompt

Add `"docs/cartography/"` to the `allowed_prefixes` function in `crates/ecc-workflow/src/commands/phase_gate.rs` (alongside the existing `docs/specs/`, `docs/audits/`, etc. entries). Rebuild `ecc-workflow`. Verify cartography writes succeed during plan/spec phases.

## Acceptance Criteria

- [ ] `docs/cartography/` added to `allowed_prefixes` in phase_gate.rs
- [ ] cargo test -p ecc-workflow -- phase_gate passes
- [ ] cargo clippy passes
- [ ] Cartography writes succeed during plan phase (manual test)
