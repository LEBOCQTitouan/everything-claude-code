# ADR 0036: Comms Repo Isolation

## Status
Accepted

## Context
The comms pipeline generates public-facing content from private codebases. Generated artifacts (drafts, strategies, calendars) must not pollute the source code repository. The pipeline needs a persistent output location that survives across sessions.

## Decision
Content lives in a separate git repo (comms/). When invoked inside a code repo, the comms directory is gitignored. The agent manages the comms repo's git operations (init, commit). Strategy files live in the comms repo alongside generated content.

## Consequences
- Code repo stays clean — `git status` never shows comms artifacts
- Comms repo has its own history, independent of code commits
- Strategy files are project-specific, not baked into skills
- Agent must handle git operations in comms repo (init, commit, push)
