---
name: trunk-based-dev
category: cicd
tags: [cicd, branching, continuous-integration, workflow]
languages: [all]
difficulty: beginner
---

## Intent

Reduce integration pain by having all developers commit to a single shared branch (trunk/main) with short-lived feature branches, enabling continuous integration and rapid delivery.

## Problem

Long-lived feature branches diverge from trunk, creating merge conflicts and delaying integration. Feature branches hide integration bugs until merge time. Code review cycles are long because branches accumulate large diffs.

## Solution

Developers work on short-lived branches (hours to 1-2 days) that branch from and merge back to trunk. Feature flags hide incomplete work. The trunk is always deployable. CI runs on every push to trunk, catching integration issues immediately.

## Language Implementations

### GitHub Actions (trunk CI)

```yaml
name: Trunk CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: make build
      - name: Test
        run: make test
      - name: Lint
        run: make lint
```

### Git workflow

```bash
# Short-lived branch workflow
git checkout main && git pull
git checkout -b feat/add-search      # branch from trunk
# ... work for hours, not days ...
git push -u origin feat/add-search   # push for CI + review
# PR merged same day via squash merge
git checkout main && git pull
git branch -d feat/add-search
```

## When to Use

- When the team practices continuous integration and frequent deployments.
- When feature flags are available to hide incomplete work.
- When code review turnaround is fast (same-day reviews).

## When NOT to Use

- When the team cannot review and merge PRs within 1-2 days.
- When feature flags are not available and incomplete features cannot be hidden.
- When regulatory requirements mandate long-lived release branches.

## Anti-Patterns

- Keeping branches open for weeks — defeats the purpose of trunk-based development.
- Merging large PRs without breaking them into smaller increments.
- Skipping CI on the main branch, assuming feature branch CI is sufficient.

## Related Patterns

- [cicd/feature-flags](feature-flags.md) — enable trunk-based dev by hiding incomplete features.
- [cicd/pipeline-as-code](pipeline-as-code.md) — CI pipeline defined alongside trunk.
- [cicd/canary](canary.md) — safely validate trunk deployments in production.

## References

- Trunk-Based Development: https://trunkbaseddevelopment.com/
- Paul Hammant — Trunk-Based Development: https://paulhammant.com/2013/04/05/what-is-trunk-based-development/
- Google Engineering — Why Google Stores Billions of Lines in a Single Repository: https://research.google/pubs/pub45424/
