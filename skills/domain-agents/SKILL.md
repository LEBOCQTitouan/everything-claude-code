---
name: domain-agents
description: Discovery and usage patterns for domain-specialized agents generated from bounded contexts
origin: ECC
---

# Domain Agents

Domain agents are generated, read-only agents in `agents/domain/` that encode deep knowledge of a specific bounded context — its types, errors, test patterns, and naming conventions.

## Directory Structure

```
agents/domain/
  backlog.md        # Backlog bounded context agent
  workflow.md       # Workflow bounded context agent
  config.md         # Config bounded context agent
  ...
```

Each file is named `<module-name>.md` where `<module-name>` matches the module name from `docs/domain/bounded-contexts.md`.

## Naming Convention

- File: `agents/domain/<module>.md` (kebab-case, matching bounded context name)
- Agent name in frontmatter: `<module>-domain` (e.g., `workflow-domain`)
- Frontmatter includes `generated: true` and `generated_at: <ISO 8601>` for staleness tracking

## Lookup Pattern

To find the domain agent for a module:

1. Read `docs/domain/bounded-contexts.md` to identify the module name
2. Check if `agents/domain/<module>.md` exists
3. If it exists, spawn as a read-only Task subagent with `allowedTools: [Read, Grep, Glob]`

For pipeline commands, Phase 0.7 (Domain Context) automates this lookup by matching feature descriptions or affected modules against agent filenames.

## Matching Algorithm

- **spec commands**: Tokenize the feature description into words, exact-match (case-insensitive) against module names from bounded-contexts.md
- **design command**: Read the spec's Affected Modules table Module column, match against `agents/domain/<module>.md`
- **implement command**: Parse the design's File Changes table crate paths, extract module names, match against agent filenames
- Cap: at most 3 domain agents per phase (first 3 alphabetically if more match)

## Generation

Run `/generate-domain-agents` to create or update domain agents. The command reads `docs/domain/bounded-contexts.md`, inspects source code, and generates structured agent files with Domain Model, Error Catalogue, Test Patterns, Cross-Module Dependencies, and Naming Conventions sections.

## Graceful Degradation

If `agents/domain/` does not exist or contains no `.md` files, all domain agent features skip silently. No errors, no warnings. Pipeline commands proceed without domain context injection. This ensures the feature is opt-in and non-breaking for repos that haven't run `/generate-domain-agents`.

## Staleness

Domain agents have a `generated_at` timestamp. Run `/generate-domain-agents --check-staleness` to detect agents that may be outdated relative to source code changes (uses `git log --since`).
