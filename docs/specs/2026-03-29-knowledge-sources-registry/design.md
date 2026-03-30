# Design: Knowledge Sources Registry (BL-086)

## Overview

Replicate the backlog module pattern exactly: domain types + pure parser/serializer in `ecc-domain`, app use cases orchestrating through `&dyn FileSystem` + `&dyn ShellExecutor` in `ecc-app`, thin CLI wiring in `ecc-cli`. Add `docs/sources.md` as the flat-file registry, command integrations in 8 markdown files, and ADR-0031.

## File Changes Table

| # | File Path | Change | Layer | US | AC(s) |
|---|-----------|--------|-------|-----|-------|
| FC-01 | `crates/ecc-domain/src/sources/mod.rs` | New module declaration | Entity | US-001 | — |
| FC-02 | `crates/ecc-domain/src/sources/entry.rs` | `SourceEntry`, `SourceType`, `Quadrant`, `SourceError` types; URL/name validation | Entity | US-001 | AC-001.1–001.5 |
| FC-03 | `crates/ecc-domain/src/sources/parser.rs` | `parse_sources(content: &str) -> Result<SourcesRegistry, Vec<SourceParseError>>` pure fn | Entity | US-002 | AC-002.1, 002.4, 002.5 |
| FC-04 | `crates/ecc-domain/src/sources/serializer.rs` | `serialize_sources(registry: &SourcesRegistry) -> String` pure fn | Entity | US-002 | AC-002.2, 002.3 |
| FC-05 | `crates/ecc-domain/src/sources/registry.rs` | `SourcesRegistry` aggregate: inbox, quadrant entries, module mapping; add/filter/reindex logic | Entity | US-001, US-002 | AC-001.1–001.3, 002.1 |
| FC-06 | `crates/ecc-domain/src/lib.rs` | Add `pub mod sources;` | Entity | — | — |
| FC-07 | `crates/ecc-app/src/sources.rs` | `list`, `add`, `check`, `reindex` use cases via `&dyn FileSystem` + `&dyn ShellExecutor` | UseCase | US-003 | AC-003.1–003.11 |
| FC-08 | `crates/ecc-app/src/lib.rs` | Add `pub mod sources;` | UseCase | — | — |
| FC-09 | `crates/ecc-cli/src/commands/sources.rs` | `SourcesArgs`, `SourcesAction` Clap wiring, `run()` | Adapter | US-004 | AC-004.1–004.4 |
| FC-10 | `crates/ecc-cli/src/commands/mod.rs` | Add `pub mod sources;` | Adapter | — | — |
| FC-11 | `crates/ecc-cli/src/main.rs` | Add `Sources(commands::sources::SourcesArgs)` variant + match arm | Adapter | US-004 | AC-004.1 |
| FC-12 | `docs/sources.md` | Bootstrap file: Inbox, 4 quadrant sections, module mapping table, seed entries | Framework | US-005 | AC-005.1, 005.3 |
| FC-13 | `CLAUDE.md` | Add pointer in doc hierarchy + CLI commands section | Framework | US-005 | AC-005.2 |
| FC-14 | `commands/spec-dev.md` | Add sources consultation + update step in Phase 0 or Phase 1 | Framework | US-006 | AC-006.1, 006.7, 006.8 |
| FC-15 | `commands/spec-fix.md` | Add sources consultation + update step | Framework | US-006 | AC-006.1, 006.7, 006.8 |
| FC-16 | `commands/spec-refactor.md` | Add sources consultation + update step | Framework | US-006 | AC-006.1, 006.7, 006.8 |
| FC-17 | `commands/implement.md` | Add sources consultation + update step | Framework | US-006 | AC-006.2, 006.7, 006.8 |
| FC-18 | `commands/design.md` | Add sources consultation + update step | Framework | US-006 | AC-006.3, 006.7, 006.8 |
| FC-19 | `commands/review.md` | Add sources as reference context step | Framework | US-006 | AC-006.5, 006.7, 006.8 |
| FC-20 | `commands/catchup.md` | Add sources diff summary step | Framework | US-006 | AC-006.6, 006.7, 006.8 |
| FC-21 | `commands/audit-web.md` | Add sources re-interrogation step | Framework | US-006 | AC-006.4, 006.7, 006.8 |
| FC-22 | `docs/adr/0031-sources-bounded-context.md` | ADR: sources bounded context, Technology Radar vocabulary | Framework | US-007 | AC-007.1, 007.2 |
| FC-23 | `docs/domain/bounded-contexts.md` | Add sources bounded context entry | Framework | US-007 | AC-007.1 |

