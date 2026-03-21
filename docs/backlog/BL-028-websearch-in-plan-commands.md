---
id: BL-028
title: Add active web search phase to /spec-dev, /spec-fix, /spec-refactor
status: open
created: 2026-03-21
promoted_to: ""
tags: [plan, search-first, exa, web-search, best-practices, planning]
scope: MEDIUM
target_command: /spec-dev, /spec-fix, /spec-refactor
---

## Optimized Prompt

Add a mandatory web search phase to all three spec commands (`/spec-dev`, `/spec-fix`, `/spec-refactor`) so that every plan is grounded in current best practices, relevant libraries, and prior art — not just Claude's training data and the local codebase.

**Context:**
The `search-first` skill and `exa-web-search` MCP exist in the project but are only referenced in documentation or mentioned as optional. In practice, `/spec` commands do not invoke them. Plans are generated purely from codebase inspection and training knowledge, missing recent libraries, updated idioms, or established patterns for the specific problem at hand.

**Scope of change:**
- `commands/spec-dev.md`
- `commands/spec-fix.md`
- `commands/spec-refactor.md`
- Possibly `skills/search-first.md` if it needs a concrete invocation protocol

**What to implement:**

1. Insert a dedicated "Web Research" phase early in each plan command (after intent classification, before the grill-me interview or spec writing). The phase must:
   - Use `exa-web-search` MCP to search for: best practices for the stated problem, relevant libraries or tools available in the project's ecosystem, recent patterns, known pitfalls, and prior art
   - Produce a short "Research Summary" block (3-7 bullet points) that is carried forward into the planning phase
   - Be concrete and mandatory — not a suggestion or a reference to a separate skill

2. The Research Summary must appear in the plan output so the user can see what external knowledge shaped the plan.

3. Queries should be derived from the user's stated intent + detected tech stack, not hardcoded. Example derivation: "Rust async error handling best practices 2024", "tokio vs async-std comparison", "hexagonal architecture Rust crates".

**Acceptance criteria:**
- All three spec commands have a labeled "Web Research" phase with explicit `exa-web-search` calls
- The Research Summary is emitted as a visible section in the plan output
- The phase runs before the spec/solution is written, so findings can influence decisions
- If `exa-web-search` MCP is unavailable, the phase degrades gracefully with a warning (does not hard-fail the plan)
- The change does not increase plan token cost by more than ~10% on average (queries should be focused, not broad)

**Scope boundaries — do NOT:**
- Modify agents (planner, architect) directly — changes go in the command files only
- Add web search to `/verify`, `/review`, or other non-spec commands
- Rewrite the `search-first` skill — only reference or invoke it as needed
- Add persistent caching of search results

**Verification steps:**
1. Run `/spec-dev` on a sample feature request and confirm a "Web Research" section appears in output
2. Run `/spec-fix` on a sample bug description and confirm the research phase fires
3. Run `/spec-refactor` on a sample scope and confirm the research phase fires
4. Disable the MCP (or simulate unavailability) and confirm the plan degrades gracefully with a warning

## Original Input

The existing exa-web-search and search-first skill are NOT actually used in practice during planning. The user wants to ensure that when running /spec, Claude actively searches the web for best practices, libraries, patterns, and prior art — so the plan isn't limited to what Claude already knows about the codebase. This should be a concrete phase/step in the plan commands, not just a reference in docs. Applies to all three plan commands: plan-dev, plan-fix, plan-refactor.

## Challenge Log

**Q1: Which plan commands should this apply to?**
All three: `/spec-dev`, `/spec-fix`, `/spec-refactor`.

**Q2: The search-first skill and exa-web-search MCP already exist — what's the gap?**
They are referenced in documentation but not actively invoked during planning. The goal is to make web search a concrete, mandatory phase in each plan command, not an optional or implicit step.

**Q3: Scope and priority?**
MEDIUM — this touches three command files and potentially the search-first skill, but does not require new infrastructure.

## Related Backlog Items

- BL-009 — Add negative examples to planner agent (planner agent is downstream consumer of plan commands; better research phases improve planner input quality)
