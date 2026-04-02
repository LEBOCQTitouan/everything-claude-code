# Spec: BL-119 GitHub Workflow Templates for Claude Code Integration

## Problem Statement

Developers want standardized CI/CD integration patterns with Claude Code but must build workflows from scratch. Community repos (anthropics/claude-code-action, shinpr/claude-code-workflows) and blog posts demonstrate strong demand, and ECC is positioned to provide reference templates as first-class installable content. Currently, ECC has a `github-actions` skill with general CI/CD patterns but no installable workflow YAML files or Claude Code-specific automation templates.

## Research Summary

- **`anthropics/claude-code-action@v1`** is the official GitHub Action -- supports `claude_args` for model/tools/prompt customization, interactive and automation triggers
- **Five proven community recipes** exist: PR review, issue-to-PR via `@claude`, doc updates, test generation, release notes -- all use `paths`/`paths-ignore` filters to prevent infinite loops
- **Security best practices**: least-privilege `permissions`, `pull_request` (not `pull_request_target`) for fork safety, `--allowedTools` to restrict Claude's CI capabilities, never log `ANTHROPIC_API_KEY`
- **Fork safety is built-in**: `pull_request` trigger prevents fork PRs from accessing secrets; two-workflow pattern (`workflow_run`) enables controlled fork review
- **Tool restriction is critical**: `--allowedTools Read,Grep,Glob` for review-only; add `Edit,Write` only for generation workflows
- **Key pitfalls**: missing `fetch-depth: 0` for git history, no `paths-ignore` causing trigger loops, deprecated top-level inputs vs `claude_args`, `write-all` permissions shortcuts
- **Claude Code Review (managed)** is a higher-level alternative dispatching specialized agents per PR -- 84% of large PRs receive findings averaging 7.5 issues

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | New `workflow-templates/` top-level content directory | Workflow YAML files are a distinct content type that doesn't fit skills (Markdown), commands, or examples. Clean separation enables dedicated validation. | Yes |
| 2 | `/scaffold-workflows` slash command for distribution | `ecc install` targets `~/.claude/` (global) but templates need `<project>/.github/workflows/` (project-local). A slash command avoids Rust changes for v1 while leveraging Claude Code's file-writing capabilities. | Yes |
| 3 | Merge `github-actions` + new content into comprehensive CI/CD skill | Avoids duplication between general CI/CD patterns and Claude Code workflow specifics. Single source of truth. | No |
| 4 | Merge "PR review" and "code review on push" into single PR review template; replace with CI convention linter | Push-triggered review has no PR to post comments on. A convention linter is more useful and differentiating. | No |
| 5 | Two-workflow pattern for fork PR support | OSS repos need fork PR review. Safe trigger + `workflow_run` is the established pattern. | No |
| 6 | Full integration testing with `act` (nektos/act) | Validates workflow execution locally for high-fidelity testing beyond YAML parsing. | No |

## User Stories

### US-001: Claude Code PR Review Workflow Template

**As a** developer using ECC, **I want** a reusable GitHub Actions workflow template that runs Claude Code as a PR reviewer, **so that** every pull request gets AI-assisted code review automatically.

#### Acceptance Criteria

- AC-001.1: Given a file `workflow-templates/claude-pr-review.yml`, when a user copies it to `.github/workflows/`, then it triggers on `pull_request` events (opened, synchronize) and runs Claude Code review via `anthropics/claude-code-action@v1`
- AC-001.2: Given the workflow is triggered, when the PR has code changes, then Claude Code posts a single PR review comment summarizing findings with inline annotations on specific changed files, using `--allowedTools Read,Grep,Glob`
- AC-001.3: Given the workflow template, when inspected for security, then permissions are `contents: read`, `pull-requests: write`, secrets use `${{ secrets.ANTHROPIC_API_KEY }}`, and `pull_request_target` is NOT used
- AC-001.4: Given a fork PR, when the workflow triggers, then it gracefully skips (no secrets available) with a logged message
- AC-001.5: Given a two-workflow variant `workflow-templates/claude-pr-review-fork-safe.yml`, when a fork PR is opened, then the safe trigger collects the diff and `workflow_run` processes it with secrets
- AC-001.6: Given environment variables, when the user customizes review scope, then `CLAUDE_MODEL`, `CLAUDE_MAX_TURNS`, `CLAUDE_ALLOWED_TOOLS`, and `CLAUDE_PROMPT` are documented in YAML comments and in the CI/CD skill
- AC-001.7: Given `ANTHROPIC_API_KEY` is not configured as a repository secret, when the workflow runs on a non-fork PR, then it fails fast within 30 seconds with a clear error message
- AC-001.8: Given the workflow template, when inspected, then it includes a `concurrency` group (`claude-review-${{ github.event.pull_request.number }}`) to prevent duplicate reviews on the same PR

