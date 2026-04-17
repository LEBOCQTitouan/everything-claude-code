---
id: BL-108
title: Smart stop notification — only notify on final stop or user input needed
status: implemented
scope: MEDIUM
target: /spec-dev
tags: [hooks, notifications, ux]
created: 2026-03-29
---

# BL-108 — Smart Stop Notification

## Problem

The `stop_notify` hook (tier2) fires on every `Stop` event, including intermediate subagent completions. This causes excessive notifications — the user gets pinged multiple times during a single orchestrated turn (e.g., parallel audit agents finishing one by one). The user should only be notified when:

1. **User input is required** (permission prompt, AskUserQuestion)
2. **All Claude actions are terminated** (top-level turn complete, no pending subagents)

## Investigation Needed

- Determine what context the `Stop` hook JSON payload provides (is there a nesting depth, parent agent ID, or "reason" field that distinguishes subagent stops from the final stop?)
- If no such signal exists, design a mechanism to track nesting depth (e.g., increment on subagent spawn, decrement on subagent stop, notify only when depth reaches 0)
- Consider whether Claude Code exposes hook event metadata that could be leveraged

## Ready-to-Paste Prompt

```
/spec-dev Upgrade the stop_notify hook (crates/ecc-app/src/hook/handlers/tier2_notify.rs) to only
fire notifications in two cases:

1. User input is required (the stop reason indicates a permission prompt or question)
2. The top-level Claude turn is fully complete (no pending subagents)

Currently the hook fires on every Stop event including intermediate subagent completions,
causing notification spam during orchestrated multi-agent turns.

Investigation: check what fields the Stop hook stdin JSON contains — look for nesting depth,
stop reason, or parent context. If no distinguishing signal exists, design a lightweight
nesting-depth tracker (increment on agent spawn, decrement on agent stop, notify at zero).

Constraints:
- Fire-and-forget semantics must be preserved (notification failures never block)
- ECC_NOTIFY_ENABLED=0 opt-out must still work
- All existing sanitization and cross-platform logic stays intact
- Add ECC_NOTIFY_MODE env var: "all" (current behavior), "smart" (new default)
```
