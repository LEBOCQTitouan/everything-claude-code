<!-- Generated: 2026-03-08 | Files scanned: 47 | Token estimate: ~400 -->

# Data Structures & Storage

## No Database

ECC uses file-based storage only — no database, no external services.

## Core Data Structures

### ECC Manifest (`.ecc-manifest.json`)

```typescript
interface EccManifest {
  version: string;           // ECC version at install time
  installedAt: string;       // ISO timestamp
  updatedAt: string;         // ISO timestamp
  languages: string[];       // ['typescript', 'golang']
  artifacts: {
    agents: string[];        // ['planner.md', 'architect.md']
    commands: string[];      // ['tdd.md', 'plan.md']
    skills: string[];        // ['tdd-workflow']
    rules: Record<string, string[]>; // { common: ['coding-style.md'] }
    hookDescriptions: string[];
  };
}
```

### Session Metadata

```typescript
interface SessionMetadata {
  id: string;
  date: string;              // YYYY-MM-DD
  startTime: string;         // HH:MM
  project: string;
  tasks: string[];
  filesModified: string[];
  toolsUsed: string[];
  stats: { totalMessages: number };
}
```

### Detection Result

```typescript
interface DetectionResult {
  agents: Array<{ filename: string; name: string | null }>;
  commands: string[];
  skills: Array<{ dirname: string; hasSkillMd: boolean }>;
  rules: Record<string, string[]>;
  hooks: Array<{ event: string; description: string }>;
  claudeMdSections: string[];
  hasSettingsJson: boolean;
}
```

### Merge Report

```typescript
interface MergeReport {
  added: string[];
  updated: string[];
  skipped: string[];
  smartMerged: string[];
  errors: string[];
}
```

### Diff Line (LCS algorithm)

```typescript
interface DiffLine {
  type: 'same' | 'removed' | 'added';
  text: string;
}
```

## Configuration Files

| File | Format | Location |
|------|--------|----------|
| `hooks.json` | JSON | Package `hooks/` dir |
| `settings.json` | JSON | `~/.claude/` |
| `.ecc-manifest.json` | JSON | `~/.claude/` |
| `session-aliases.json` | JSON | `~/.claude/` |
| `CLAUDE.md` | Markdown | Project root |
| Agent/Command/Skill files | Markdown (YAML frontmatter) | `~/.claude/` subdirs |
