---
id: BL-072
title: Deterministic artifact scaffolding — spec, solution, and tasks template generation
status: open
scope: MEDIUM
target: /spec dev
created: 2026-03-26
tags: [deterministic, templates, scaffolding, rust-cli]
related: [BL-029, BL-030]
---

# BL-072: Deterministic Artifact Scaffolding

## Problem

When creating spec.md, solution.md, and tasks.md artifacts, the LLM generates boilerplate structure:
- Frontmatter with date, title, concern fields
- Section headers (Overview, Acceptance Criteria, Edge Cases, etc.)
- AC numbering scaffold (AC-001.1, AC-001.2)
- PC table headers with column definitions
- tasks.md table with status trail format

This boilerplate is identical across all work items — the LLM only adds content, not structure.

## Proposed Solution

### `ecc scaffold spec <feature-name>`
Creates `docs/specs/YYYY-MM-DD-<slug>/spec.md`:
```markdown
---
title: <Feature Name>
concern: <prompted or inferred>
created: 2026-03-26
status: draft
---

# <Feature Name>

## Overview
<!-- Brief description -->

## User Story
As a [role], I want [goal], so that [benefit].

## Acceptance Criteria
- AC-001.1:

## Edge Cases
<!-- List edge cases -->

## Out of Scope
<!-- What this does NOT include -->
```

### `ecc scaffold solution <spec-path>`
Reads spec.md, extracts ACs, generates solution.md with:
- Pre-filled "Verifies AC" column from spec ACs
- Empty PC table rows
- File Changes table template
- SOLID Assessment scaffold

### `ecc scaffold tasks <solution-path>`
Reads solution.md PCs, generates tasks.md with:
- One row per PC
- Status trail initialized to `| pending`
- Pre-filled PC descriptions from solution

## Impact

- **Speed**: Instant scaffolding vs 30-60s LLM generation
- **Consistency**: Identical structure across all work items
- **Agent simplification**: LLM fills content only, not boilerplate

## Research Context

Continue.dev: template-based scaffolding for deterministic structure, LLM for content.