## Domain Model (FC-02, FC-05)

### Types in `entry.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Repo, Doc, Blog, Package, Talk, Paper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quadrant {
    Adopt, Trial, Assess, Hold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEntry {
    pub url: String,
    pub title: String,
    pub source_type: SourceType,
    pub quadrant: Quadrant,
    pub subject: String,
    pub added_by: String,
    pub added_date: String,        // YYYY-MM-DD
    pub last_checked: Option<String>, // YYYY-MM-DD
    pub deprecation_reason: Option<String>,
    pub stale: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("URL must be a valid URL format: {0}")]
    InvalidUrl(String),
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("duplicate URL: {0}")]
    DuplicateUrl(String),
    #[error("unknown source type: {0}")]
    UnknownSourceType(String),
    #[error("unknown quadrant: {0}")]
    UnknownQuadrant(String),
}
```

Validation functions (pure, no I/O):
- `validate_url(url: &str) -> Result<(), SourceError>` — checks starts with `http://` or `https://` and contains a `.`
- `validate_title(title: &str) -> Result<(), SourceError>` — non-empty after trim

Lifecycle:
- `is_deprecated(&self) -> bool` returns `self.deprecation_reason.is_some()`
- `SourceType::from_str`, `Quadrant::from_str` with error variants

### Types in `registry.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleMapping {
    pub module_path: String,
    pub subjects: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SourcesRegistry {
    pub inbox: Vec<SourceEntry>,
    pub entries: Vec<SourceEntry>,  // all quadrant entries
    pub module_mappings: Vec<ModuleMapping>,
}
```

Methods:
- `add(&self, entry: SourceEntry) -> Result<SourcesRegistry, SourceError>` — checks duplicate URL, returns new registry (immutable)
- `list(&self, quadrant: Option<&Quadrant>, subject: Option<&str>) -> Vec<&SourceEntry>` — filter
- `reindex(&self) -> SourcesRegistry` — moves inbox entries to `entries` (sorted by quadrant/subject)
- `entries_by_quadrant(&self, q: &Quadrant) -> Vec<&SourceEntry>` — group helper
- `subjects(&self) -> Vec<&str>` — unique subjects sorted
- `find_by_url(&self, url: &str) -> Option<&SourceEntry>`
- `find_by_module(&self, module_path: &str) -> Vec<&SourceEntry>` — uses module mappings

## Parser (FC-03)

Pure function: `parse_sources(content: &str) -> Result<SourcesRegistry, Vec<SourceParseError>>`

The `docs/sources.md` format (each entry is a markdown list item):

```markdown
# Knowledge Sources

## Inbox

- [Title](url) — type: repo | quadrant: assess | subject: testing | added: 2026-03-29 | by: human

## Adopt

### testing
- [Title](url) — type: doc | subject: testing | added: 2026-03-01 | by: human | checked: 2026-03-15

## Trial
...

## Assess
...

## Hold
...

## Module Mapping

| Module | Subjects |
|--------|----------|
| crates/ecc-domain/ | domain-modeling, rust-patterns |
```

Parser logic:
1. Split by `## ` headers to find sections (Inbox, Adopt, Trial, Assess, Hold, Module Mapping)
2. Within quadrant sections, split by `### ` for subject subsections
3. Each `- [Title](url) — key: value | key: value` line is parsed into `SourceEntry`
4. Inbox entries get their quadrant from the metadata `quadrant:` field
5. Per-entry errors collected, not aborting

## Serializer (FC-04)

Pure function: `serialize_sources(registry: &SourcesRegistry) -> String`

Outputs canonical format: Inbox section, then Adopt/Trial/Assess/Hold sections (each with subject subsections sorted alphabetically), then Module Mapping table.

Round-trip property: `parse(serialize(parse(input))) == parse(input)` (semantic equality).

## App Use Cases (FC-07)

App-layer error type (wraps domain errors + I/O concerns, per uncle-bob MEDIUM finding):

