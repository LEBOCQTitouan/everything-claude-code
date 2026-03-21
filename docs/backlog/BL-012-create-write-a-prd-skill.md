---
id: BL-012
title: Create write-a-prd skill
tier: 3
scope: MEDIUM
target: /plan dev
status: open
created: 2026-03-20
file: skills/write-a-prd/SKILL.md
---

## Action

PRD generation through interactive interview + codebase exploration. Flow: (1) Ask user for detailed problem description and solution ideas. (2) Explore repo to verify assertions and understand current state. (3) Present alternative approaches, interview about tradeoffs. (4) Hammer out exact scope — what's in AND what's explicitly out. (5) Sketch major modules, actively seek deep modules (Ousterhout: small interface hiding significant complexity). (6) Check module design with user. (7) Write PRD to `docs/prds/{feature}-prd.md` using template: Problem Statement, Target Users, User Stories with acceptance criteria, Non-Goals, Risks & Mitigations, Module Sketch, Success Metrics, Open Questions. Trigger: "write a prd", "product requirements", "feature spec", "define what we're building". No tracker integration — file only.
