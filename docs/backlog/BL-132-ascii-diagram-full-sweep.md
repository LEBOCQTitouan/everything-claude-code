---
id: BL-132
title: Full ASCII diagram sweep of all 9 ECC crates
tier: 5
scope: HIGH
target: direct edit
status: open
created: 2026-04-08
file: crates/
---

## Action

Run the ascii-doc-diagrams convention (skill) across all 9 ECC Rust crates. Add ASCII state-transition diagrams, flow/decision diagrams, composition diagrams, and pattern annotations to all eligible public items per the skill's eligibility criteria. Priority targets: workflow FSM (transition.rs), task FSM (status.rs), state_resolver chain, hook dispatch, port traits. Estimated scope: ~50-100 files across ecc-domain, ecc-ports, ecc-app, ecc-infra, ecc-cli, ecc-workflow, ecc-flock, ecc-test-support, ecc-integration-tests.
