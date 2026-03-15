<!-- Generated: 2026-03-15 | Crates: 6 | Files: 109 .rs -->

# Data Structures & Storage

## No Database

ECC uses file-based storage only -- no database, no external services.

## Core Data Structures (Rust)

### ECC Manifest (`.ecc-manifest.json`)

```rust
// ecc-domain::config::manifest
pub struct EccManifest {
    pub version: String,           // ECC version at install time
    pub installed_at: String,      // ISO 8601 timestamp
    pub updated_at: String,        // ISO 8601 timestamp
    pub languages: Vec<String>,    // ["typescript", "golang"]
    pub artifacts: Artifacts,
}

pub struct Artifacts {
    pub agents: Vec<String>,       // ["planner.md", "architect.md"]
    pub commands: Vec<String>,     // ["plan.md", "verify.md"]
    pub skills: Vec<String>,       // ["tdd-workflow"]
    pub rules: BTreeMap<String, Vec<String>>, // { "common": ["coding-style.md"] }
    pub hook_descriptions: Vec<String>,
}
```

### Merge Report

```rust
// ecc-domain::config::merge
pub struct MergeReport {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<String>,
    pub smart_merged: Vec<String>,
    pub errors: Vec<String>,
}

// ecc-app::merge
pub enum ReviewChoice { Accept, Keep, SmartMerge }
```

### Audit Types

```rust
// ecc-domain::config::audit
pub enum Severity { Critical, High, Medium, Low }

pub struct AuditFinding {
    pub id: String,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub fix: String,
}

pub struct AuditReport {
    pub checks: Vec<AuditCheckResult>,
    pub score: i32,      // 0-100
    pub grade: String,   // A/B/C/D/F
}
```

### Detection Types

```rust
// ecc-domain::detection
pub enum Language { Rust, TypeScript, JavaScript, Python, Go, Java, /* ... */ }
pub enum Framework { React, NextJs, Django, Spring, /* ... */ }
pub enum PackageManager { Npm, Pnpm, Yarn, Bun, Cargo, Pip, /* ... */ }
```

### Session Aliases

```rust
// ecc-domain::session::aliases
pub struct AliasesData {
    pub version: String,
    pub aliases: BTreeMap<String, AliasEntry>,
    pub metadata: AliasMetadata,
}

pub struct AliasEntry {
    pub session_path: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub title: Option<String>,
}
```

### Hook Types

```rust
// ecc-app::hook
pub struct HookContext {
    pub hook_id: String,
    pub stdin_payload: String,
    pub profiles_csv: Option<String>,
}

pub struct HookResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,  // 0=pass, 2=block
}
```

### Port Traits

```rust
// ecc-ports
pub trait FileSystem: Send + Sync { /* 13 methods */ }
pub trait ShellExecutor: Send + Sync { /* 4 methods */ }
pub trait Environment: Send + Sync { /* 5 methods */ }
pub trait TerminalIO: Send + Sync { /* 5 methods */ }
pub trait StdinReader: Send + Sync { /* 2 methods */ }
pub trait ReplInput: Send + Sync { /* 1 method */ }
```

## Configuration Files

| File | Format | Location |
|------|--------|----------|
| `hooks.json` | JSON | Package root |
| `settings.json` | JSON | `~/.claude/` |
| `.ecc-manifest.json` | JSON | `~/.claude/` |
| `session-aliases.json` | JSON | `~/.claude/claw/` |
| `CLAUDE.md` | Markdown | Project root |
| Agent/Command files | Markdown (YAML frontmatter) | `~/.claude/{agents,commands}/` |
| Skill directories | Markdown (`SKILL.md`) | `~/.claude/skills/` |
| Rule files | Markdown | `~/.claude/rules/{group}/` |
| Session files | Markdown | `~/.claude/claw/sessions/` |
