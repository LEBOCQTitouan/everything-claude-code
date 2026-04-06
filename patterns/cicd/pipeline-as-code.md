---
name: pipeline-as-code
category: cicd
tags: [cicd, pipeline, automation, versioned]
languages: [all]
difficulty: beginner
---

## Intent

Define CI/CD pipelines as version-controlled code files within the repository, making build and deployment processes reviewable, reproducible, and auditable.

## Problem

Pipelines configured through web UIs are opaque, not versioned, and cannot be reviewed. Changes to build processes are invisible to the team. Reproducing a pipeline on a different project requires manual recreation. There is no history of why a pipeline changed.

## Solution

Store pipeline definitions as code files in the repository (e.g., `.github/workflows/`, `Jenkinsfile`, `.gitlab-ci.yml`). Pipeline changes go through the same PR review process as application code. Reusable actions or templates reduce duplication across repositories.

## Language Implementations

### GitHub Actions

```yaml
name: CI
on:
  pull_request:
    branches: [main]

permissions:
  contents: read

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: npm ci
      - run: npm test
      - run: npm run build
```

### Reusable workflow (GitHub Actions)

```yaml
# .github/workflows/reusable-test.yml
name: Reusable Test
on:
  workflow_call:
    inputs:
      node-version:
        type: string
        default: "20"

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: ${{ inputs.node-version }} }
      - run: npm ci && npm test
```

### Terraform (CI for IaC)

```yaml
jobs:
  terraform:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: hashicorp/setup-terraform@v3
      - run: terraform fmt -check
      - run: terraform init
      - run: terraform validate
      - run: terraform plan
```

## When to Use

- Always. Pipeline-as-code is the baseline expectation for modern CI/CD.
- When pipelines must be auditable and reproducible.
- When multiple repositories share common build patterns (use reusable workflows).

## When NOT to Use

- Rarely. Even simple projects benefit from versioned pipelines.
- When prototyping and a hosted CI is genuinely overkill (local scripts suffice temporarily).

## Anti-Patterns

- Configuring pipelines through a web UI when a code-based option exists.
- Duplicating pipeline logic across repositories instead of extracting reusable workflows.
- Using `@latest` for action versions instead of pinning specific versions.

## Related Patterns

- [cicd/iac](iac.md) — infrastructure definitions follow the same versioned-code principle.
- [cicd/gitops](gitops.md) — Git-driven reconciliation extends pipeline-as-code to deployment.
- [cicd/trunk-based-dev](trunk-based-dev.md) — pipeline-as-code enables CI on every trunk commit.

## References

- GitHub Actions Documentation: https://docs.github.com/en/actions
- Jenkins — Pipeline as Code: https://www.jenkins.io/doc/book/pipeline-as-code/
- GitLab CI/CD: https://docs.gitlab.com/ee/ci/