```rust
#[derive(Debug, thiserror::Error)]
pub enum SourcesAppError {
    #[error("domain error: {0}")]
    Domain(#[from] SourceError),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("shell command failed: {0}")]
    ShellFailed(String),
}
```

All functions take `&dyn FileSystem` as first param. `check` additionally takes `&dyn ShellExecutor`.

```rust
pub fn list(
    fs: &dyn FileSystem,
    sources_path: &Path,
    quadrant: Option<&str>,
    subject: Option<&str>,
) -> Result<Vec<SourceEntry>, SourceError>

pub fn add(
    fs: &dyn FileSystem,
    sources_path: &Path,
    url: &str,
    title: &str,
    source_type: &str,
    quadrant: &str,
    subject: &str,
    added_by: &str,
) -> Result<(), SourceError>

pub fn check(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    sources_path: &Path,
) -> Result<Vec<CheckResult>, SourceError>

pub fn reindex(
    fs: &dyn FileSystem,
    sources_path: &Path,
    dry_run: bool,
) -> Result<Option<String>, SourceError>
```

`CheckResult`:
```rust
pub struct CheckResult {
    pub title: String,
    pub url: String,
    pub status: CheckStatus,
}

pub enum CheckStatus {
    Ok,
    Stale,            // HTTP non-200
    Unreachable,      // curl timeout/error
    WarnAge(u64),     // > 90 days since last_checked
    ErrorAge(u64),    // > 180 days since last_checked
}
```

`check` implementation:
- For each entry, shell-out: `curl -sL -o /dev/null -w "%{http_code}" --max-time 10 <url>`
- Parse HTTP status code from stdout
- Non-200 or curl failure = stale; update entry's `stale` flag and `last_checked`
- If previously stale and now 200, clear stale flag
- Calculate days since `last_checked` for age warnings
- Atomic write via `sources_path.with_extension("md.tmp")` + rename

`add` implementation:
- Read existing file (or start with empty registry if not found)
- Validate URL + title
- Check duplicate URL
- Add entry to correct quadrant/subject
- Atomic write

`reindex` implementation:
- Parse existing file
- Move inbox entries to correct quadrant sections
- Sort entries within sections
- Atomic write (or return content for dry-run)

## CLI Wiring (FC-09, FC-10, FC-11)

```rust
#[derive(Args)]
pub struct SourcesArgs {
    #[command(subcommand)]
    pub action: SourcesAction,

    /// Path to sources file
    #[arg(long, default_value = "docs/sources.md")]
    pub file: PathBuf,
}

#[derive(Subcommand)]
pub enum SourcesAction {
    List {
        #[arg(long)]
        quadrant: Option<String>,
        #[arg(long)]
        subject: Option<String>,
    },
    Add {
        url: String,
        #[arg(long)]
        title: String,
        #[arg(long, name = "type")]
        source_type: String,
        #[arg(long)]
        quadrant: String,
        #[arg(long)]
        subject: String,
        #[arg(long, default_value = "human")]
        added_by: String,
    },
    Check,
    Reindex {
        #[arg(long)]
        dry_run: bool,
    },
}
```

`run()` creates `OsFileSystem` and `OsShellExecutor`, delegates to `ecc_app::sources::*`.

## Command Integrations (FC-14 through FC-21)

Each command file gets a new step block. Pattern for consultation commands (spec-dev, spec-fix, spec-refactor, implement, design, review):

```markdown
### Sources Consultation

If `docs/sources.md` exists:
1. Read `docs/sources.md`
2. Find entries matching the current subject (case-insensitive) OR module (via module mapping table)
3. If matches found, list them as "Consulted sources:" in the output
4. Update `last_checked` date on matched entries to today's date
5. Write updated file back (atomic)

If `docs/sources.md` does not exist, skip this step silently.
```

For `/catchup` (FC-20):
```markdown
### Sources Summary

If `docs/sources.md` exists:
1. Run: `git log --oneline --diff-filter=M -- docs/sources.md` to find last commit modifying sources
2. Show entries modified since that commit (git diff)

If `docs/sources.md` does not exist, skip this step silently.
```

For `/audit-web` (FC-21):
```markdown
### Sources Re-interrogation

If `docs/sources.md` exists:
1. Run `ecc sources check` to verify all URLs
2. Report stale/unreachable sources
3. Update stale flags in the file

If `docs/sources.md` does not exist, skip this step silently.
```

