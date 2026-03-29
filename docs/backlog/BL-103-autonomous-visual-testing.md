---
id: BL-103
title: "Autonomous visual testing integration — vision-based UI validation"
scope: HIGH
target: "/spec-dev"
status: open
tags: [testing, e2e, visual, vision, browser-automation]
created: 2026-03-29
related: []
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-103: Autonomous Visual Testing Integration

## Problem

ECC's /e2e command uses Playwright for browser automation but lacks vision-based UI validation. Users actively request AI agents that can launch apps, interact with UIs, and validate visual results (Claude Code issue #31532).

## Proposed Solution

Extend the e2e-runner agent to support visual testing:
- Screenshot capture at key interaction points
- Claude vision API for visual assertion ("does this look correct?")
- Visual regression detection (compare screenshots across runs)
- Integration with existing Playwright test infrastructure

## Ready-to-Paste Prompt

```
/spec-dev Extend the e2e-runner agent with autonomous visual testing:

1. Screenshot Capture
   - Add screenshot hooks at key interaction points in Playwright tests
   - Store screenshots as test artifacts with metadata (URL, viewport, timestamp)

2. Vision-Based Assertions
   - Use Claude's vision capabilities to validate UI state from screenshots
   - Support natural language assertions: "the login form should show an error"
   - Compare actual screenshots against expected visual state descriptions

3. Visual Regression Detection
   - Compare screenshots across test runs for unexpected visual changes
   - Generate visual diff reports with highlighted changes

4. Integration
   - Extend e2e-runner agent to support --visual flag
   - Add visual test examples to the e2e-testing skill

Reference: Claude Code #31532, Browser Use (78K GitHub stars)
Source: docs/audits/web-radar-2026-03-29-r2.md
```
