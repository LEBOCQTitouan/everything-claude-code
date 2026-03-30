# Bounded Contexts

The `ecc-domain` crate contains 8 domain modules, each responsible for a distinct bounded context. All modules are pure business logic with zero I/O.

## Module Map

| Module | Responsibility | Key Types |
|--------|---------------|-----------|
| `claw` | NanoClaw REPL session management — conversation branching, search, export, compaction | `Session`, `Turn`, `Branch` |
| `config` | Configuration model — artifact types, language groups, hook profiles, merge options | `ArtifactType`, `LanguageGroup`, `HookProfile`, `MergeOptions` |
| `detection` | Content detection — identify artifact types, validate structure, generate reports | `DetectionResult`, `ValidationReport` |
| `diff` | Diff computation — LCS algorithm, line-level diffs, merge conflict representation | `LineDiff`, `DiffResult`, `MergeConflict` |
| `hook_runtime` | Hook execution model — profile gating, flag checking, stdin passthrough protocol | `HookConfig`, `HookFlags`, `ExecutionGate` |
| `session` | Session lifecycle — metadata parsing, alias management, statistics | `SessionMetadata`, `SessionAlias`, `SessionStats` |
| `backlog` | Backlog management — entry parsing, ID generation, duplicate detection | `BacklogEntry`, `BacklogError`, `BacklogId` |
| `workflow` | Workflow state machine — phase transitions, state persistence, toolchain config | `WorkflowState`, `Phase`, `Concern`, `Timestamp`, `Completion` |

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