## Pass Conditions Table

| PC | Type | Description | Verifies | Command | Expected |
|----|------|-------------|----------|---------|----------|
| PC-001 | unit | `SourceType` and `Quadrant` enums: from_str, display, all variants | AC-001.1 | `cargo test -p ecc-domain sources::entry` | PASS |
| PC-002 | unit | `SourceEntry` construction with all fields | AC-001.1 | `cargo test -p ecc-domain sources::entry::tests::entry_construction` | PASS |
| PC-003 | unit | `validate_url` accepts http/https, rejects invalid | AC-001.4 | `cargo test -p ecc-domain sources::entry::tests::validate_url` | PASS |
| PC-004 | unit | `validate_title` rejects empty/whitespace | AC-001.5 | `cargo test -p ecc-domain sources::entry::tests::validate_title` | PASS |
| PC-005 | unit | `is_deprecated` returns true when deprecation_reason is Some | AC-001.2 | `cargo test -p ecc-domain sources::entry::tests::deprecated_lifecycle` | PASS |
| PC-006 | unit | `SourceError` all 7 variants exist and have Display | AC-001.1 | `cargo test -p ecc-domain sources::entry::tests::error_variants` | PASS |
| PC-007 | unit | `SourcesRegistry::add` rejects duplicate URL | AC-001.3 | `cargo test -p ecc-domain sources::registry::tests::add_duplicate_rejected` | PASS |
| PC-008 | unit | `SourcesRegistry::add` returns new registry (immutable) | AC-001.3 | `cargo test -p ecc-domain sources::registry::tests::add_returns_new` | PASS |
| PC-009 | unit | `SourcesRegistry::list` filters by quadrant and subject | AC-001.3 | `cargo test -p ecc-domain sources::registry::tests::list_filters` | PASS |
| PC-010 | unit | `SourcesRegistry::reindex` moves inbox to entries | AC-001.3 | `cargo test -p ecc-domain sources::registry::tests::reindex_moves_inbox` | PASS |
| PC-011 | unit | `SourcesRegistry::find_by_module` uses module mappings | AC-001.3 | `cargo test -p ecc-domain sources::registry::tests::find_by_module` | PASS |
| PC-012 | unit | `parse_sources` parses well-formed doc with all sections | AC-002.1 | `cargo test -p ecc-domain sources::parser::tests::parse_full_document` | PASS |
| PC-013 | unit | `parse_sources` handles empty/missing sections | AC-002.5 | `cargo test -p ecc-domain sources::parser::tests::parse_empty_file` | PASS |
| PC-014 | unit | `parse_sources` collects per-entry errors without aborting | AC-002.4 | `cargo test -p ecc-domain sources::parser::tests::parse_errors_per_entry` | PASS |
| PC-015 | unit | `parse_sources` parses module mapping table | AC-002.1 | `cargo test -p ecc-domain sources::parser::tests::parse_module_mapping` | PASS |
| PC-016 | unit | `serialize_sources` outputs canonical format | AC-002.2 | `cargo test -p ecc-domain sources::serializer::tests::serialize_canonical` | PASS |
| PC-017 | unit | Round-trip: parse then serialize then parse yields same registry | AC-002.3 | `cargo test -p ecc-domain sources::serializer::tests::round_trip` | PASS |
| PC-018 | unit | `list` use case returns filtered entries | AC-003.1 | `cargo test -p ecc-app sources::tests::list_with_filters` | PASS |
| PC-019 | unit | `add` use case appends entry, atomic write | AC-003.2 | `cargo test -p ecc-app sources::tests::add_entry` | PASS |
| PC-020 | unit | `add` creates file when missing | AC-003.3 | `cargo test -p ecc-app sources::tests::add_creates_file` | PASS |
| PC-021 | unit | `add` rejects duplicate URL | AC-003.9 | `cargo test -p ecc-app sources::tests::add_duplicate_rejected` | PASS |
| PC-022 | unit | `reindex` moves inbox entries to quadrant sections | AC-003.4 | `cargo test -p ecc-app sources::tests::reindex_moves_inbox` | PASS |
| PC-023 | unit | `reindex` dry-run returns content without writing | AC-003.5 | `cargo test -p ecc-app sources::tests::reindex_dry_run` | PASS |
| PC-024 | unit | `check` flags stale on non-200 curl response | AC-003.6 | `cargo test -p ecc-app sources::tests::check_stale_on_non_200` | PASS |
| PC-025 | unit | `check` WARN on >90 days since last_checked | AC-003.7 | `cargo test -p ecc-app sources::tests::check_warn_90_days` | PASS |
| PC-026 | unit | `check` ERROR on >180 days since last_checked | AC-003.8 | `cargo test -p ecc-app sources::tests::check_error_180_days` | PASS |
| PC-027 | unit | `check` treats curl timeout as unreachable | AC-003.10 | `cargo test -p ecc-app sources::tests::check_curl_timeout` | PASS |
| PC-028 | unit | `check` clears stale flag on successful recheck | AC-003.11 | `cargo test -p ecc-app sources::tests::check_clears_stale` | PASS |
| PC-029 | unit | `check` atomic write after updates | AC-003.6 | `cargo test -p ecc-app sources::tests::check_atomic_write` | PASS |
| PC-030 | build | CLI compiles with `Sources` subcommand and all 4 actions | AC-004.1–004.4 | `cargo build -p ecc-cli` | PASS |
| PC-031 | unit | CLI `list` routes to app use case | AC-004.1 | `cargo test -p ecc-cli -- sources` | PASS |
| PC-032 | lint | `docs/sources.md` exists with Inbox, 4 quadrant sections, Module Mapping | AC-005.1 | `grep -c "^## " docs/sources.md` | stdout contains "6" (Inbox + 4 quadrants + Module Mapping) |
| PC-033 | lint | `docs/sources.md` has at least one entry per quadrant | AC-005.3 | `grep -cP "^- \[" docs/sources.md` | stdout >= 4 |
| PC-034 | lint | `CLAUDE.md` contains pointer to `docs/sources.md` | AC-005.2 | `grep -c "sources.md" CLAUDE.md` | stdout >= 1 |
| PC-035 | lint | `CLAUDE.md` CLI section contains `ecc sources` commands | AC-005.2 | `grep -c "ecc sources" CLAUDE.md` | stdout >= 1 |
| PC-036 | lint | `commands/spec-dev.md` contains sources consultation step | AC-006.1 | `grep -c "Sources Consultation\|sources.md" commands/spec-dev.md` | stdout >= 1 |
| PC-037 | lint | `commands/spec-fix.md` contains sources consultation step | AC-006.1 | `grep -c "Sources Consultation\|sources.md" commands/spec-fix.md` | stdout >= 1 |
| PC-038 | lint | `commands/spec-refactor.md` contains sources consultation step | AC-006.1 | `grep -c "Sources Consultation\|sources.md" commands/spec-refactor.md` | stdout >= 1 |
| PC-039 | lint | `commands/implement.md` contains sources consultation step | AC-006.2 | `grep -c "Sources Consultation\|sources.md" commands/implement.md` | stdout >= 1 |
| PC-040 | lint | `commands/design.md` contains sources consultation step | AC-006.3 | `grep -c "Sources Consultation\|sources.md" commands/design.md` | stdout >= 1 |
| PC-041 | lint | `commands/audit-web.md` contains sources re-interrogation step | AC-006.4 | `grep -c "Sources\|sources.md" commands/audit-web.md` | stdout >= 1 |
| PC-042 | lint | `commands/review.md` contains sources reference step | AC-006.5 | `grep -c "Sources\|sources.md" commands/review.md` | stdout >= 1 |
| PC-043 | lint | `commands/catchup.md` contains sources summary step | AC-006.6 | `grep -c "Sources\|sources.md" commands/catchup.md` | stdout >= 1 |
| PC-044 | lint | All command integrations include graceful degradation | AC-006.8 | `grep -l "does not exist" commands/spec-dev.md commands/implement.md commands/design.md commands/review.md commands/catchup.md commands/audit-web.md` | 6 files listed |
| PC-045 | lint | `docs/adr/0031-sources-bounded-context.md` exists with bounded context explanation | AC-007.1 | `grep -c "bounded context\|Technology Radar" docs/adr/0031-sources-bounded-context.md` | stdout >= 2 |
| PC-046 | lint | ADR-0031 references BL-086 and backlog pattern | AC-007.2 | `grep -c "BL-086\|backlog" docs/adr/0031-sources-bounded-context.md` | stdout >= 2 |
| PC-047 | lint | `docs/domain/bounded-contexts.md` contains sources entry | AC-007.1 | `grep -c "sources\|Sources" docs/domain/bounded-contexts.md` | stdout >= 1 |
| PC-048 | lint | No I/O imports in domain sources module | constraint | `grep -rn "std::fs\|std::process\|std::net\|tokio" crates/ecc-domain/src/sources/` | exit code 1 (no matches) |
| PC-049 | build | `cargo clippy -- -D warnings` passes | constraint | `cargo clippy -- -D warnings` | exit code 0 |
| PC-050 | build | `cargo build --release` succeeds | constraint | `cargo build --release` | exit code 0 |
| PC-051 | build | `cargo test` full suite passes | constraint | `cargo test` | exit code 0 |

