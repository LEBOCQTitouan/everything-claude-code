# ADR-0033: Socratic Questioning Protocol

## Status

Accepted (2026-03-29). Partially supersedes ADR-0017 (question cap removed, depth profiles introduced).

## Context

The grill-me skill produced shallow questioning — surface-level questions, no progressive depth drilling, no mutual understanding confirmation. The 25-question cap forced breadth over depth. Research identified four SOTA techniques that address these gaps: OARS (motivational interviewing), Laddering (depth-first elicitation), MECE (exhaustive decomposition), and Socratic questioning taxonomies.

## Decision

### 1. OARS with Acknowledge (not Affirm)
After every user answer: Open follow-up, Acknowledge (factual recognition without praise — "That addresses the concurrency concern"), Reflect (mandatory restatement), Summarize (at stage transitions). Acknowledge replaces Affirm to avoid sycophantic patterns.

**Research mapping**: SMART-DREAM (ACM 2024) demonstrates that structured OARS sequencing improves agent-led conversations.

### 2. Uncapped Laddering Depth
Progressive "why?" drilling governed by answer specificity (concrete nouns, measurable quantities, falsifiable claims), not a fixed count. Safety valve at depth 7 prevents infinite regress.

**Research mapping**: LadderBot (KIT 2019) validates laddering for automated requirement interviews.

### 3. Universal MECE Decomposition
Every requirement/design space is decomposed into Mutually Exclusive, Collectively Exhaustive sub-questions. Binary splits isolate root causes. Atomic/non-decomposable topics are exempt with rationale.

**Research mapping**: McKinsey/BCG issue tree methodology.

### 4. Socratic 6-Type Rotation with Visible Annotations
Questions annotated with type tags visible to the user: [Clarification], [Assumption], [Evidence], [Viewpoint], [Implication], [Meta]. No type exceeds 2x fair share.

**Research-to-implementation mapping**:
- Elenchus (cross-examination) → [Assumption]
- Maieutics (drawing out knowledge) → [Evidence]
- Dialectic (thesis/antithesis) → [Viewpoint]
- Generalization → [Implication]
- Definition → [Clarification]
- Counterfactual → [Meta]

**Research mapping**: Chang (2023) — Prompting LLMs with the Socratic Method (arXiv:2303.08769).

### 5. Depth Profiles
Three intensity levels: shallow (OARS Reflect only, minimal MECE), standard (full OARS, 1-2 ladder levels, MECE), deep (all techniques, unlimited laddering). Mode defaults: backlog=standard, spec=deep, standalone=deep. Consumer commands can override.

## Consequences

- Questions drill deeper, producing higher-quality specs
- Reflective rephrasing confirms mutual understanding before proceeding
- Universal MECE ensures exhaustive coverage of requirement spaces
- Depth profiles prevent over-questioning on simple topics
- 25-question cap removed — depth is governed by answer specificity
- All consumers (/spec-*, /backlog, standalone) benefit automatically
- grill-me-adversary companion updated to use enhanced question types