#### Dependencies

- Depends on: none

### US-002: Claude Code Issue Triage Workflow Template

**As a** project maintainer, **I want** a workflow template that uses Claude Code to auto-label and triage new issues, **so that** issues are categorized consistently without manual effort.

#### Acceptance Criteria

- AC-002.1: Given a file `workflow-templates/claude-issue-triage.yml`, when a user copies it to `.github/workflows/`, then it triggers on `issues: [opened]` events
- AC-002.2: Given a new issue is opened, when the workflow runs, then Claude Code analyzes the issue body and applies labels from the default taxonomy: `bug`, `feature`, `question`, `docs`, `enhancement`, `security`, `performance`
- AC-002.3: Given the workflow template, when inspected for permissions, then it uses `issues: write`, `contents: read` and nothing more
- AC-002.4: Given a configurable label taxonomy, when the user sets `LABEL_TAXONOMY` env var, then custom labels are used instead of defaults
- AC-002.5: Given an issue with an empty or non-textual body (image-only), when the workflow runs, then it applies a `needs-triage` label and skips AI analysis

#### Dependencies

- Depends on: none

### US-003: Claude Code Release Notes Generation Workflow Template

**As a** project maintainer, **I want** a workflow template that generates release notes using Claude Code, **so that** releases have well-written changelogs without manual effort.

#### Acceptance Criteria

- AC-003.1: Given a file `workflow-templates/claude-release-notes.yml`, when a tag matching `v*` is pushed, then the workflow generates release notes from commit history since the previous tag
- AC-003.2: Given the workflow runs, when generating notes, then Claude Code categorizes commits (features, fixes, breaking changes, docs, chores) and produces human-readable summaries
- AC-003.3: Given the generated notes, when a GitHub Release is created/updated, then the notes are attached to the release body
- AC-003.4: Given permissions, when inspected, then `contents: write` is the only write permission
- AC-003.5: Given this is the first release (no previous tag exists), when the workflow runs, then it generates notes from the full commit history with a truncation limit of 500 commits

#### Dependencies

- Depends on: none

### US-004: Claude Code CI Convention Linter Workflow Template

**As a** developer using ECC, **I want** a workflow template that uses Claude Code to enforce project conventions in CI, **so that** PRs are checked for naming conventions, CHANGELOG updates, commit message format, and custom rules.

#### Acceptance Criteria

- AC-004.1: Given a file `workflow-templates/claude-ci-linter.yml`, when a PR is opened/synchronized, then Claude Code checks the PR against configurable convention rules
- AC-004.2: Given the convention check, when violations are found, then Claude posts specific inline comments identifying the violation and suggesting a fix
- AC-004.3: Given a `CONVENTION_RULES` env var or `.claude-lint-rules` file, when the user defines custom rules, then those rules are enforced in addition to defaults; when both exist, `.claude-lint-rules` file takes precedence and the env var is ignored
- AC-004.4: Given permissions, when inspected, then `contents: read`, `pull-requests: write` are the only permissions

#### Dependencies

- Depends on: none

### US-005: /scaffold-workflows Slash Command

**As a** developer using ECC, **I want** a `/scaffold-workflows` slash command that installs selected workflow templates into my project, **so that** I don't have to manually copy YAML files.

#### Acceptance Criteria

- AC-005.1: Given the command `commands/scaffold-workflows.md` exists, when a user runs `/scaffold-workflows`, then they are prompted to select which templates to install via AskUserQuestion
- AC-005.2: Given the user selects templates, when the command executes, then selected `.yml` files are written to `<project>/.github/workflows/`; no variable substitution is performed -- templates are copied verbatim since all customization is via environment variables at runtime
- AC-005.3: Given workflow files already exist at the target, when the command runs, then the user is warned and asked whether to overwrite
- AC-005.4: Given `--dry-run` argument, when the command executes, then it shows what would be written without modifying files

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

### US-006: Comprehensive CI/CD Workflows Skill

**As a** developer using ECC, **I want** a unified skill covering both general CI/CD patterns and Claude Code workflow specifics, **so that** Claude Code has complete context when helping with CI/CD tasks.

#### Acceptance Criteria

