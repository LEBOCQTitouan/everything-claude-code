---
id: BL-024
title: "Add context:fork to heavy skills"
tier: 6
scope: LOW
target: direct edit
status: "implemented"
created: 2026-03-20
files: doc-orchestrator, audit-full, and other skills that scan >50 files
---

## Action

Add `context: fork` to frontmatter of skills that do large-scale codebase scanning so they run in a subagent with their own context window. Not urgent until you observe context pressure during long sessions, but worth doing proactively for doc-orchestrator (383 lines, heaviest agent) and audit-full (runs all audit domains in parallel).
