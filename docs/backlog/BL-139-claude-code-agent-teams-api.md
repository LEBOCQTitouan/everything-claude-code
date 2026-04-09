---
id: BL-139
title: "Monitor Claude Code Agent Teams API for ECC integration"
scope: LOW
target: "/spec-dev"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [claude-code, api, agents]
---

## Context

Anthropic shipped Agent Teams (parallel sub-agents) and Channels (Discord/Telegram/webhook). ECC's agent orchestration patterns should map to Agent Teams API when stable. Monitor Channels for notification integration.

## Prompt

Track Claude Code Agent Teams API stability. When it reaches GA, design a mapping from ECC's current Agent tool dispatch to the Agent Teams API. Evaluate whether ECC's wave dispatch, tdd-executor, and team manifests can leverage native Agent Teams for better isolation and parallel execution.

## Acceptance Criteria

- [ ] Agent Teams API docs reviewed
- [ ] Mapping document: ECC patterns → Agent Teams equivalents
- [ ] Prototype integration if API is stable
