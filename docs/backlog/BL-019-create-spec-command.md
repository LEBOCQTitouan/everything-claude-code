---
id: BL-019
title: Create /spec command
tier: 4
scope: MEDIUM
target: /plan dev
status: open
created: 2026-03-20
file: commands/spec.md
---

## Action

Runs the `/plan` -> `/solution` pipeline and halts after solution-adversary passes. Does NOT enter `/implement`. Produces a reviewed spec+solution artifact pair without committing to implementation. Useful for: design discussions, architecture reviews, getting robert's opinion before investing implementation time, sharing a plan with collaborators. Set `disable-model-invocation: true`. On completion, print: "Spec and solution are ready at {paths}. Run `/implement` when ready to proceed, or `/grill` to stress-test further."
