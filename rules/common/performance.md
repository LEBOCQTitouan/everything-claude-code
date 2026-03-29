# Performance Optimization

## Model Selection Strategy

> Per Anthropic guidance: "Start with Sonnet, route only the most demanding to Opus."

**Haiku 4.5** ($1/$5 per MTok — diff-based detection, simple extraction):
- Diff-based detection and staleness checks (drift-checker)
- Diagram generation and doc formatting
- Web research per-category workers (web-radar-analyst)

**Sonnet 4.6** ($3/$15 per MTok — Code review, audit checks, orchestration, TDD):
- Language-specific code review (python, go, rust, typescript, java, kotlin, cpp, csharp, shell, database reviewers)
- Checklist-based audit agents (error-handling, convention, observability, test auditors)
- Documentation validation and orchestration (doc-validator, doc-orchestrator, web-scout)
- TDD execution, build resolution, refactoring, E2E testing
- Backlog curation

**Opus 4.6** ($5/$25 per MTok — Architecture decisions, security, adversarial reasoning):
- Architecture design and review (architect, architect-module, arch-reviewer, uncle-bob)
- Security vulnerability analysis (security-reviewer)
- Adversarial spec/solution review (spec-adversary, solution-adversary)
- Complex multi-phase planning (planner, requirements-analyst)
- Code review orchestration (code-reviewer)
- Professional conscience audit (robert)
- Design exploration (interviewer, interface-designer)
- Audit orchestration (audit-orchestrator)

## Context Window Management

Avoid last 20% of context window for:
- Large-scale refactoring
- Feature implementation spanning multiple files
- Debugging complex interactions

Lower context sensitivity tasks:
- Single-file edits
- Independent utility creation
- Documentation updates
- Simple bug fixes

## Extended Thinking + Plan Mode

Extended thinking is enabled by default, reserving up to 31,999 tokens for internal reasoning.

Control extended thinking via:
- **Toggle**: Option+T (macOS) / Alt+T (Windows/Linux)
- **Config**: Set `alwaysThinkingEnabled` in `~/.claude/settings.json`
- **Budget cap**: `export MAX_THINKING_TOKENS=10000`
- **Verbose mode**: Ctrl+O to see thinking output

For complex tasks requiring deep reasoning:
1. Ensure extended thinking is enabled (on by default)
2. Enable **Plan Mode** for structured approach
3. Use multiple critique rounds for thorough analysis
4. Use split role sub-agents for diverse perspectives

## Build Troubleshooting

If build fails:
1. Use **build-error-resolver** agent
2. Analyze error messages
3. Fix incrementally
4. Verify after each fix
