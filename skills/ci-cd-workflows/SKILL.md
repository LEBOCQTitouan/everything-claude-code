---
name: ci-cd-workflows
description: CI/CD patterns, GitHub Actions best practices, and Claude Code workflow template usage for automated code review, issue triage, release notes, and convention linting.
origin: ECC
---

# CI/CD Workflows

Comprehensive guide to GitHub Actions CI/CD patterns and Claude Code workflow integration.

## General CI/CD Patterns

### Triggers
- `pull_request` for validation, `push` for post-merge. Use `paths:` filter to skip irrelevant changes.
- `on: push: tags: ['v*']` for tag-based releases.
- `schedule` with `cron:` for periodic tasks. Always add `workflow_dispatch` for manual triggers.

### Job Dependencies
- `needs: [build]` for sequential gates. `if: always()` to run cleanup regardless of failure.
- Status checks reference **job names**, not workflow filenames.

### Caching
- `actions/cache@v4` with `key: ${{ runner.os }}-${{ hashFiles('**/lockfile') }}`
- Use `restore-keys:` fallback for partial cache hits. Cache eviction: 10 GB per repo, LRU.

### Secrets
- Access: `${{ secrets.NAME }}`. Never echo secrets in `run:` blocks.
- Use **environment-level secrets** for deploy credentials (scoped, auditable).
- OIDC: Prefer `aws-actions/configure-aws-credentials` over long-lived keys.

### Concurrency
- `concurrency: { group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }`
- Serialize deploys: `cancel-in-progress: false` for CD workflows.

### gh CLI
- `gh run list --limit 5` — recent runs
- `gh run view <id> --log-failed` — failed step logs
- `gh workflow run <name>` — manual trigger

## Claude Code Templates

ECC provides reusable workflow templates in `workflow-templates/`:

| Template | Trigger | Purpose |
|----------|---------|---------|
| `claude-pr-review.yml` | `pull_request` | AI code review with inline annotations |
| `claude-pr-review-fork-safe.yml` | `workflow_run` | Fork-safe two-workflow review pattern |
| `claude-issue-triage.yml` | `issues: opened` | Auto-label with configurable taxonomy |
| `claude-release-notes.yml` | `push: tags: v*` | Categorized release notes generation |
| `claude-ci-linter.yml` | `pull_request` | Convention enforcement with custom rules |

Install via `/scaffold-workflows` or copy manually to `.github/workflows/`.

All templates use `anthropics/claude-code-action@v1` and require `ANTHROPIC_API_KEY` as a repository secret.

### Customization

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `CLAUDE_MODEL` | Claude model to use | Action default |
| `CLAUDE_MAX_TURNS` | Maximum conversation turns | Action default |
| `CLAUDE_ALLOWED_TOOLS` | Comma-separated tool list | `Read,Grep,Glob` |
| `CLAUDE_PROMPT` | Custom review prompt | Template default |
| `LABEL_TAXONOMY` | Issue triage labels | `bug,feature,question,docs,enhancement,security,performance` |
| `CONVENTION_RULES` | CI linter rules | Default conventions |

Set via GitHub repository variables (Settings → Variables → Actions).

## Security

### Permissions
- Always set `permissions:` to least privilege. Never use default (full write).
- PR review: `contents: read`, `pull-requests: write`
- Issue triage: `issues: write`, `contents: read`
- Release notes: `contents: write`

### Secrets
- Store `ANTHROPIC_API_KEY` as a repository or organization secret.
- Never echo or log secrets in workflow steps.
- Use `${{ secrets.NAME }}` — not environment variables from external sources.

### Tool Restriction
- Use `--allowedTools Read,Grep,Glob` for read-only review workflows.
- Only add `Edit,Write` for workflows that need to modify code (e.g., auto-fix).
- Never grant `Bash` access in CI — prevents arbitrary command execution.

### Pitfalls
- **`pull_request_target`**: Runs on base branch with full secret access. Never checkout PR code without SHA pinning.
- **Path filters**: `paths:` only works with `push` and `pull_request`, not `workflow_dispatch`.
- **YAML quoting**: `${{ }}` expressions need quoting inside `if:` conditions.

## Fork Safety

Fork PRs cannot access repository secrets when triggered via `pull_request`. This is a security feature.

### Simple Pattern (skip forks)
The default `claude-pr-review.yml` skips fork PRs with:
```yaml
if: github.event.pull_request.head.repo.fork != true
```

### Two-Workflow Pattern (review forks safely)
For OSS repos that need fork review, use `claude-pr-review-fork-safe.yml`:

1. **Trigger workflow** (`pull_request`): Collects the diff as an artifact, skips fork PRs for the main review
2. **Review workflow** (`workflow_run`): Triggered when the trigger workflow completes, downloads the artifact, runs Claude Code with secrets

**Never** use `pull_request_target` to work around fork limitations — it exposes secrets to untrusted code.

## Customization

### Custom Convention Rules
The CI linter supports two configuration sources (file takes precedence over env var):

1. **`.claude-lint-rules` file** in repo root — one rule per line
2. **`CONVENTION_RULES`** repository variable — semicolon-separated rules

### Custom Review Prompts
Set `CLAUDE_PROMPT` as a repository variable to customize what Claude focuses on during review.

### Filtering
Use `paths:` and `paths-ignore:` to control which file changes trigger workflows:
```yaml
on:
  pull_request:
    paths:
      - 'src/**'
    paths-ignore:
      - '**/*.md'
```

**Important**: Add `paths-ignore` for files that Claude itself may modify to prevent infinite trigger loops.

For ECC-specific workflow conventions, see `rules/ecc/github-actions.md`.
