---
id: BL-077
title: "Full documentation pass — coverage, drift validation, and gap analysis"
scope: EPIC
target: /spec-dev
status: open
created: 2026-03-27
related: [BL-050, BL-056, BL-064]
---

# BL-077 — Full documentation pass — coverage, drift validation, and gap analysis

## Problem

Documentation has been updated incrementally during /implement Phase 7.5 runs, but many features were implemented before Phase 7.5 existed or through direct edits that skipped doc updates. The result is uneven coverage: some crates have full MODULE-SUMMARIES entries and diagrams, others have none. ADRs may reference superseded decisions, runbooks may cite stale CLI flags, and CLAUDE.md stats (test count, command tables) may be out of date.

## Goal

Bring all project documentation to maximum coverage and accuracy in a single coordinated effort:

1. **Gap analysis** — identify every undocumented module, command, domain concept, and public API surface
2. **Drift validation** — cross-reference doc claims (function names, CLI flags, paths, architecture descriptions) against actual code and fix mismatches
3. **Generation** — fill all gaps: MODULE-SUMMARIES.md, ARCHITECTURE.md, bounded-contexts.md, ADRs, runbooks, getting-started.md, commands-reference.md, domain glossary
4. **Diagrams** — generate/update sequence, component, dependency, and C4 diagrams for all crates
5. **CLAUDE.md** — update test count, command tables, gotchas, architecture pointer
6. **Validation** — run doc-validator against all generated docs, score quality, fix issues

## Constraints (from grill-me)

- **Scope:** Full doc suite + gap analysis + drift detection against code — not just coverage
- **CLAUDE.md:** Included in scope (not deferred to /claude-md-improver)
- **Drift:** Validate doc claims against actual code, not just ensure docs exist
- **Execution:** Via /spec to design a custom pass extending /doc-suite capabilities
- **Session management:** Must include explicit session break checkpoints with /catchup resume points — this is too large for a single session

## Ready-to-Paste Prompt

```
/spec-dev

Full documentation pass for the ECC project — bring all docs to maximum coverage and accuracy.

## What needs to happen

1. **Gap analysis phase** (Session 1):
   - Run doc-analyzer across all 7 crates to identify undocumented public API surfaces
   - Scan commands/, agents/, skills/, hooks/ for undocumented or under-documented items
   - Map domain concepts in ecc-domain against docs/domain/bounded-contexts.md
   - Produce a gap report with priority ranking (high-traffic modules first)

2. **Drift validation phase** (Session 1 or 2):
   - Cross-reference ARCHITECTURE.md claims against actual crate structure
   - Validate commands-reference.md CLI flags/subcommands against clap definitions
   - Check ADR decisions against current code (are any superseded?)
   - Validate runbook procedures against current CLI behavior
   - Produce a drift report with severity (blocking vs cosmetic)

3. **Generation phase** (Sessions 2-3):
   - Fill MODULE-SUMMARIES.md entries for all 7 crates
   - Update ARCHITECTURE.md with current hex arch layers and crate boundaries
   - Update bounded-contexts.md with current domain model
   - Generate/update sequence diagrams for key workflows (spec, design, implement, audit)
   - Generate/update C4 component diagrams for crate dependencies
   - Write missing ADRs for decisions made since last ADR
   - Update getting-started.md with current setup steps
   - Update commands-reference.md with current CLI surface

4. **CLAUDE.md update** (Session 3):
   - Update test count (cargo test --no-run count)
   - Update command tables
   - Update gotchas section
   - Verify architecture pointer is current

5. **Validation phase** (Session 3):
   - Run doc-validator on all generated/updated docs
   - Score quality using doc-quality-scoring rubric
   - Fix any issues below threshold

## Session break strategy

Each numbered phase above is a session checkpoint. At the end of each session:
- Commit all doc changes
- Update a tracking file (docs/specs/BL-077/progress.md) with completed items
- Next session resumes via /catchup

## Acceptance criteria

- Every public function/struct in all 7 crates has a MODULE-SUMMARIES entry
- Every CLI subcommand has a commands-reference entry matching clap definitions
- Every ADR is current (superseded ones are marked as such)
- All diagrams reflect current architecture
- CLAUDE.md test count matches `cargo test` output
- Doc-validator passes with no CRITICAL or HIGH issues
- Doc quality score >= 80% across all dimensions
```
