---
description: Generate domain-specialized agents from bounded-contexts.md module definitions
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - Agent
  - AskUserQuestion
---

# Generate Domain Agents

Analyze `docs/domain/bounded-contexts.md` and source code to produce one domain-specialized agent per bounded context in `agents/domain/`.

## Arguments

- `--check-staleness` — Only check if existing agents are stale (exit 1 if stale, exit 0 if fresh). Does not regenerate.
- `--overwrite` — Overwrite existing agents without asking.

## Phase 0: Preconditions

1. Check if `docs/domain/bounded-contexts.md` exists
2. If NOT: exit with "bounded-contexts.md not found — run /project-foundation or create it manually"
3. Parse the file for module entries:
   - **Table format**: Read the `## Module Map` table. For each row, extract the Module column value (e.g., `backlog`, `workflow`, `config`)
   - **Freestanding sections**: Scan for `###` headings after the table. Each `###` heading is a module name (e.g., `### Sources`, `### Pre-Hydration`)
4. If zero modules found: exit with "No modules found in bounded-contexts.md"
5. Report: "Found N modules in bounded-contexts.md"

## Phase 1: Module Selection

Present discovered modules via AskUserQuestion:

- Option 1: "All N modules" — generate agents for every discovered module
- Option 2-N: Individual module names for selective generation
- The user selects which modules to generate agents for

## Phase 2: Source Discovery

For each selected module:

1. Check if `crates/ecc-domain/src/<module>/` directory exists
2. If NOT, check if `crates/ecc-domain/src/<module>.rs` single file exists
3. If neither exists: warn "Source not found for <module> — skipping" and continue to next module
4. Record the source path (directory or single file) for extraction

## Phase 3: Domain Extraction

For each module with a discovered source, launch a read-only analysis subagent (max 3 concurrent) with allowedTools: [Read, Grep, Glob]:

The subagent reads the module source and extracts:

a. **Public types**: All lines matching `pub struct` or `pub enum` with their names and first-line doc-comments
b. **Error variants**: All `#[error("...")]` attribute strings from thiserror error enums
c. **Test patterns**: Count of `#[test]` annotations, names of `#[cfg(test)] mod` blocks, whether `assert_eq!` or `pretty_assertions` is used
d. **Module description**: The module's description from bounded-contexts.md
e. **Cross-module dependencies**: From the `## Cross-Module Dependencies` section of bounded-contexts.md
f. **Naming conventions**: Common function prefix patterns (e.g., `resolve_`, `parse_`, `validate_`) counted by grep, constant casing style

## Phase 4: Agent Generation

For each module with extracted data:

1. Check if `agents/domain/<module>.md` already exists
2. If it exists AND `--overwrite` was NOT passed: ask "Agent for <module> already exists. Overwrite?" via AskUserQuestion
3. If user declines: skip this module

Write the agent file to `agents/domain/<module>.md` with this structure:

### Frontmatter

```yaml
---
name: <module>-domain
description: "Domain expert for the <module> bounded context — types, errors, patterns, and conventions"
model: sonnet
effort: medium
tools:
  - Read
  - Grep
  - Glob
generated: true
generated_at: "<ISO 8601 timestamp>"
---
```

### Body Sections

#### ## Domain Model

List each `pub struct` and `pub enum` name with its first-line doc-comment if present, or "no doc-comment" if absent. Format as a markdown table:

| Type | Kind | Description |
|------|------|-------------|
| WorkflowState | struct | Root aggregate for workflow state machine data |
| Phase | enum | no doc-comment |

#### ## Error Catalogue

List each `#[error("...")]` variant name and its error message string. If no thiserror error types exist in this module, write: "No thiserror error types in this module."

| Error Type | Variant | Message |
|-----------|---------|---------|
| WorkflowError | IllegalTransition | "illegal transition from {from} to {to}" |

#### ## Test Patterns

- Total `#[test]` count: N
- Test module names: `mod tests`, `mod integration_tests`, etc.
- Assertion style: `assert_eq!` / `pretty_assertions` (detected by grep)
- Test naming convention: e.g., `verb_noun_condition` pattern

#### ## Cross-Module Dependencies

List dependencies from bounded-contexts.md Cross-Module Dependencies section. If none listed, write: "Independent — no cross-module dependencies."

#### ## Naming Conventions

- Common function prefixes: `resolve_` (N occurrences), `parse_` (N), `validate_` (N)
- Constant casing: SCREAMING_SNAKE_CASE / other
- Module-specific patterns observed

### Verification

After generating each agent body, verify no placeholder text leaked:

Grep the generated file for these 5 patterns: `TODO`, `<describe>`, `<fill>`, `[step`, `[your`. If any are found, regenerate that section.

## Phase 4.5: Staleness Check

For each existing agent in `agents/domain/` that has a `generated_at` frontmatter field:

1. Run `git log --since=<generated_at> -- crates/ecc-domain/src/<module>/` to count commits since generation
2. If commits found: warn "Agent for <module> may be stale — N commits since generation"
3. If no commits: report "Agent for <module> is up-to-date"

### `--check-staleness` mode

When invoked with `--check-staleness` (and without `--overwrite`):

1. Only perform staleness checks — do NOT regenerate any agents
2. Check all existing agents in `agents/domain/`
3. If any are stale: exit 1 (enables CI-level staleness gating)
4. If all are fresh: exit 0

### Git unavailable

If `git` is not available or the directory is not a git repo: log "git not available — staleness check skipped" and continue without staleness checks.

## Phase 5: Commit

After all agents are generated:

1. Create `agents/domain/` directory if it doesn't exist
2. Stage all new/modified files in `agents/domain/`
3. Commit with message: `feat: generate domain agents for <module-list>`

Where `<module-list>` is the comma-separated list of generated module names.
