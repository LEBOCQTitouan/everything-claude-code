---
id: BL-102
title: "Promptware Engineering practices — prompt testing and monitoring"
scope: MEDIUM
target: "/spec-dev"
status: open
tags: [prompts, testing, quality, agents, skills]
created: 2026-03-29
related: [BL-090]
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-102: Promptware Engineering Practices

## Problem

ECC already treats prompts (agents, skills, commands) as versioned Markdown artifacts with frontmatter. However, there's no systematic testing or monitoring of prompt quality — changes to agent directives are validated only by manual observation.

## Proposed Solution

Apply Promptware Engineering lifecycle practices (arXiv 2503.02400):
- Prompt testing: automated eval harness for agent/skill output quality
- Prompt debugging: structured approach to diagnosing prompt regression
- Prompt monitoring: track prompt effectiveness over time

## Ready-to-Paste Prompt

```
/spec-dev Add promptware engineering practices to ECC:

1. Prompt Testing Framework
   - Create an eval harness that runs agent/skill prompts against known inputs
   - Score outputs on correctness, format compliance, and instruction adherence
   - Integrate with /ecc-test-mode for regression testing

2. Prompt Quality Metrics
   - Define measurable quality dimensions for each agent type
   - Track prompt effectiveness across sessions via structured logs

3. Prompt Debugging Guide
   - Document structured approach for diagnosing prompt regressions
   - Add to docs/runbooks/

Reference: arXiv 2503.02400 (Promptware Engineering framework)
Source: docs/audits/web-radar-2026-03-29-r2.md
```
