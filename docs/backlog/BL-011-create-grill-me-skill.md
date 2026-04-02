---
id: BL-011
title: Create grill-me skill
tier: 3
scope: MEDIUM
target: /spec dev
status: "implemented"
created: 2026-03-20
file: skills/grill-me/SKILL.md
---

## Action

Standalone adversarial interview skill. Uses `AskUserQuestion` for every question — one question per turn, wait for answer before proceeding. Never answer its own questions or offer solutions.

### Stages (sequential, gate each before moving on)

1. **Problem** — What user problem does this solve? Who is affected? What happens if we don't do it? Drill until the problem statement is unambiguous and falsifiable.
2. **Edge cases** — Walk every boundary condition, degenerate input, concurrent scenario, and failure mode. Do not accept "it will be fine" — demand specifics.
3. **Scope** — What is explicitly included? What is explicitly NOT included? Pin down every ambiguous border until there is zero overlap with adjacent concerns.
4. **Rollback** — What is the undo plan? At what point is rollback no longer possible? What data or state is irreversible?
5. **Success criteria** — How will we know it worked? What metrics, signals, or observable behaviors prove success? Reject vague criteria ("it should be fast") — demand measurable thresholds.

### Relentless questioning

Within each stage, keep asking follow-up questions until every aspect is clearly stated. Do NOT move to the next stage while open branches remain. Track resolved vs open branches explicitly and show progress to the user.

### Vocabulary miscomprehension detection

Before accepting any user answer, check for possible vocabulary miscomprehension: ambiguous terms, overloaded words, domain jargon used loosely, or terms that mean different things in different contexts. When detected, surface the ambiguity immediately ("When you say X, do you mean A or B?") and do not proceed until the term is pinned down.

### Negative examples

- DO NOT answer your own questions
- DO NOT offer solutions — you are interviewing, not consulting
- DO NOT validate prematurely — "that sounds great" is not interviewing
- DO NOT accept hand-waving — demand specifics for every claim
- DO NOT skip a stage because the user seems eager to move on
- DO NOT assume shared vocabulary — verify meaning of key terms

### Output

Interview transcript + refined problem statement written to `docs/interviews/{topic}-{date}.md`.

### Trigger

"grill me", "challenge my assumptions", "stress test this idea", "poke holes in this".
