---
id: BL-010
title: Create ubiquitous-language skill
tier: 3
scope: MEDIUM
target: /spec dev
status: implemented
created: 2026-03-20
file: skills/ubiquitous-language/SKILL.md
---

## Action

Domain language extraction skill. Takes a bounded context name or module path as `$ARGUMENTS`. Reads source code types, function names, doc comments, test names, and error variants. Outputs a structured glossary as a Markdown table: term -> definition -> source file:line -> aliases/synonyms. Writes to `docs/domain/glossary-{context}.md`. Trigger patterns: "ubiquitous language", "domain glossary", "bounded context terms", "extract domain vocabulary", "what terms does this module use". Negative example: "DO NOT invent terms that don't exist in the code — extraction only, never synthesis". Keep SKILL.md under 500 words; no `references/` needed for v1.
