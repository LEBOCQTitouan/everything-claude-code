# Bounded Contexts

The `ecc-domain` crate contains 9 domain modules, each responsible for a distinct bounded context. All modules are pure business logic with zero I/O.

## Module Map

| Module | Responsibility | Key Types |
|--------|---------------|-----------|
| `claw` | NanoClaw REPL session management — conversation branching, search, export, compaction | `Session`, `Turn`, `Branch` |
| `config` | Configuration model — artifact types, language groups, hook profiles, merge options | `ArtifactType`, `LanguageGroup`, `HookProfile`, `MergeOptions` |
| `detection` | Content detection — identify artifact types, validate structure, generate reports | `DetectionResult`, `ValidationReport` |
| `diff` | Diff computation — LCS algorithm, line-level diffs, merge conflict representation | `LineDiff`, `DiffResult`, `MergeConflict` |
| `hook_runtime` | Hook execution model — profile gating, flag checking, stdin passthrough protocol | `HookConfig`, `HookFlags`, `ExecutionGate` |
| `session` | Session lifecycle — metadata parsing, alias management, statistics | `SessionMetadata`, `SessionAlias`, `SessionStats` |
| `backlog` | Backlog management — entry parsing, ID generation, duplicate detection, lock file lifecycle, status reconciliation | `BacklogEntry`, `BacklogStatus`, `BacklogError`, `LockFile`, `DuplicateCandidate` |
| `workflow` | Workflow state machine — phase transitions, state persistence, toolchain config | `WorkflowState`, `Phase`, `Concern`, `Timestamp`, `Completion` |
| `cartography` | Cartography document model — delta merge, schema validation, staleness detection, coverage calculation, element registry, cross-reference matrix | `SessionDelta`, `ProjectType`, `ChangedFile`, `CartographyMeta`, `CoverageReport`, `ElementEntry`, `ElementType`, `CrossReferenceMatrix` |

## Cross-Module Dependencies

```
claw ──> session (uses session metadata)
config ──> detection (artifact type classification)
hook_runtime ──> config (reads hook profiles)
```

All other modules are independent.

## Relationship to Ports

Each domain module defines its business logic in terms of data structures and pure functions. When I/O is needed, the `ecc-ports` crate defines traits (e.g., `FileSystem`, `ShellExecutor`) that the `ecc-app` layer injects.

See also: [Glossary](glossary.md) | [Architecture](../ARCHITECTURE.md)

### Sources

Knowledge reference registry with Technology Radar vocabulary. Owns `docs/sources.md`. Independent bounded context — no dependencies on or from other domain modules. Entries organized by quadrants (Adopt/Trial/Assess/Hold) with subject-to-module mapping for command integrations. Value objects: `SourceUrl` (validated HTTP/HTTPS URL newtype), `SourceType`, `Quadrant`. Aggregate root: `SourcesRegistry`.

### Workflow Templates

Installable GitHub Actions workflow YAML files that users copy to their project's `.github/workflows/` directory. Distinct from skills (Markdown knowledge for Claude Code) and commands (slash commands). Customized via environment variables at runtime, not via file modification. Distributed via `/scaffold-workflows` slash command. Content type: `.yml` files in `workflow-templates/`.

### Pre-Hydration

Deterministic context injection before ECC commands. UserPromptSubmit hook pre-fetches project type, workflow state, and per-command context (spec: git+backlog, design: spec summary, implement: design summary+tests). Tool subsetting recommendations per command type. Lives in `hook_runtime` bounded context as a specialized handler.

### Audit Web

Profile management for `/audit-web` command. Owns `docs/audits/audit-web-profile.yaml`. Independent bounded context. Aggregate: `AuditWebProfile` (versioned YAML with dimensions, thresholds, improvement history). Value objects: `AuditDimension` (with sanitized query templates), `DimensionThreshold`. Report validation: `ReportValidationResult` with section/score/citation checks.

### Memory

Three-tier memory system (working/episodic/semantic) for cross-session knowledge persistence. Owns `~/.ecc/memory/memory.db`. Independent bounded context. Value objects: `MemoryId` (integer newtype), `MemoryTier` (enum). Domain services: `jaccard_3gram_similarity`, `compute_relevance_score`, `contains_likely_secret`.
