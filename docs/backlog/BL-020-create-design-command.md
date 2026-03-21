---
id: BL-020
title: Create /design command
tier: 4
scope: LOW
target: direct edit
status: open
created: 2026-03-20
file: commands/design.md
---

## Action

Thin wrapper that invokes the design-an-interface skill (BL-014). Takes a module name or path as `$ARGUMENTS`. Set `disable-model-invocation: true`. Spawns the parallel sub-agents, presents comparison, writes output to `docs/designs/`. Useful at port/adapter boundaries in hexagonal architecture work.

## Dependencies

- BL-014 (design-an-interface skill)
