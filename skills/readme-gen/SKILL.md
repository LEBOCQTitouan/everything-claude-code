---
name: readme-gen
description: Generate and synchronise README.md from codebase state — project description, setup, commands, structure, and badges.
origin: ECC
---

# README Generation and Sync

Generation skill for producing and maintaining `README.md` that stays synchronised with the actual codebase state. Covers project description, installation, usage, structure, and contribution guidelines.

## When to Activate

- During doc-orchestrator Phase 4 (README Sync)
- When creating a new project's README
- When project structure has changed significantly
- After adding new commands, agents, or major features

## Methodology

### 1. Source of Truth Discovery

Scan the codebase for authoritative data:

| README Section | Source of Truth |
|---------------|----------------|
| Project description | `package.json` description, or first paragraph of existing README |
| Installation | `package.json` scripts, `Dockerfile`, CI config |
| Commands/Scripts | `package.json` scripts, `commands/*.md` |
| Project structure | Actual directory listing via `ls`/`glob` |
| Test count | Latest test output or test file count |
| Agent/skill count | File count in `agents/`, `skills/` directories |
| Dependencies | `package.json` dependencies |
| License | `LICENSE` file |

### 2. Section Generation

Generate each section by reading the source of truth:

**Project Header:**
```markdown
# [Project Name]

[Description from package.json or existing README]

[Badges: build status, test count, license, version]
```

**Installation:**
```markdown
## Installation

\```bash
[Detected from package.json scripts, Dockerfile, or Makefile]
\```
```

**Usage / Commands:**
```markdown
## Commands

| Command | Description |
|---------|-------------|
[Read from commands/*.md frontmatter]
```

**Project Structure:**
```markdown
## Project Structure

\```
[Actual directory tree, depth 2, with descriptions]
\```
```

**Scripts Table:**
```markdown
## npm Scripts

| Script | Description |
|--------|-------------|
[Read from package.json scripts with descriptions]
```

### 3. Sync Strategy

When updating an existing README:

1. **Preserve manual content**: Identify sections written by humans (no `<!-- AUTO-GENERATED -->` markers)
2. **Update generated sections**: Replace content between `<!-- AUTO-GENERATED -->` and `<!-- END AUTO-GENERATED -->` markers
3. **Add missing sections**: Append sections that exist in the template but not in the current README
4. **Never remove sections**: If a section exists in the README but not in the template, leave it
5. **Validate links**: Check that all internal links (`[text](path)`) resolve

### 4. Marker Format

Use HTML comments to mark auto-generated sections:

```markdown
<!-- AUTO-GENERATED:commands -->
| Command | Description |
|---------|-------------|
| `/spec` | Spec-driven planning |
| `/verify` | Run quality checks |
<!-- END AUTO-GENERATED:commands -->
```

Sections between markers are regenerated on each sync. Content outside markers is preserved.

### 5. Quality Checks

Before writing the updated README:

- [ ] All referenced files/directories exist
- [ ] All commands in the table match actual `commands/*.md` files
- [ ] Script names match `package.json`
- [ ] Test count is current (if mentioned)
- [ ] No broken internal links
- [ ] No duplicate sections

## Output

- **File**: `README.md` (project root)
- **Approach**: In-place update preserving manual sections
- **Commit**: `docs: sync README with current project state`

## Related

- Doc orchestrator: `agents/doc-orchestrator.md`
- Doc updater agent: `agents/doc-updater.md`
- Doc guidelines: `skills/doc-guidelines/SKILL.md`
- CLAUDE.md validation: `agents/doc-validator.md` (Step 7)