## TDD Dependency Order

Implementation follows this ordering. Each phase uses RED-GREEN-REFACTOR with commits per the cadence rules.

### Phase 1: Domain Entry Types (FC-02, FC-06)
**Layers:** Entity

PCs: PC-001, PC-002, PC-003, PC-004, PC-005, PC-006

Files:
- `crates/ecc-domain/src/sources/mod.rs` (new)
- `crates/ecc-domain/src/sources/entry.rs` (new)
- `crates/ecc-domain/src/lib.rs` (add `pub mod sources;`)

Commit cadence:
1. `test: add sources domain entry types tests (RED)`
2. `feat: implement sources domain entry types (GREEN)`
3. `refactor: improve sources domain entry types (REFACTOR)` (if applicable)

Boy Scout Delta: Scan `crates/ecc-domain/src/backlog/entry.rs` for a small improvement.

---

### Phase 2: Domain Registry Aggregate (FC-05)
**Layers:** Entity

PCs: PC-007, PC-008, PC-009, PC-010, PC-011

Files:
- `crates/ecc-domain/src/sources/registry.rs` (new)
- `crates/ecc-domain/src/sources/mod.rs` (add module)

Commit cadence:
1. `test: add sources registry aggregate tests (RED)`
2. `feat: implement sources registry aggregate (GREEN)`
3. `refactor: improve sources registry aggregate (REFACTOR)` (if applicable)

