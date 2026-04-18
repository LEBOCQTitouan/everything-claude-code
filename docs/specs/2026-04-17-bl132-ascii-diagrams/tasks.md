# Tasks: BL-132 ASCII Diagram Sweep

Source: docs/specs/2026-04-17-bl132-ascii-diagrams/design.md (prior + Revision 2026-04-18 + Supplement + Supplement v2)
Concern: dev | Feature: BL-132 ASCII diagram sweep across 9 ECC crates

## TDD Status Trail

Status format: `pending` | `red@<ts>` | `green@<ts>` | `done@<ts>` | `failed@<ts>` | `eval:<verdict>@<ts>`

### Step A — Scaffolding (Session 1)

- [ ] PC-032 — Classifier scripts exist + executable: pending
- [ ] PC-033-v2 — Fixture corpus exact match (8 files): pending
- [ ] PC-038 — Tool prerequisites (gawk + rg --pcre2): pending

### Step B — CI amendment (Session 1)

- [ ] PC-022 — CI workflow wires PC-021: pending
- [ ] PC-035 — CI workflow wires --help smoke: pending

### Step C — ADR (Session 1)

- [ ] PC-034 — ADR-0067 present with required sections: pending

### Gate — Step A+B+C PCs all green

### Step D Wave 1a — US-001 ecc-domain

- [ ] PC-001-v3 — Phase enum state diagram: pending
- [ ] PC-002 — ecc-domain cargo doc: pending
- [ ] PC-017 — Priority targets present: pending
- [ ] PC-024 — Crate floor ≥20 text fences: pending

### Step D Wave 1b — US-002 ecc-workflow

- [ ] PC-003 — transition flow diagram: pending
- [ ] PC-004 — ecc-workflow cargo doc: pending
- [ ] PC-025 — Crate floor ≥5 text fences: pending

### Step D Wave 2 — US-003 + US-004 + US-005

- [ ] PC-005 — ecc-ports cargo doc: pending
- [ ] PC-026 — ecc-ports ≥8 # Pattern: pending
- [ ] PC-006-v3 — dispatch flow diagram: pending
- [ ] PC-007 — ecc-app cargo doc: pending
- [ ] PC-027 — ecc-app ≥10 text fences: pending
- [ ] PC-008 — ecc-infra cargo doc: pending
- [ ] PC-028 — ecc-infra ≥8 # Pattern: pending

### Step D Wave 3 — US-006 + US-007

- [ ] PC-009 — ecc-cli cargo doc: pending
- [ ] PC-029 — ecc-cli ≥2 diagrams at //!/impl: pending
- [ ] PC-010 — FlockGuard RAII annotation: pending
- [ ] PC-011 — ecc-flock cargo doc: pending
- [ ] PC-030 — ecc-flock 1 diagram + 1 Pattern: pending

### Step E — Final verification

- [ ] PC-012 — Workspace cargo doc: pending
- [ ] PC-013 — Workspace clippy: pending
- [ ] PC-014 — Workspace build: pending
- [ ] PC-015-v3 — Clap-derive deny-list: pending
- [ ] PC-016-v3 — Drift-anchor presence: pending
- [ ] PC-018-v3 — Unicode box-drawing ban: pending
- [ ] PC-019-v3 — Fence language tag required: pending
- [ ] PC-020 — Cap-override on diagrams >20 lines: pending
- [ ] PC-021 — --help smoke test: pending
- [ ] PC-023 — Attribute-form bypass ban: pending
- [ ] PC-031-v2 — Minimum shippable subset (US-001 + US-002): pending
- [ ] PC-037 — CHANGELOG has BL-132-sweep-shipped sentinel: pending

### Post-TDD

- [ ] E2E tests: pending
- [ ] Code review: pending
- [ ] Doc updates: pending
- [ ] Supplemental docs: pending
- [ ] Write implement-done.md: pending
