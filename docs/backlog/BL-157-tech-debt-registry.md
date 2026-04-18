---
id: BL-157
title: Tech Debt Registry linked to Backlog
status: open
scope: HIGH
target_command: /spec
created: 2026-04-18
addresses_debt: []
---

## Problem

The ECC pipeline's adversarial reviewers (spec-adversary, solution-adversary, design-reviewer, uncle-bob, robert, pc-evaluator) regularly flag design shortcuts, SOLID violations, missing tests, doc gaps, and architectural concessions. Today these findings are either fixed ad-hoc or forgotten entirely. There is no persistent registry to track what debt has been accepted, what pays it down, and what remains open. Findings surface once and evaporate — there is no way to know the accumulated debt load, prioritize paydown, or link a backlog entry to the debt it retires.

## Solution Sketch

New `docs/debt/` registry mirroring `docs/backlog/` conventions, bidirectionally linked with it. Hybrid capture mechanism: adversary/reviewer agents emit structured debt findings as part of their normal output; a phase-end consolidation step in `/spec`, `/design`, `/implement` collects those findings and writes `DEBT-NNN-<slug>.md` entries. Full taxonomy combining Cunningham + Fowler + Kruchten + industry (14 debt types, Fowler quadrant, severity, principal, interest rate). Four surfacing channels: `/debt-review` command, audit integration, `/spec` Phase 0 cross-reference, and `ecc debt status` dashboard.

## Acceptance Criteria

- `docs/debt/DEBT.md` index exists with the same shape as `BACKLOG.md`
- `DEBT-NNN-<slug>.md` per-entry files follow the full schema (id, title, status, created, debt_type, quadrant, source_phase, source_agent, originating_spec, severity, component, remediation_hint, addressed_by, principal, interest_rate)
- All 14 debt_type values are validated by `ecc validate debt`
- All 4 Fowler quadrant values are validated
- `ecc debt list [--type X] [--severity Y] [--status Z]` works
- `ecc debt add` (manual capture) works
- `ecc debt update-status DEBT-NNN <status>` updates the file and reindexes DEBT.md
- `ecc debt reindex` rebuilds DEBT.md from entry files
- `ecc debt next-id` returns the next sequential DEBT-NNN
- `ecc debt status` prints a dashboard (count by type, severity distribution, oldest unpaid, total estimated principal)
- Backlog entry schema gains `addresses_debt: [DEBT-NNN, ...]` field (empty = new implementation, non-empty = debt paydown)
- `ecc backlog list --addresses-debt` filter works
- Bidirectional sync: promoting/implementing a BL entry updates its linked DEBT entries to `addressed`
- Every adversary agent output in `/spec`, `/design`, `/implement` can produce structured debt findings (new section or JSON block)
- Phase-end consolidation step exists in all three pipeline commands and writes DEBT-NNN entries
- Deduplication logic prevents duplicate entries when the same finding appears across phases
- `/debt-review` command walks open entries, prioritizes by `interest_rate × time_carried / principal`, and suggests promoting to backlog
- `/audit-full` and domain audits cross-reference `docs/debt/` and flag stale debt
- `/spec` Phase 0 checks whether the incoming spec addresses existing debt and suggests linking via `addresses_debt:`
- One-time migration scan of `docs/audits/` and `docs/specs/*/adversary-*.md` seeds initial DEBT-NNN entries

## Scope & Non-Goals

**In scope:**
- `docs/debt/` registry (DEBT.md index + DEBT-NNN-<slug>.md entries)
- CLI surface under `ecc debt` mirroring `ecc backlog`
- Schema validation via `ecc validate debt`
- Adversary agent output extensions (structured debt findings section)
- Phase-end consolidation in `/spec`, `/design`, `/implement`
- Deduplication by component + debt_type + finding fingerprint
- Bidirectional linkage with `docs/backlog/` (addresses_debt field)
- Four surfacing channels (debt-review command, audit integration, spec Phase 0, dashboard)
- One-time migration / bootstrap from existing audit and adversary artifacts

**Out of scope:**
- Automated debt paydown (user must explicitly `/spec` from a debt entry)
- Auto-generated remediation specs
- Debt import from external tools (SonarQube, CAST, etc.)

## Open Questions for /design

- New crate `ecc-debt` vs extend `ecc-backlog` crate (shared index-file plumbing argues for extension; distinct domain model argues for new crate)
- Phase-end consolidation: hook-based (automatic) vs command-based (explicit) vs both
- Finding fingerprint algorithm for deduplication across phases (hash of component + debt_type + normalized finding text?)
- Schema validation approach (`ecc validate debt`) — extend existing validate dispatcher or new target?
- Migration scope: how much of `docs/audits/` and adversary output is machine-parseable vs requires manual triage?

## Optimized Prompt

