---
id: BL-013
title: Create interview-me skill
tier: 3
scope: MEDIUM
target: /plan dev
status: open
created: 2026-03-20
file: skills/interview-me/SKILL.md
---

## Action

Collaborative (non-adversarial) requirements interview. Distinct from grill-me: this is helpful extraction, not hostile stress-testing. Claude asks structured questions covering: current state, desired end state, constraints (technical, timeline, budget), stakeholders, dependencies, prior art, failure modes. Reads codebase before asking to avoid wasting the user's time on questions it can answer itself. Hard-blocks on security gaps — if the user's plan has obvious security implications they haven't addressed, flag immediately. Output: structured interview notes to `docs/interviews/{topic}-{date}.md`. Trigger: "interview me", "help me think through", "extract requirements", "what should I consider". Negative example: "DO NOT skip the security checkpoint even if the feature seems low-risk".
