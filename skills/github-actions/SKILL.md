---
name: github-actions
description: CI/CD patterns, debugging, caching, secrets, and pitfalls for GitHub Actions workflows.
origin: ECC
---

# GitHub Actions

Patterns, debugging, and pitfalls for GitHub Actions workflows.

## CI Patterns

- **Triggers**: `pull_request` for validation, `push` for post-merge. Use `paths:` filter to skip irrelevant changes.
- **Job dependencies**: `needs: [build]` for sequential gates. `if: always()` to run cleanup regardless of failure.
- **Status checks**: Required checks reference **job names**, not workflow filenames.

## CD Patterns

- **Tag-based releases**: `on: push: tags: ['v*']` triggers release builds.
- **Environment protection**: Use `environment:` with required reviewers for production deploys.
- **Deployment gates**: `concurrency: deploy-prod` serializes deploys.

## Cron

- **Schedule syntax**: `cron: '0 9 * * 1'` (Monday 9am UTC). Always set `workflow_dispatch` too for manual triggers.
- **Timeout**: Scheduled workflows have a 5-minute startup window.

## gh CLI

Debug workflows from terminal:

- `gh run list --limit 5` — recent runs
- `gh run view <id> --log-failed` — failed step logs
- `gh workflow run <name>` — manual trigger
- `gh run rerun <id> --failed` — rerun only failed jobs

## Caching

- `actions/cache@v4` with `key: ${{ runner.os }}-${{ hashFiles('**/lockfile') }}`
- Use `restore-keys:` fallback for partial cache hits.
- Cache eviction: 10 GB per repo, LRU.

## Secrets

- Access: `${{ secrets.NAME }}`. Never echo secrets in `run:` blocks.
- Use **environment-level secrets** for deploy credentials (scoped, auditable).
- OIDC: Prefer `aws-actions/configure-aws-credentials` over long-lived keys.

## Concurrency

- `concurrency: { group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }`
- Serialize deploys: `cancel-in-progress: false` for CD workflows.

## Pitfalls

- **`pull_request_target`**: Runs on the base branch. Never checkout PR code without explicit SHA pinning.
- **`permissions`**: Always set to least privilege. Default is `contents: read`.
- **YAML quoting**: `${{ }}` expressions need quoting inside `if:` conditions.
- **Path filters**: `paths:` only works with `push` and `pull_request`, not `workflow_dispatch`.

For ECC-specific conventions, see `rules/ecc/github-actions.md`.
