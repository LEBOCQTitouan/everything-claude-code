# Custom Diagrams

Diagrams listed here are regenerated on every `/doc-suite` or `/doc-diagrams` run.
The diagram-generator reads the files in **Source Context** to understand the domain, then produces the Mermaid diagram.

To add a new diagram: append a row to the table below and run `/doc-diagrams`.

| File | Type | Title | Source Context | Description |
|------|------|-------|---------------|-------------|
| agent-orchestration.md | flowchart | Agent Orchestration | agents/*.md, commands/plan.md, commands/orchestrate.md, rules/common/agents.md, rules/common/development-workflow.md | Full development flow: /plan with TDD execution, E2E assessment, and mandatory code review |
| tdd-workflow.md | flowchart | TDD Workflow | commands/tdd.md, agents/tdd-guide.md, skills/tdd-workflow/SKILL.md | RED-GREEN-REFACTOR cycle with coverage gates and uncle-bob review |
| security-review.md | flowchart | Security Review | agents/security-reviewer.md, agents/code-reviewer.md, skills/security-review/SKILL.md | Code review pipeline: security, clean code, and quality checklists |
| feature-development.md | sequence | Feature Development | commands/plan.md, commands/tdd.md, commands/code-review.md, rules/common/development-workflow.md | Feature lifecycle from /plan through design, implementation, and review |
| refactoring.md | flowchart | Refactoring | agents/refactor-cleaner.md, commands/refactor-clean.md, skills/tdd-workflow/SKILL.md | Safe refactoring flow with test baseline, incremental removal, and rollback |