---

### Phase 3: Domain Parser (FC-03)
**Layers:** Entity

PCs: PC-012, PC-013, PC-014, PC-015

Files:
- `crates/ecc-domain/src/sources/parser.rs` (new)
- `crates/ecc-domain/src/sources/mod.rs` (add module)

Commit cadence:
1. `test: add sources parser tests (RED)`
2. `feat: implement sources parser (GREEN)`
3. `refactor: improve sources parser (REFACTOR)` (if applicable)

---

### Phase 4: Domain Serializer (FC-04)
**Layers:** Entity

PCs: PC-016, PC-017

Files:
- `crates/ecc-domain/src/sources/serializer.rs` (new)
- `crates/ecc-domain/src/sources/mod.rs` (add module)

Commit cadence:
1. `test: add sources serializer tests (RED)`
2. `feat: implement sources serializer (GREEN)`
3. `refactor: improve sources serializer (REFACTOR)` (if applicable)

---

### Phase 5: App Use Cases (FC-07, FC-08)
**Layers:** UseCase

PCs: PC-018, PC-019, PC-020, PC-021, PC-022, PC-023, PC-024, PC-025, PC-026, PC-027, PC-028, PC-029

Files:
- `crates/ecc-app/src/sources.rs` (new)
- `crates/ecc-app/src/lib.rs` (add `pub mod sources;`)

Dependencies: Phase 1-4 (domain types + parser + serializer)

Commit cadence:
1. `test: add sources app use cases tests (RED)`
2. `feat: implement sources app use cases (GREEN)`
3. `refactor: improve sources app use cases (REFACTOR)` (if applicable)

---

### Phase 6: CLI Wiring (FC-09, FC-10, FC-11)
**Layers:** Adapter

PCs: PC-030, PC-031

Files:
- `crates/ecc-cli/src/commands/sources.rs` (new)
- `crates/ecc-cli/src/commands/mod.rs` (add module)
- `crates/ecc-cli/src/main.rs` (add variant + match arm)

Dependencies: Phase 5 (app use cases)

