---
name: write-a-prd
description: Interactive PRD generation through problem interview, codebase exploration, and structured template output to docs/prds/.
origin: ECC
---

# Write a PRD

Generate a Product Requirements Document through structured interview and codebase exploration.

## When to Activate

- User says "write a prd" or "product requirements"
- User says "feature spec" or "define what we're building"
- User wants structured product thinking before `/spec`

## Flow

Six steps, sequential. Use AskUserQuestion for each user input — one question at a time. If AskUserQuestion is unavailable, fall back to conversational questions.

### Step 1: Problem Interview

Ask the user to describe: the problem, who it affects, current workarounds, and why now. Probe vague answers — "just a CLI tool" needs decomposition.

### Step 2: Codebase Exploration

Use Read, Grep, and Glob to verify user assertions about the current state. Surface contradictions: "You said there's no config module, but `src/config/` exists with 12 files." Establish baseline context even for greenfield features.

### Step 3: Alternatives and Tradeoffs

Present at least 2 approaches. Interview about complexity, maintenance, and compatibility tradeoffs. If only one viable approach exists, confirm explicitly. Capture the chosen approach and rationale.

### Step 4: Scope Hammering

Define what is explicitly IN scope and what is OUT (non-goals). Push back on expanding scope — note complexity growth. Flag contradictions between in-scope and non-goals.

### Step 5: Module Sketch

Propose major modules with interface descriptions. Check with the user — they can add, remove, or rename. For simple features, note "single-module" and move on.

### Step 6: Write PRD

Write to `docs/prds/{feature}-prd.md` (kebab-case slug, max 40 chars). Create the `docs/prds/` directory automatically if missing. If a PRD already exists at the path, ask whether to overwrite or append a revision.

## Template

The PRD file uses this structure. Empty sections use "None identified" rather than being omitted.

| Section | Content |
|---------|---------|
| Problem Statement | What, who, why, current state |
| Target Users | Primary and secondary personas |
| User Stories | US-NNN with acceptance criteria |
| Non-Goals | Explicitly excluded scope |
| Risks & Mitigations | Technical, product, timeline risks |
| Module Sketch | Major modules and interfaces |
| Success Metrics | How to measure if this worked |
| Open Questions | Unresolved items for follow-up |
