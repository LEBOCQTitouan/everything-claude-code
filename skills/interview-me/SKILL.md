---
name: interview-me
description: Collaborative requirements interview for structured extraction. Helps users think through current state, desired state, constraints, stakeholders, dependencies, and failure modes before speccing.
origin: ECC
---

# Interview Me

Collaborative requirements interview that helps users extract and structure their thinking before speccing a feature, fix, or refactor.

## When to Activate

- User says "interview me" or "help me think through"
- User says "extract requirements" or "what should I consider"
- User wants structured help articulating a fuzzy idea before committing to a spec

## Interview Stages

The interview proceeds through 8 sequential stages. Adapt depth per stage to the complexity of the topic. Skip stages only when the user explicitly requests it, recording skipped stages in the output.

### Stage 1: Current State

Understand what exists today. What is the current system, process, or situation? What works well? What does not?

### Stage 2: Desired State

Clarify what the user wants. What does the end state look like? What changes? What stays the same?

### Stage 3: Constraints

Surface technical, timeline, budget, and organizational constraints. What cannot change? What boundaries must be respected?

### Stage 4: Security Checkpoint

HARD GATE. Review all information gathered so far for security implications: authentication, authorization, data exposure, secret handling, input validation, injection vectors. Flag every unaddressed security concern. MUST NOT proceed to Stage 5 until every flagged concern has an explicit mitigation or accepted-risk acknowledgment from the user.

### Stage 5: Stakeholders

Identify who is affected by this change and who decides. Who benefits? Who bears the cost? Who has veto power?

### Stage 6: Dependencies

Map what this work depends on and what depends on this work. Upstream and downstream impacts, blocking vs non-blocking.

### Stage 7: Prior Art

Explore existing solutions, patterns, libraries, and internal precedents. What has been tried before? What can be reused or adapted?

### Stage 8: Failure Modes

Enumerate what can go wrong. For each failure mode, identify detection strategy, blast radius, and recovery plan.

## Output

After all stages are complete (or the user explicitly ends the interview), persist structured notes to `docs/interviews/{topic}-{date}.md` containing: stages completed, key findings per stage, open questions, and a requirements summary.

## Anti-Patterns

- DO NOT skip the security checkpoint even if the feature seems low-risk
- DO NOT answer for the user — extract their knowledge, do not impose yours
- DO NOT batch multiple questions in one turn — ask one at a time, wait for the response
- DO NOT assume shared vocabulary — clarify ambiguous terms before proceeding

## Distinction from Grill-Me

Interview-me is **collaborative**: it helps the user extract, organize, and structure their own thinking. Grill-me is **adversarial**: it challenges assumptions and stress-tests ideas through relentless questioning. Use interview-me when requirements are fuzzy and need shaping; use grill-me when a proposal exists and needs hardening.
