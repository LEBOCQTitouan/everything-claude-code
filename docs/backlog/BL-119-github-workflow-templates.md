---
id: BL-119
title: "Create GitHub workflow templates for Claude Code integration"
scope: HIGH
target: "/spec-dev"
status: open
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: adopt
tags: [feature, github, claude-code, templates]
---

## Context

Multiple community repos (shinpr/claude-code-workflows, anthropics/claude-code-action) and blog posts demonstrate strong demand for standardized CI/CD integration patterns with Claude Code. ECC is positioned to provide reference templates and hooks as first-class content.

## Prompt

Create a set of reusable GitHub Actions workflow templates for Claude Code integration. Templates should cover: (1) PR review with Claude Code agent, (2) automated code review on push, (3) issue triage with Claude Code, (4) release notes generation. Provide these as ECC skills or commands that users can install via `ecc install`. Reference anthropics/claude-code-action for official patterns. Include documentation on customization and security considerations.

## Acceptance Criteria

- [ ] At least 3 workflow templates created
- [ ] Templates installable via `ecc install`
- [ ] Documentation on customization
- [ ] Security review of template permissions
- [ ] Example usage in README or getting-started guide
