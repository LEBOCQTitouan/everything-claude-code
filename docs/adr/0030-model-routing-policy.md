# ADR 0030: Agent Model Routing Policy

## Status

Accepted

## Context

32 of 51 ECC agents (62.7%) were assigned to Opus, but Anthropic's official guidance recommends Sonnet as the default for most work, reserving Opus for complex multi-step reasoning. Many agents perform checklist-based audits, pattern matching, or code review that Sonnet handles equally well. This inflated cost by ~75% per invocation for tasks that don't benefit from Opus-level reasoning.

Cost ratio: Haiku $1/$5, Sonnet $3/$15, Opus $5/$25 per MTok.

## Decision

Adopt a three-tier model routing policy aligned with Anthropic's official guidance:

**Haiku** — Diff-based detection, simple extraction, diagram generation. Agents: drift-checker, diagram-generator, doc-reporter, doc-updater, doc-generator, module-summary-updater, diagram-updater, web-radar-analyst.

**Sonnet** — Code review, language-specific review, audit checks, orchestration, TDD. Agents: all 10 language-specific reviewers (python, go, rust, typescript, java, kotlin, cpp, csharp, shell, database), error-handling-auditor, convention-auditor, observability-auditor, test-auditor, doc-validator, doc-orchestrator, web-scout, tdd-executor, tdd-guide, build-error-resolver, go-build-resolver, kotlin-build-resolver, e2e-runner, refactor-cleaner, backlog-curator.

**Opus** — Architecture decisions, security review, adversarial reasoning, planning, design exploration. Agents: code-reviewer, security-reviewer, architect, architect-module, uncle-bob, arch-reviewer, robert, spec-adversary, solution-adversary, planner, requirements-analyst, interviewer, interface-designer, audit-orchestrator, doc-analyzer, harness-optimizer, evolution-analyst, component-auditor.

## Consequences

- Estimated 30-40% cost reduction on review-heavy sessions
- 14 agents moved: 1 opus→haiku (drift-checker), 13 opus→sonnet (10 reviewers + doc-validator + web-scout + doc-orchestrator)
- 4 agents deferred for future evaluation: doc-analyzer, harness-optimizer, evolution-analyst, component-auditor
- Zero quality regression expected for code review (Anthropic-confirmed: Sonnet handles coding tasks)
- Policy documented in `rules/common/performance.md` for ongoing reference
