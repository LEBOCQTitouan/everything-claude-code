---
name: gitops
category: cicd
tags: [cicd, gitops, argocd, declarative, kubernetes]
languages: [all]
difficulty: advanced
---

## Intent

Use Git as the single source of truth for declarative infrastructure and application configuration, with automated agents ensuring the live state matches the desired state in the repository.

## Problem

Infrastructure state drifts from what is defined in code when operators apply manual changes. CI-push pipelines can fail silently, leaving environments in unknown states. There is no single authoritative source for "what should be running."

## Solution

Store all desired state in a Git repository. A GitOps operator (ArgoCD, Flux) continuously reconciles the live cluster state against the Git repository. Changes are made exclusively through pull requests. The operator detects and corrects drift automatically.

## Language Implementations

### ArgoCD Application

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-app
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/org/infra-config.git
    targetRevision: main
    path: apps/my-app
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### GitHub Actions (GitOps trigger)

```yaml
name: Update manifest
on:
  push:
    branches: [main]
    paths: [src/**]

jobs:
  update-manifests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: org/infra-config
          token: ${{ secrets.INFRA_TOKEN }}

      - name: Update image tag
        run: |
          yq -i ".spec.template.spec.containers[0].image = \"app:${{ github.sha }}\"" \
            apps/my-app/deployment.yaml

      - name: Commit and push
        run: |
          git add .
          git commit -m "chore: update my-app to ${{ github.sha }}"
          git push
```

## When to Use

- When Kubernetes is your deployment platform.
- When you need audit trails for every infrastructure change.
- When drift detection and self-healing are requirements.

## When NOT to Use

- When infrastructure is not declarative (legacy VMs with imperative scripts).
- When the team is not disciplined about making all changes through Git.
- When secrets management cannot be handled outside the Git repository.

## Anti-Patterns

- Committing secrets to the GitOps repository instead of using sealed-secrets or external vaults.
- Disabling self-heal and letting drift accumulate.
- Using a single monorepo for all environments without branch or path separation.

## Related Patterns

- [cicd/iac](iac.md) — define infrastructure as code; GitOps adds reconciliation.
- [cicd/pipeline-as-code](pipeline-as-code.md) — versioned CI/CD definitions complement GitOps.
- [cicd/trunk-based-dev](trunk-based-dev.md) — short-lived branches feed the GitOps repository.

## References

- OpenGitOps Principles: https://opengitops.dev/
- ArgoCD Documentation: https://argo-cd.readthedocs.io/en/stable/
- Weaveworks — Guide to GitOps: https://www.weave.works/technologies/gitops/
