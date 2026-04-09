# Cartography System Element Catalog

**Type**: System Element  
**Category**: Documentation Infrastructure  
**Layer**: Meta (doc generation and state tracking)

## Overview

The cartography system is a session-aware documentation pipeline that tracks code changes and generates journey and flow documentation. It operates in two phases:

1. **Delta Collection** (session:start / session:stop hooks)
2. **Delta Processing** (/doc-suite --phase=cartography)

## Components

### Session Delta Collection

**Trigger**: `session:start` and `session:stop` hooks

**Handlers**:
- `start_cartography()` - Scans for pending deltas and prints reminder
- `stop_cartography()` - Detects changed files and writes delta JSON

**Output**: `.claude/cartography/pending-delta-<session_id>.json`

**Data Structure**:
```rust
SessionDelta {
    session_id: String,        // e.g., "session-1775719305-19260"
    timestamp: u64,            // Unix seconds
    changed_files: Vec<ChangedFile>,
    project_type: ProjectType, // Rust|JavaScript|TypeScript|Python|Go|Java|Unknown
}

ChangedFile {
    path: String,              // relative to project root
    classification: String,    // crate/package/directory name
}
```

### File Classification Rules

**Rust Projects**:
- `crates/<crate_name>/` → classification = `<crate_name>`
- Example: `crates/ecc-domain/src/lib.rs` → `ecc-domain`

**JavaScript/TypeScript Projects**:
- `packages/<package_name>/` → classification = `<package_name>`
- Example: `packages/core/src/index.ts` → `core`

**Unknown/Other Projects**:
- Top-level directory → classification
- Example: `docs/guide.md` → `docs`, `src/main.rb` → `src`

### Journey vs. Flow Determination

**Journey Targets** (command handlers, CLI entrypoints, user-facing operations):
- Changes in `commands/`
- Changes in `agents/`
- Changes in `hooks/`

**Flow Targets** (cross-module data flows, integrations):
- Changes spanning multiple crates (Rust)
- Changes spanning multiple packages (JavaScript/TypeScript)
- Changes across different top-level directories (Unknown)

### Slug Derivation Algorithm

For file path construction:

1. Convert to lowercase
2. Replace non-alphanumeric characters with hyphens
3. Collapse multiple hyphens to single hyphens
4. Truncate at 60 characters
5. Strip leading/trailing hyphens

Example: `Phase Gate Worktree State Fix` → `phase-gate-worktree-state-fix`

## Processing Pipeline

```
pending-delta-*.json
    ↓
[cartography-processing skill]
    ├─ Parse & validate
    ├─ Determine targets
    ├─ Dispatch cartographer agent (per delta)
    └─ Collect results
    ↓
[doc-orchestrator]
    ├─ Write to docs/cartography/
    ├─ Single git commit
    └─ Archive to processed/
```

## Output Structure

**Journeys**: `docs/cartography/journeys/<slug>.md`
- Command handlers and CLI entrypoints
- Step-by-step user workflows
- Entry and exit conditions

**Flows**: `docs/cartography/flows/<slug>.md`
- Cross-module data flows
- Integration points
- Module boundaries and dependencies

**Elements**: `docs/cartography/elements/<slug>.md`
- System components and services
- Configuration structures
- Reusable patterns

## State Management

Processed deltas are archived to `.claude/cartography/processed/` to enable idempotent re-entry. The processing lock at `.claude/cartography/cartography-merge.lock` prevents concurrent processing.

## Related Systems

- **Doc Orchestrator**: Phase 1.5 of the /doc-suite pipeline
- **Session Lifecycle Hooks**: Triggered by session:start and session:stop
- **Git Delta Detection**: Uses `git diff --name-only HEAD`
- **Project Type Detection**: Analyzes build files at project root
