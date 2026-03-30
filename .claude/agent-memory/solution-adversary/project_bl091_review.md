---
name: BL-091 Design Review
description: Adversary review of diagnostics/tracing solution design — CONDITIONAL verdict with 3 blocking issues
type: project
---

BL-091 diagnostics/tracing design scored CONDITIONAL (72/100) on 2026-03-29.

Three blocking issues:
1. No rollback plan for a 26-file, 5-crate migration
2. Domain `from_toml`/`to_toml` methods contradict the dep graph (toml crate added to ecc-app, not ecc-domain)
3. PC-026/027/028 assume ECC_LOG env var activates tracing in unit tests, but tests don't run main()

**Why:** These are addressable without redesign — the solution structure is sound.
**How to apply:** If the design is revised, verify these three items are resolved before approving.
