---
id: BL-157
title: Tech Debt Registry linked to Backlog
status: open
scope: HIGH
target_command: /spec
created: 2026-04-18
addresses_debt: []
tags: [debt, registry, pipeline, adversary, cli, audit]
---

## Problem

Adversarial reviewers (spec-adversary, solution-adversary, design-reviewer, uncle-bob, robert, pc-evaluator) regularly flag design shortcuts, SOLID violations, missing tests, doc gaps, and architectural concessions during the `/spec` → `/design` → `/implement` pipeline. Today these findings are fixed ad-hoc or forgotten entirely — there is no persistent registry to track what debt has been accepted, what pays it down, and what remains open. The backlog captures future intentions; nothing captures deliberately deferred technical obligations.

## Solution Sketch

New `docs/debt/` registry bidirectionally linked with `docs/backlog/`:
- `DEBT.md` index (same shape as BACKLOG.md) and `DEBT-NNN-<slug>.md` per-entry files
- Hybrid capture: adversary/reviewer agents emit structured debt findings in their normal output; a phase-end consolidation step in `/spec`, `/design`, `/implement` collects those findings and writes DEBT-NNN entries
- Full taxonomy combining Cunningham (principal + interest), Fowler quadrant, Kruchten/Nord/Ozkaya debt type, and industry extensions (CAST/SonarQube/SEI — 14 total debt types)
- Four surfacing channels: `/debt-review` command, audit integration, `/spec` Phase 0 cross-reference, `ecc debt status` dashboard
- CLI surface mirroring `ecc backlog`: list, add, update-status, reindex, next-id, status

## Acceptance Criteria

- [ ] Every adversary agent output in `/spec`, `/design`, `/implement` can produce structured debt findings (new section or JSON block)
- [ ] `docs/debt/DEBT.md` index exists with documented schema matching the 14-field entry format
- [ ] `DEBT-NNN-<slug>.md` per-entry files are created by phase-end consolidation steps
- [ ] `ecc debt` CLI exposes: `list [--type X] [--severity Y] [--status Z]`, `add`, `update-status`, `reindex`, `next-id`, `status`
- [ ] Backlog entry schema gains `addresses_debt: [DEBT-NNN, ...]` field; `ecc backlog` supports `--addresses-debt` filter
- [ ] Bidirectional sync: promoting/implementing a BL entry updates linked DEBT entries to `addressed`
- [ ] `/debt-review` command exists and prioritizes by `interest_rate × time_carried / principal`
- [ ] Audit commands (`/audit-full`, domain audits) cross-reference `docs/debt/` and flag stale entries
- [ ] `/spec` Phase 0 checks for debt linkage via `addresses_debt:` and surfaces matches
- [ ] Deduplication works across phases by component + debt_type + finding fingerprint
- [ ] `ecc validate debt` validates entry files against schema
- [ ] One-time migration scan of `docs/audits/` and `docs/specs/*/adversary-*.md` produces seed DEBT-NNN entries for manual review

## Scope & Non-Goals

**In scope:**
- `docs/debt/` registry (DEBT.md index + DEBT-NNN-<slug>.md entries)
- Full entry schema (14 fields + optional principal/interest_rate)
- Phase-end consolidation in `/spec`, `/design`, `/implement`
- Adversary agent output changes to emit structured debt findings
- `ecc debt` CLI (list/add/update-status/reindex/next-id/status)
- Backlog `addresses_debt` field + bidirectional sync
- `/debt-review` command
- Audit cross-referencing
- `/spec` Phase 0 debt linkage check
- `ecc validate debt`
- One-time migration bootstrap

**Out of scope:**
- Automated debt paydown (user must explicitly `/spec` from a debt entry)
- Auto-generated remediation specs
- Debt import from external tools (SonarQube, CAST, Jira)

## Open Questions for /design

- New crate `ecc-debt` vs. extend `ecc-backlog` crate (shared index-file plumbing argues for extension; distinct domain model argues for a new crate)
- Phase-end consolidation: hook-based (automatic, zero agent changes) vs. command-based (explicit, user-controlled) vs. both
- Finding fingerprint algorithm for deduplication across phases (hash of component + debt_type + normalized finding text?)
- Schema validation approach — extend `ecc validate <target>` dispatcher or standalone `ecc validate debt`
- Migration scope — how deeply to scan `docs/audits/` and adversary artifacts for retroactive seed entries

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

Add a tech debt registry to ECC, linked to the backlog, so that deferred technical debt surfaced by adversary agents during the pipeline is persistently tracked rather than lost.

## Challenge Log

**Mode**: backlog-mode (escalated to full 5 stages — HIGH scope)

**Q1 (Location)**: Where should the debt registry live?
**A**: `docs/debt/` standalone directory, bidirectionally linked with `docs/backlog/`.

**Q2 (Capture trigger)**: When and how are debt entries created?
**A**: Hybrid — adversary agents emit structured findings during normal runs; phase-end consolidation step in `/spec`, `/design`, `/implement` writes DEBT-NNN entries.

**Q3 (Schema)**: What fields does each entry need?
**A**: Full combined schema (Cunningham + Fowler + Kruchten + industry): 14 debt_type values, Fowler quadrant, severity, source_phase, source_agent, originating_spec, component, remediation_hint, addressed_by, optional principal + interest_rate.

**Q4 (Surfacing)**: How does debt get surfaced for remediation?
**A**: All four channels — `/debt-review` command with priority scoring, audit integration, `/spec` Phase 0 cross-reference, `ecc debt status` dashboard.

## Related Backlog Items

- BL-064 (cartography / docs coverage) — overlaps with documentation debt_type entries
- BL-091 (diagnostics) — audit cross-reference overlap
- BL-093 (memory system) — structured agent output patterns
