---
id: BL-011
title: Create grill-me skill
tier: 3
scope: MEDIUM
target: /plan dev
status: open
created: 2026-03-20
file: skills/grill-me/SKILL.md
---

## Action

Standalone adversarial interview skill. One question per turn, wait for answer before proceeding. Walk the decision tree branch by branch. Before asking a question the codebase could answer, read the codebase first and present findings as context for a sharper question. Structured rounds: (1) What user problem does this solve? (2) What happens if we don't do it? (3) Edge cases? (4) Scope boundaries — what is explicitly NOT included? (5) Who else is affected? (6) Rollback plan? (7) How will we know it worked? Track resolved vs open branches. Output: interview transcript + refined problem statement written to `docs/interviews/{topic}-{date}.md` (GitHub-decoupled). Trigger: "grill me", "challenge my assumptions", "stress test this idea", "poke holes in this". Negative examples: "DO NOT answer your own questions", "DO NOT offer solutions — you are interviewing, not consulting", "DO NOT validate prematurely — 'that sounds great' is not interviewing", "DO NOT accept 'it will be fine' — demand specifics".
