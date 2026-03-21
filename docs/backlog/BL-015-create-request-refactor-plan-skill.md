---
id: BL-015
title: Create request-refactor-plan skill
tier: 3
scope: MEDIUM
target: /spec dev
status: open
created: 2026-03-20
file: skills/request-refactor-plan/SKILL.md
---

## Action

Interview-driven refactoring plan with tiny commits. Flow: (1) Interview user about what needs to change and why. (2) Explore codebase to understand current structure. (3) Identify all affected files and modules. (4) Decompose into the smallest possible commits — each commit must leave the codebase in a green (compiling + tests passing) state. (5) Order commits by dependency. (6) For each commit: describe the change, list affected files, state the risk, provide rollback instruction. Write plan to `docs/refactors/{name}-plan.md`. Trigger: "refactor plan", "plan a refactoring", "how should I restructure this", "tiny commits for this change". No GitHub Issues — file only.
