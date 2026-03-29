---
id: BL-107
title: "Audit-web guided profile — interactive setup, persisted dimensions, improvement suggestions"
scope: HIGH
target: "/spec-dev"
status: open
tags: [audit-web, profile, guided-setup, interactive, self-improvement]
created: 2026-03-29
related: [BL-081, BL-083]
---

# BL-107: Audit-Web Guided Profile & Self-Improvement

## Problem

`/audit-web` currently scans 8 hardcoded dimensions (deps, arch, tools, ecosystem, competitors, user-requests, blogs, research) with no project-specific customization. Issues:

1. **No guided setup** — the user must know which `--focus` dimensions to pick. The audit doesn't analyze the codebase to suggest what matters most.
2. **No self-improvement** — after producing findings, the audit never suggests gaps in its own coverage (e.g., "you have no CI pipeline, consider adding a `ci` dimension").
3. **No persistence** — custom audit configurations are lost between runs. Each run starts from scratch.

## Proposed Solution

### Feature A: Audit Self-Improvement (Post-Report Phase)

After the radar report, add a new Phase 5 that:
- Analyzes the findings for coverage gaps (e.g., "no findings in Platforms — is that because you have no infra, or because the audit didn't look?")
- Suggests new custom dimensions based on what the codebase actually contains
- Proposes threshold adjustments (e.g., "your strategic fit scores cluster at 3-4 — consider tightening the Adopt threshold")
- Persists suggestions to the audit profile for next run

### Feature B: Interactive Guided Setup (Phase 0)

Before the inventory scan, add a Phase 0 that:
- Scans the codebase for project characteristics (language, framework, CI, infra, domain)
- Generates a suggested audit profile: which dimensions to scan, custom dimensions for this project, focus areas
- Presents the profile interactively via AskUserQuestion — user can accept, modify, or add dimensions
- Can be re-invoked on any run (`/audit-web --setup` or automatically if no profile exists)

### Audit Profile Artifact

Persist the audit profile to `docs/audits/audit-web-profile.md` (or `.yaml`):
- Custom dimensions with search query templates
- Focus area priorities
- Threshold overrides
- History of improvement suggestions (accepted/rejected)

## Ready-to-Paste Prompt

```
/spec-dev Upgrade /audit-web with guided profile setup and self-improvement:

1. Phase 0: Interactive Guided Setup
   - Scan codebase for project characteristics (language, frameworks, CI config,
     infra files, domain patterns)
   - Generate suggested audit profile: dimensions to scan, custom dimensions,
     focus areas ranked by relevance
   - Present via AskUserQuestion — user accepts, modifies, or adds dimensions
   - Triggered automatically on first run (no profile exists) or explicitly
     via --setup flag on any run
   - Profile is editable between runs

2. Audit Profile Artifact
   - Persist to docs/audits/audit-web-profile.md
   - Format: YAML frontmatter with dimension list, custom dimensions with
     search query templates, focus priorities, threshold overrides
   - Include history section tracking accepted/rejected improvement suggestions
   - Profile is loaded at start of every /audit-web run

3. Phase 5: Self-Improvement Suggestions (Post-Report)
   - After the radar report, analyze findings for coverage gaps
   - Suggest new custom dimensions based on project characteristics not covered
   - Propose threshold adjustments based on score distributions
   - Present suggestions via AskUserQuestion — user accepts or rejects each
   - Accepted suggestions are persisted to the audit profile for next run

4. Integration
   - Update the audit-web command to load profile at Phase 1
   - Pass custom dimensions to web-scout alongside the 8 standard dimensions
   - Custom dimension agents use the search query templates from the profile
   - Profile changes are committed alongside the radar report

Constraints:
- Standard 8 dimensions remain available and default
- Custom dimensions add to (not replace) the standard set
- Profile is human-readable and editable outside the command
```