```
/spec Add a persistent tech-debt registry at `docs/debt/` that captures technical and architectural debt surfaced by the `/spec` → `/design` → `/implement` pipeline and links it bidirectionally with the existing `docs/backlog/`.

**Problem**: Adversarial reviewers (spec-adversary, solution-adversary, design-reviewer, uncle-bob, robert, pc-evaluator) regularly flag design shortcuts, SOLID violations, missing tests, doc gaps, and architectural concessions. Today these findings get fixed ad-hoc or forgotten. There is no persistent registry to track what debt has been accepted, what pays it down, and what remains open.

**Goal**: A `docs/debt/` registry mirroring `docs/backlog/` conventions, with:
- `DEBT.md` index (same shape as BACKLOG.md)
- `DEBT-NNN-<slug>.md` per-entry files
- Bidirectional linkage: debt entries carry `addressed_by: [BL-NNN, ...]`; backlog entries gain `addresses_debt: [DEBT-NNN, ...]` (empty means new implementation, non-empty means debt paydown)

**Taxonomy** — combine four authoritative philosophies:
- Cunningham 1992 (original metaphor): `principal` (effort to remediate) + `interest_rate` (carry cost: none | linear | compounding)
- Fowler 2009 Quadrant: `quadrant` ∈ {reckless-deliberate, reckless-inadvertent, prudent-deliberate, prudent-inadvertent}
- Kruchten/Nord/Ozkaya IEEE 2012: `debt_type` ∈ {architecture, design, code, test, documentation, infrastructure, dependency, requirement, process, build, knowledge}
- Industry (CAST/SonarQube/SEI) extends debt_type with {security, compliance, performance}

**Full entry schema**:
- `id`, `title`, `status` (open | addressed | accepted | archived), `created`
- `debt_type` (14-value enum above)
- `quadrant` (Fowler 4-value enum)
- `source_phase` (spec | design | implement)
- `source_agent` (which reviewer flagged it)
- `originating_spec` (path to producing artifact)
- `severity` (LOW | MEDIUM | HIGH | CRITICAL)
- `component` (affected crate/module)
- `remediation_hint` (short guidance)
- `addressed_by` (list of BL-NNN)
- `principal` (OPTIONAL effort estimate — hours/days)
- `interest_rate` (OPTIONAL: none | linear | compounding)

Principal and interest_rate are optional at capture time; filled during periodic review.

**Capture mechanism — hybrid**:
- Adversary/reviewer agents emit structured debt findings as part of their normal output (new section or JSON block)
- A dedicated phase-end consolidation step in `/spec`, `/design`, `/implement` commands collects findings across all agents run in that phase and writes DEBT-NNN entries to `docs/debt/`
- Deduplicate when the same finding appears across phases (by component + debt_type + fingerprint of the finding text)

**Surfacing for remediation — four channels**:
1. `/debt-review` command — walks open entries, prioritizes by `interest_rate × time_carried / principal`, suggests promoting to backlog via `/backlog add`
2. Audit integration — `/audit-full` and domain audits (archi, code, test, evolution) cross-reference `docs/debt/` and flag stale debt in their reports
3. `/spec` Phase 0 cross-reference — when a new spec starts, check if it addresses existing debt; suggest linking via `addresses_debt:`
4. `ecc debt status` — dashboard summary (count by type, severity distribution, oldest unpaid, total estimated principal)

**CLI surface** — mirror `ecc backlog`:
- `ecc debt list [--type X] [--severity Y] [--status Z]`
- `ecc debt add` (manual capture)
- `ecc debt update-status DEBT-NNN <status>`
- `ecc debt reindex` (rebuild DEBT.md from entries)
- `ecc debt next-id`
- `ecc debt status` (dashboard)

**Backlog extension**:
- Add `addresses_debt: [DEBT-NNN, ...]` field to backlog entry schema (empty = new implementation)
- `ecc backlog list --addresses-debt` filter
- Bidirectional sync: when a BL is promoted/implemented, update its linked DEBT entries to `addressed`

**Migration / bootstrap**:
- One-time scan of `docs/audits/` and `docs/specs/*/adversary-*.md` for deferred findings
- Produce seed DEBT-NNN entries for manual review

**Open design questions** (for /design phase):
- New crate `ecc-debt` vs extend `ecc-backlog` crate (shared index-file plumbing argues for extension; distinct domain model argues for new crate)
- Phase-end consolidation: hook-based (auto) vs command-based (explicit) vs both
- Finding fingerprint algorithm for deduplication across phases
- Schema validation approach (`ecc validate debt`) mirroring `ecc validate <target>` pattern

**Out of scope**:
- Automated debt paydown
- Auto-generated remediation specs (user must `/spec` explicitly from a debt entry)
- Debt import from external tools (SonarQube/CAST integration)

**References**:
- Cunningham, OOPSLA 1992 — original debt metaphor
- Fowler 2009 — https://martinfowler.com/bliki/TechnicalDebtQuadrant.html
- Kruchten/Nord/Ozkaya — "Technical Debt: From Metaphor to Theory and Practice" — IEEE Software 2012
- CAST Research, SonarQube taxonomy, SEI technical debt reports
```

## Original Input

Add a tech debt registry that links to backlog entries — capturing debt found by adversary agents during pipeline reviews and surfacing it for systematic paydown.

## Challenge Log

**Mode**: backlog-mode (escalated to HIGH scope — full 5 stages)

**Q1 (Location)**: Where should the debt registry live? → `docs/debt/` standalone directory, bidirectionally linked with `docs/backlog/`.

**Q2 (Capture trigger)**: When and how are debt entries created? → Hybrid: adversary agents emit structured findings during normal runs; phase-end consolidation step in `/spec`, `/design`, `/implement` writes DEBT-NNN entries.

**Q3 (Schema)**: What fields does each entry need? → Full combined schema (Cunningham + Fowler + Kruchten + industry): 14 debt_type values, Fowler quadrant, severity, source_phase, source_agent, originating_spec, component, remediation_hint, addressed_by, optional principal + interest_rate.

**Q4 (Surfacing)**: How does debt get surfaced for remediation? → All four channels: `/debt-review` command with priority scoring, audit integration, `/spec` Phase 0 cross-reference, `ecc debt status` dashboard.

## Related Backlog Items

- BL-064 (cartography / docs coverage) — overlaps with documentation debt_type entries
- BL-148 (session resume/persist/delegate hook lifecycle) — phase-end consolidation may intersect with hook lifecycle