- AC-006.1: Given a new skill `skills/ci-cd-workflows/SKILL.md`, when loaded by Claude Code, then it contains H2 sections for: "General CI/CD Patterns", "Claude Code Templates", "Security", "Fork Safety", "Customization"
- AC-006.2: Given the existing `skills/github-actions/SKILL.md`, when the new skill is created, then the old skill directory is retained as a deprecated redirect (single line: "Moved to `skills/ci-cd-workflows/`. This redirect will be removed in a future release.") for one release cycle before removal
- AC-006.3: Given the skill frontmatter, when validated, then it has `name: ci-cd-workflows`, `description`, `origin: ECC` fields
- AC-006.4: Given references in other files to `skills/github-actions/`, when the skill is renamed, then all cross-references are updated

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

### US-007: Validation and Testing

**As a** developer, **I want** workflow templates to be validated and tested with `act`, **so that** template quality is assured.

#### Acceptance Criteria

- AC-007.1: Given `ecc validate workflow-templates` is not yet implemented in Rust, when validation is needed, then a validation script or CI step checks: YAML parses, required keys (`name`, `on`, `permissions`, `jobs`), pinned action versions, least-privilege permissions
- AC-007.2: Given `act` is installed, when structural integration tests run, then each workflow template is verified for syntax, job ordering, and step execution with mocked action outputs; tests do NOT require a live Anthropic API key
- AC-007.3: Given the CI workflow `ci.yml`, when the validation step runs, then workflow templates are included in the content validation suite (structural YAML validation only; `act` execution is a local-only gate, not a CI requirement)
- AC-007.4: Given each workflow template, when inspected, then it includes a header comment `# ECC Template vX.Y.Z` with the ECC version that generated it, for future drift detection

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `workflow-templates/` | Content (new) | 5 new YAML files (4 templates + 1 fork-safe variant) |
| `commands/scaffold-workflows.md` | Content (new) | New slash command |
| `skills/ci-cd-workflows/` | Content (new) | Merged comprehensive skill replacing `github-actions` |
| `skills/github-actions/` | Content (removed) | Replaced by `ci-cd-workflows` |
| `skills/github-actions-rust/` | Content (existing) | Update cross-reference to new skill name |
| `rules/ecc/github-actions.md` | Content (existing) | Update cross-reference to new skill name |
| `docs/adr/` | Docs (new) | 2 new ADRs |
| `docs/domain/bounded-contexts.md` | Docs (update) | Add "workflow template" concept |
| `CLAUDE.md` | Docs (update) | Add `/scaffold-workflows` to command list |

## Constraints

- No Rust code changes in this spec -- content-only
- Templates must use `anthropics/claude-code-action@v1` (pinned major version)
- All templates must have `permissions` set to least-privilege (never default/write-all)
- Fork-safe variant must never use `pull_request_target` with unchecked code checkout
- `act` testing requires Docker -- document as optional for CI, mandatory for local dev testing

## Non-Requirements