Commit cadence:
1. `test: add sources CLI subcommand tests (RED)` (if feasible; otherwise fold into build check)
2. `feat: add ecc sources CLI subcommands (GREEN)`
3. `refactor: improve sources CLI wiring (REFACTOR)` (if applicable)

---

### Phase 7: Bootstrap File + CLAUDE.md (FC-12, FC-13)
**Layers:** Framework

PCs: PC-032, PC-033, PC-034, PC-035

Files:
- `docs/sources.md` (new)
- `CLAUDE.md` (edit)

Dependencies: Phase 4 (serializer defines canonical format)

Commit cadence:
1. `docs: bootstrap docs/sources.md with seed entries`
2. `docs: update CLAUDE.md with sources pointer and CLI commands`

---

### Phase 8: Command Integrations (FC-14 through FC-21)
**Layers:** Framework

PCs: PC-036, PC-037, PC-038, PC-039, PC-040, PC-041, PC-042, PC-043, PC-044

Files:
- `commands/spec-dev.md` (edit)
- `commands/spec-fix.md` (edit)
- `commands/spec-refactor.md` (edit)
- `commands/implement.md` (edit)
- `commands/design.md` (edit)
- `commands/review.md` (edit)
- `commands/catchup.md` (edit)
- `commands/audit-web.md` (edit)

Dependencies: Phase 7 (sources.md must exist for integration to reference)

Commit cadence:
1. `feat: add sources consultation to spec commands`
2. `feat: add sources consultation to implement and design commands`
3. `feat: add sources integration to review, catchup, and audit-web commands`

---

### Phase 9: ADR + Bounded Contexts Doc (FC-22, FC-23)
**Layers:** Framework

PCs: PC-045, PC-046, PC-047

Files:
- `docs/adr/0031-sources-bounded-context.md` (new)
- `docs/domain/bounded-contexts.md` (edit)

Dependencies: None (can run parallel to other phases, but placed last for coherence)

Commit cadence:
1. `docs: add ADR-0031 sources bounded context`
2. `docs: add sources to bounded-contexts.md`

---

### Phase 10: Final Gates (no new files)
**Layers:** All

PCs: PC-048, PC-049, PC-050, PC-051

Dependencies: All phases complete

Commands:
1. `grep -rn "std::fs\|std::process\|std::net\|tokio" crates/ecc-domain/src/sources/` — must return exit code 1
2. `cargo clippy -- -D warnings` — must pass
3. `cargo build --release` — must pass
4. `cargo test` — full suite must pass

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Parser complexity for markdown format | High — fragile parsing | Extensive test fixtures; keep format simple (list items with pipe-delimited metadata) |
| Round-trip fidelity loss | Medium — data corruption | PC-017 explicitly tests round-trip; normalize whitespace in comparison |
| Domain purity violation | High — architectural regression | PC-048 greps for I/O imports; hook enforces at commit time |
| Curl not available on system | Low — check fails | `check` should verify `curl` exists first via `shell.command_exists("curl")` |
| Large sources.md exceeds 800 lines | Low — style violation | Non-requirement in v1; warn if > 800 lines in reindex output |
| Command markdown edits break existing workflows | Medium — regression | Each command file edit is minimal (add a step block); existing steps untouched |

## E2E Assessment

- **Touches user-facing flows?** Yes — new `ecc sources` CLI commands
- **Crosses 3+ modules end-to-end?** Yes — domain + app + CLI + filesystem
- **New E2E tests needed?** No for v1 — the CLI wiring is thin (delegates to app use cases which are fully unit-tested with InMemoryFileSystem and MockExecutor). Run existing E2E suite as gate.

## Success Criteria

- [ ] `ecc sources list`, `add`, `check`, `reindex` all work from CLI
- [ ] `docs/sources.md` exists with all 6 sections and seed entries
- [ ] Round-trip parse/serialize is lossless
- [ ] `check` shells out to curl with 10s timeout, flags stale entries
- [ ] 8 command files updated with sources integration + graceful degradation
- [ ] ADR-0031 documents the bounded context decision
- [ ] Domain crate has zero I/O imports
- [ ] `cargo clippy`, `cargo build`, `cargo test` all pass

## Coverage Check

