# Custom Diagrams

Diagrams listed here are regenerated on every `/doc-suite` or `/doc-diagrams` run.
The diagram-generator reads the files in **Source Context** to understand the domain, then produces the Mermaid diagram.

To add a new diagram: append a row to the table below and run `/doc-diagrams`.

| File | Type | Title | Source Context | Description |
|------|------|-------|---------------|-------------|
| agent-orchestration.md | flowchart | Agent Orchestration | agents/*.md, commands/plan.md, rules/common/agents.md, rules/common/development-workflow.md | Full development flow: /plan with TDD execution, E2E assessment, and mandatory code review |
| tdd-workflow.md | flowchart | TDD Workflow | commands/plan.md, agents/tdd-guide.md, skills/tdd-workflow/SKILL.md | RED-GREEN-REFACTOR cycle with coverage gates and uncle-bob review |
| security-review.md | flowchart | Security Review | agents/security-reviewer.md, agents/code-reviewer.md, skills/security-review/SKILL.md | Code review pipeline: security, clean code, and quality checklists |
| feature-development.md | sequence | Feature Development | commands/plan.md, commands/verify.md, rules/common/development-workflow.md | Feature lifecycle from /plan through design, implementation, and review |
| refactoring.md | flowchart | Refactoring | agents/refactor-cleaner.md, commands/plan.md, skills/tdd-workflow/SKILL.md | Safe refactoring flow with test baseline, incremental removal, and rollback |
| cmd-plan.md | flowchart | /plan Command | commands/plan.md | Stories decomposition, planning, TDD execution, E2E, and /verify |
| cmd-verify.md | flowchart | /verify Command | commands/verify.md | Build, tests, lint, code review, arch review, coverage, dead code |
| cmd-build-fix.md | flowchart | /build-fix Command | commands/build-fix.md | Detect errors, classify, fix incrementally, verify |
| cmd-e2e.md | flowchart | /e2e Command | commands/e2e.md | Discover flows, generate Playwright tests, run, capture artifacts |
| cmd-doc-suite.md | flowchart | /doc-suite Command | commands/doc-suite.md | Plan, sync, analyze, generate, validate, diagrams, coverage |
| cmd-audit.md | flowchart | /audit Command | commands/audit.md | 7 parallel domain audits, cross-correlation, report |
| cmd-backlog.md | flowchart | /backlog Command | commands/backlog.md | Add, challenge, optimize, store, list, promote |
| cmd-uncle-bob-audit.md | flowchart | /uncle-bob-audit Command | commands/uncle-bob-audit.md | Invoke robert agent, oath evaluation, rework ratio, self-audit |
