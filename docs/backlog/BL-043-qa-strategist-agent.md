---
id: BL-043
title: Add QA strategist agent
status: open
created: 2026-03-21
scope: LOW
target_command: agents/qa-strategist.md (new)
tags: [bmad, qa, testing, strategy, validation]
---

## Optimized Prompt

Create a dedicated QA strategist agent (`agents/qa-strategist.md`) that independently validates test plans before /implement begins. The agent reviews: edge case coverage completeness, boundary condition testing, integration test adequacy, E2E scenario selection, and test isolation. It receives the spec + design from conversation and produces a QA assessment with: coverage gaps, missing edge cases, suggested test scenarios, and a confidence score. Invoked optionally between /design and /implement, or as part of /design's review phases. Model: opus. Tools: Read, Grep, Glob. Skills: test-architecture, tdd-workflow.

## Framework Source

- **BMAD**: Quinn (QA Engineer) persona — designs test strategy, identifies edge cases, validates ACs independently

## Related Backlog Items

- None