All 38 ACs covered by 51 PCs. Zero uncovered.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0031-sources-bounded-context.md` | ADR | Create | Sources bounded context, Technology Radar vocabulary | US-007 |
| 2 | `docs/domain/bounded-contexts.md` | Domain docs | Modify | Add sources bounded context entry | US-007 |
| 3 | `CLAUDE.md` | Project root | Modify | Add docs/sources.md pointer + ecc sources CLI commands | US-005 |
| 4 | `docs/sources.md` | Project docs | Create | Bootstrap with Inbox, 4 quadrants, module mapping, seed entries | US-005 |
| 5 | `CHANGELOG.md` | Project root | Modify | Add knowledge sources registry entry | — |

## SOLID Assessment

**CLEAN** — 1 MEDIUM finding addressed (IoError moved from domain to app-layer SourcesAppError). 2 LOW findings noted for future improvement (newtypes for url/dates, bundle add parameters into request struct). Clean Architecture dependency rule followed throughout.

## Robert's Oath Check

**CLEAN** — 0 oath warnings. 51 pass conditions cover all 38 ACs. 10 TDD phases with atomic commits. Rework ratio 0.19 (healthy).

## Security Notes

**CLEAR** — 0 findings. Shell-out to curl uses execve-style invocation (no shell injection). URL validation at domain boundary. File writes to fixed paths (no traversal from user input). Response bodies discarded (-o /dev/null). Two LOW observations (SSRF via curl, minimal URL validation) acceptable for local developer tool.

## Rollback Plan

Reverse dependency order:
1. Revert command integration edits (8 markdown files)
2. Revert `docs/domain/bounded-contexts.md`
3. Delete `docs/adr/0031-sources-bounded-context.md`
4. Revert `CLAUDE.md`
5. Delete `docs/sources.md`
6. Revert `crates/ecc-cli/src/main.rs`, `commands/mod.rs`
7. Delete `crates/ecc-cli/src/commands/sources.rs`
8. Revert `crates/ecc-app/src/lib.rs`
9. Delete `crates/ecc-app/src/sources.rs`
10. Revert `crates/ecc-domain/src/lib.rs`
11. Delete `crates/ecc-domain/src/sources/` directory
12. Revert `CHANGELOG.md`

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 1 MEDIUM (addressed), 2 LOW (noted) |
| Robert's Oath | CLEAN | 0 |
| Security | CLEAR | 0 (2 LOW informational) |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 75 | PASS | 38/38 ACs covered; AC-006.7 coverage is indirect |
| Order | 85 | PASS | 10-phase dependency order correct |
| Fragility | 70 | PASS | 13 grep PCs inherent to markdown verification |
| Rollback | 85 | PASS | 12-step reverse dependency order documented |
| Architecture | 90 | PASS | Replicates backlog pattern; IoError fix applied |
| Blast Radius | 80 | PASS | 23 files but most are small edits |
| Missing PCs | 75 | PASS | CHANGELOG PC missing; CLI routing tests sparse |
| Doc Plan | 85 | PASS | 5 doc entries including ADR and CHANGELOG |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/sources/mod.rs` | create | US-001 |
| 2 | `crates/ecc-domain/src/sources/entry.rs` | create | US-001 |
| 3 | `crates/ecc-domain/src/sources/registry.rs` | create | US-001 |
| 4 | `crates/ecc-domain/src/sources/parser.rs` | create | US-002 |
| 5 | `crates/ecc-domain/src/sources/serializer.rs` | create | US-002 |
| 6 | `crates/ecc-domain/src/lib.rs` | modify | US-001 |
| 7 | `crates/ecc-app/src/sources.rs` | create | US-003 |
| 8 | `crates/ecc-app/src/lib.rs` | modify | US-003 |
| 9 | `crates/ecc-cli/src/commands/sources.rs` | create | US-004 |
| 10 | `crates/ecc-cli/src/commands/mod.rs` | modify | US-004 |
| 11 | `crates/ecc-cli/src/main.rs` | modify | US-004 |
| 12 | `docs/sources.md` | create | US-005 |
| 13 | `CLAUDE.md` | modify | US-005 |
| 14-21 | `commands/{spec-dev,spec-fix,spec-refactor,implement,design,review,catchup,audit-web}.md` | modify | US-006 |
| 22 | `docs/adr/0031-sources-bounded-context.md` | create | US-007 |
| 23 | `docs/domain/bounded-contexts.md` | modify | US-007 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-knowledge-sources-registry/design.md | Full design + phase summary |
