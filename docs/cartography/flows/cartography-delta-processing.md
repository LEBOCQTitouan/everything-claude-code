# Cartography Delta Processing Flow

**Type**: Control & Data Flow  
**Module**: `ecc-app` → `ecc-domain`  
**Layer**: App (hook handlers) → Domain (cartography types)  
**Session**: session-1775467984-85077 (timestamp: 1775467984)

## Overview

The cartography delta processing flow handles the session lifecycle for journey and flow documentation generation. It spans two hooks (session start and stop) and integrates file change detection with structured cartography delta files.

## Flow Phases

### Phase 1: Session Start (start:cartography hook)

**Handler**: `start_cartography()` in `delta_reminder.rs`

**Trigger**: `session:start` hook event

**Steps**:
1. Resolve project root from `CLAUDE_PROJECT_DIR` environment variable (falls back to CWD)
2. Scan `.claude/cartography/` for `pending-delta-*.json` files
3. Count existing pending deltas
4. If count > 0, print reminder to stderr: `"N pending cartography deltas — run /doc-suite --phase=cartography to process"`
5. Passthrough stdin unchanged

**Exit Condition**: Always succeeds with exit code 0

**Ports Used**:
- `FileSystem`: `read_dir()` to list pending deltas
- `Environment`: `var("CLAUDE_PROJECT_DIR")` for project location

### Phase 2: Session End (stop:cartography hook)

**Handler**: `stop_cartography()` in `delta_writer.rs`

**Trigger**: `session:stop` hook event

**Steps**:

#### 2a. Detect Changed Files
1. Get project directory from `CLAUDE_PROJECT_DIR` environment variable
2. Run `git diff --name-only HEAD` in project directory
3. If git error or "not a git repository": passthrough (no delta written)
4. If no changed files: passthrough (no delta written)

#### 2b. Detect Project Type
1. Use `crate::detection::framework::detect_project_type(fs, &project_dir)`
2. Map detected type to `cartography::ProjectType` enum:
   - `Cargo.toml` at root → `ProjectType::Rust`
   - `package.json` + `tsconfig.json` → `ProjectType::Typescript`
   - `package.json` only → `ProjectType::Javascript`
   - Other build files → `ProjectType::Python|Go|Java`
   - No recognized file → `ProjectType::Unknown`

#### 2c. Classify Changed Files
1. Get session ID from `CLAUDE_SESSION_ID` environment variable
2. If absent, generate fallback ID: `"session-<timestamp>-<pid>"`
3. For each changed file path:
   - Call `classify_file(path, &project_type)`
   - Classification rules:
     - **Rust**: Extract crate name from `crates/<crate>/` prefix (e.g., `crates/ecc-domain/src/lib.rs` → `ecc-domain`)
     - **JS/TS**: Extract package path (e.g., `packages/core/src/index.ts` → `core`)
     - **Unknown/Other**: Use top-level directory (e.g., `docs/guide.md` → `docs`, `src/main.rb` → `src`)
4. Create `ChangedFile { path: String, classification: String }` for each file

#### 2d. Build and Write Delta
1. Create `SessionDelta` struct:
   ```rust
   SessionDelta {
       session_id: String,
       timestamp: u64 (Unix seconds),
       changed_files: Vec<ChangedFile>,
       project_type: ProjectType,
   }
   ```
2. Serialize to JSON via `serde_json::to_string_pretty()`
3. Create `.claude/cartography/` directory if missing
4. Clean up corrupt existing delta files via `clean_corrupt_deltas()`
5. Write delta to `.claude/cartography/pending-delta-<session_id>.json`

**Exit Condition**: Always passthrough with exit code 0 (non-blocking)

**Ports Used**:
- `Environment`: `var("CLAUDE_PROJECT_DIR")`, `var("CLAUDE_SESSION_ID")`
- `ShellExecutor`: `run_command_in_dir("git", ["diff", "--name-only", "HEAD"], &project_dir)`
- `FileSystem`: `create_dir_all()`, `write()`, `read_dir()`, `read_to_string()` (for cleanup)

## Data Structures

### SessionDelta (ecc-domain)
```rust
pub struct SessionDelta {
    pub session_id: String,
    pub timestamp: u64,
    pub changed_files: Vec<ChangedFile>,
    pub project_type: ProjectType,
}
```

### ChangedFile (ecc-domain)
```rust
pub struct ChangedFile {
    pub path: String,                 // relative from project root
    pub classification: String,        // crate/package/directory name
}
```

### ProjectType (ecc-domain)
```rust
pub enum ProjectType {
    Rust,
    Javascript,
    Typescript,
    Python,
    Go,
    Java,
    Unknown,
}
```

## Module Boundaries

- **ecc-app** (hook handlers): Orchestrates detection, file classification, and delta serialization
- **ecc-domain** (cartography types): Pure value types with serde derives; no I/O
- **ecc-ports** (FileSystem, ShellExecutor, Environment traits): Abstraction layer for OS interaction
- **ecc-infra** (concrete adapters): Real filesystem, git, environment variable implementations

## Error Handling

| Scenario | Behavior |
|----------|----------|
| CLAUDE_PROJECT_DIR not set | Passthrough (no delta) |
| Git repository error | Passthrough (no delta) |
| No changed files | Passthrough (no delta) |
| Corrupt existing delta | Deleted, current delta written |
| JSON serialization error | Passthrough with warning (no delta) |
| Directory creation error | Passthrough with warning (no delta) |
| File write error | Passthrough with warning (no delta) |

## Integration Points

1. **Session Lifecycle**: Hooks are triggered by `session:start` and `session:stop` events
2. **Project Detection**: Delegates to `ecc_app::detection::framework::detect_project_type()`
3. **File Classification**: Uses `classify_file()` from `ecc_domain::cartography`
4. **Delta Processing**: Downstream `/doc-suite --phase=cartography` consumes pending deltas

## Testing

Comprehensive test coverage in both handlers:

**stop_cartography tests** (5 test cases):
- PC-008: No delta when no changes
- PC-009: Writes delta for Rust project with crate classification
- PC-010: Project type variants (Rust, TypeScript, JavaScript, Unknown) + fallback session ID
- PC-011: No git repo → passthrough; corrupt JSON → deleted + current delta written

**start_cartography tests** (3 test cases):
- PC-016: Prints pending count and /doc-suite hint when deltas exist
- PC-017: Silent passthrough when no pending deltas
- PC-018: Falls back to CWD when CLAUDE_PROJECT_DIR absent

## Related Flows

- **Journey**: `/doc-suite --phase=cartography` processes pending deltas into journey/flow/element documentation
- **System**: Part of the broader `/doc-suite` phased documentation pipeline (Phase 1.5)
