---
name: rolling-update
category: cicd
tags: [cicd, deployment, kubernetes, incremental]
languages: [all]
difficulty: beginner
---

## Intent

Deploy a new version by incrementally replacing instances of the old version, ensuring the service remains available throughout the process.

## Problem

Taking all instances offline simultaneously causes downtime. Blue-green deployments double infrastructure costs. You need a deployment strategy that maintains availability with minimal extra resources.

## Solution

Replace instances one at a time (or in small batches). Each new instance passes health checks before the next old instance is drained. Configure `maxUnavailable` and `maxSurge` to control the rollout speed and resource overhead.

## Language Implementations

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  replicas: 4
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    spec:
      containers:
        - name: app
          image: app:v2
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
```

### GitHub Actions

```yaml
jobs:
  rolling-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Update image
        run: |
          kubectl set image deployment/app app=app:${{ github.sha }}
          kubectl rollout status deployment/app --timeout=300s

      - name: Verify rollout
        run: kubectl get pods -l app=app -o wide
```

## When to Use

- When you need zero-downtime deploys with minimal extra infrastructure.
- When Kubernetes is your deployment platform (native support).
- When old and new versions can coexist handling requests simultaneously.

## When NOT to Use

- When old and new versions are incompatible (breaking API changes).
- When you need instant rollback — rolling back also takes time.
- When precise traffic control is needed (use canary instead).

## Anti-Patterns

- Setting `maxUnavailable` too high, causing capacity drops during rollout.
- Missing or misconfigured readiness probes, causing traffic to unhealthy pods.
- Deploying database-breaking changes without backward compatibility.

## Related Patterns

- [cicd/blue-green](blue-green.md) — atomic switch between two full environments.
- [cicd/canary](canary.md) — traffic-weighted progressive rollout with metric gates.
- [cicd/feature-flags](feature-flags.md) — decouple release from deployment at code level.

## References

- Kubernetes — Rolling Update Strategy: https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#rolling-update-deployment
- Google Cloud — Deployment Strategies: https://cloud.google.com/architecture/application-deployment-and-testing-strategies
