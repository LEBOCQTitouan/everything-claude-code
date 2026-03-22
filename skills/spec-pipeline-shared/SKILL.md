---
name: spec-pipeline-shared
description: Shared sections for spec-dev, spec-fix, and spec-refactor commands — project detection, grill-me interview rules, adversarial review + verdict handling, and spec output schema.
origin: ECC
---

# Spec Pipeline Shared

Canonical shared content for all `/spec-*` commands. Each command references this skill instead of inlining duplicated sections.

## Project Detection

Detect the project's test, lint, and build commands using auto-detection:

- **Test**: `cargo test` (Rust) → `npm test` (Node) → `go test ./...` (Go) → `pytest` (Python)
- **Lint**: `cargo clippy -- -D warnings` → `npm run lint` → `golangci-lint run` → `ruff check .`
- **Build**: `cargo build` → `npm run build` → `go build ./...`

Store detected commands for use in spec constraints and pass conditions.

## Grill-Me Interview Rules

These rules govern all grill-me interviews across spec commands:

- Ask **one question at a time** using `AskUserQuestion`. WAIT for the user's response before asking the next question. NEVER present multiple questions in a single turn.
- For each question, use `AskUserQuestion` with the recommended answer as the first option (append "(Recommended)" to the label). Add 1-2 alternative options where relevant. The user can always select "Other" to provide a custom answer.
- Explore the codebase yourself instead of asking the user when the answer is findable in code.
- The user can say **"spec it"** at any point to accept all remaining recommended answers and skip to the spec. Check for this after each answer.
- Do NOT proceed to the spec until the user says **"spec it"** or all questions are answered.

## Adversarial Review + Verdict Handling

After spec output, launch a Task with the `spec-adversary` agent (allowedTools: [Read, Grep, Glob]):

- Pass the full spec from conversation context
- The agent attacks the spec on 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback

### Verdict Handling (max 3 rounds)

- **FAIL**: Return to Grill-Me to address issues. Re-output spec, re-run adversary. Increment round.
- **CONDITIONAL**: Add suggested ACs to spec. Re-run adversary. Increment round.
- **PASS**: Persist spec to `docs/specs/YYYY-MM-DD-<slug>/spec.md`. Run `!bash .claude/hooks/phase-transition.sh solution plan <spec_file_path>`.

After 3 FAIL rounds, offer user override or abandon.

### Persist Spec to File

1. Generate slug from feature description (lowercase, hyphens, max 40 chars)
2. Create directory `docs/specs/YYYY-MM-DD-<slug>/`
3. Write full spec to `spec.md` (append `## Revision` block if file exists)
4. Pass file path to phase-transition command

## Spec Output Schema

All spec commands output these mandatory sections:

`# Spec: <title>` → `## Problem Statement` → `## Research Summary` → `## Decisions Made` (table) → `## User Stories` (US-NNN with ACs) → `## Affected Modules` → `## Constraints` → `## Non-Requirements` → `## E2E Boundaries Affected` (table) → `## Doc Impact Assessment` (table) → `## Open Questions`