- No `ecc init --with-workflows` CLI integration (future backlog item)
- No Rust-based `ecc validate workflow-templates` command (use script/CI for v1)
- No Claude Code Review managed service integration (different product)
- No Slack/Discord notification templates
- No self-hosted runner templates

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Content install pipeline | New content type | `workflow-templates/` must be included in release tarball bundling |
| `/scaffold-workflows` command | New slash command | Claude Code reads templates and writes to project `.github/workflows/` |
| `ecc validate` | No change for v1 | Validation via script, not Rust -- future backlog item |
| CI pipeline | Extended | Add workflow template validation step to `ci.yml` |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New content type | HIGH | CLAUDE.md | Add `/scaffold-workflows` command, mention `workflow-templates/` |
| New skill | MEDIUM | CLAUDE.md | Update skill reference from `github-actions` to `ci-cd-workflows` |
| ADRs | MEDIUM | `docs/adr/` | Write 2 new ADRs |
| Domain concept | LOW | `docs/domain/bounded-contexts.md` | Add "workflow template" term |
| Cross-refs | LOW | `skills/github-actions-rust/`, `rules/ecc/github-actions.md` | Update skill name references |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is explicitly OUT of scope? Merge PR+push review? | 4 templates: PR review, issue triage, release notes, CI convention linter. Push-review merged into PR review. | User |
| 2 | Where should workflow templates live in the content model? | New `workflow-templates/` top-level directory | Recommended |
| 3 | How should users install workflow templates? | `/scaffold-workflows` slash command (no Rust changes for v1) | Recommended |
| 4 | Which critical paths need 100% coverage vs 80%? | Full integration testing with `act` (nektos/act) for local validation | User |
| 5 | How should templates handle fork PRs? | Two-workflow pattern for fork support (safe trigger + `workflow_run`) | User |
| 6 | How to handle overlap with existing github-actions skill? | Merge both into new comprehensive `ci-cd-workflows` skill | User |
| 7 | Are there domain terms needing definition? | Add "workflow template" to bounded-contexts.md | Recommended |
| 8 | Which decisions warrant an ADR? | Two ADRs: content type + distribution mechanism | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Claude Code PR Review Workflow Template | 8 | none |
| US-002 | Claude Code Issue Triage Workflow Template | 5 | none |
| US-003 | Claude Code Release Notes Generation Workflow Template | 5 | none |
| US-004 | Claude Code CI Convention Linter Workflow Template | 4 | none |
| US-005 | /scaffold-workflows Slash Command | 4 | US-001, US-002, US-003, US-004 |
| US-006 | Comprehensive CI/CD Workflows Skill | 4 | US-001, US-002, US-003, US-004 |
| US-007 | Validation and Testing | 4 | US-001, US-002, US-003, US-004 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | PR review triggers on pull_request via claude-code-action@v1 | US-001 |
| AC-001.2 | Posts single PR review comment with inline annotations | US-001 |
| AC-001.3 | Least-privilege permissions, no pull_request_target | US-001 |
| AC-001.4 | Fork PRs gracefully skipped | US-001 |
| AC-001.5 | Fork-safe two-workflow variant | US-001 |
| AC-001.6 | Env var customization documented in YAML and skill | US-001 |
| AC-001.7 | Missing API key fails fast within 30s | US-001 |
| AC-001.8 | Concurrency group prevents duplicate reviews | US-001 |
| AC-002.1 | Issue triage triggers on issues opened | US-002 |
| AC-002.2 | Default label taxonomy: bug, feature, question, docs, enhancement, security, performance | US-002 |
| AC-002.3 | Permissions: issues write, contents read only | US-002 |
| AC-002.4 | Configurable label taxonomy via env var | US-002 |
| AC-002.5 | Empty/non-textual issues get needs-triage label | US-002 |
| AC-003.1 | Release notes on v* tag push | US-003 |
| AC-003.2 | Categorized commit summaries | US-003 |
| AC-003.3 | Notes attached to GitHub Release body | US-003 |
| AC-003.4 | Permissions: contents write only | US-003 |
| AC-003.5 | First release: full history with 500-commit truncation | US-003 |
| AC-004.1 | CI linter triggers on PR open/sync | US-004 |
| AC-004.2 | Inline violation comments with fix suggestions | US-004 |
| AC-004.3 | Config precedence: .claude-lint-rules > env var | US-004 |
| AC-004.4 | Permissions: contents read, pull-requests write | US-004 |
| AC-005.1 | Interactive template selection via AskUserQuestion | US-005 |
| AC-005.2 | Verbatim copy to .github/workflows/ | US-005 |
| AC-005.3 | Overwrite warning for existing files | US-005 |
| AC-005.4 | Dry-run mode | US-005 |
| AC-006.1 | Skill with 5 required H2 sections | US-006 |
| AC-006.2 | Deprecated redirect for old skill, one release cycle | US-006 |
| AC-006.3 | Correct frontmatter: ci-cd-workflows | US-006 |
| AC-006.4 | Cross-reference updates | US-006 |
| AC-007.1 | Validation script: YAML parse, required keys, pinned versions | US-007 |
| AC-007.2 | act tests: syntax + job ordering, no live API key needed | US-007 |
| AC-007.3 | CI: structural YAML validation only | US-007 |
| AC-007.4 | ECC version header comment in templates | US-007 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Ambiguity | 62 | 88 | PASS | Tightened review comment format, label taxonomy, config precedence, skill sections |
| Edge Cases | 55 | 82 | PASS | Added missing API key, empty issues, first release, concurrency, truncation |
| Testability | 65 | 83 | PASS | Scoped act testing to structural validation, CI/local distinction, section verification |
| Scope | 78 | 82 | PASS | Well-bounded content-only scope with clear non-requirements |
| Dependencies | 82 | 80 | PASS | Clean DAG, external deps documented |
| Decisions | 75 | 82 | PASS | Well-reasoned with ADR flags |
| Rollback | 85 | 78 | PASS | Content-only, deprecated redirect for backward compat |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-github-workflow-templates/spec.md` | Full spec + Phase Summary |
