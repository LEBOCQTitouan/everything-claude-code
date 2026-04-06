---
name: canary
category: cicd
tags: [cicd, deployment, progressive, observability]
languages: [all]
difficulty: advanced
---

## Intent

Reduce deployment risk by gradually shifting traffic from the old version to the new version while monitoring key metrics for regressions.

## Problem

Deploying a new version to all users simultaneously exposes the entire user base to potential bugs. Rollback is reactive — damage is done before detection. You need a way to validate in production with limited blast radius.

## Solution

Deploy the new version alongside the current one. Route a small percentage of traffic (1-5%) to the canary. Monitor error rates, latency, and business metrics. If metrics remain healthy, progressively increase traffic. If regressions appear, route all traffic back to the stable version.

## Language Implementations

### GitHub Actions with ArgoCD

```yaml
name: Canary Deploy
on:
  push:
    branches: [main]

jobs:
  canary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy canary (5%)
        run: |
          kubectl apply -f k8s/canary.yaml
          kubectl annotate rollout app \
            argo-rollouts.argoproj.io/set-weight=5

      - name: Monitor metrics (5 min)
        run: ./scripts/canary-monitor.sh --duration 300 --threshold 0.01

      - name: Promote or rollback
        run: |
          if ./scripts/check-metrics.sh; then
            kubectl argo rollouts promote app
          else
            kubectl argo rollouts abort app
          fi
```

### ArgoCD Rollout

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: app
spec:
  strategy:
    canary:
      steps:
        - setWeight: 5
        - pause: { duration: 5m }
        - setWeight: 25
        - pause: { duration: 5m }
        - setWeight: 50
        - pause: { duration: 5m }
        - setWeight: 100
      canaryMetadata:
        labels:
          role: canary
```

## When to Use

- When you need production validation before full rollout.
- When metrics and observability infrastructure are mature enough for automated analysis.
- When user-facing changes carry high risk (payment flows, auth changes).

## When NOT to Use

- When observability infrastructure is insufficient to detect regressions.
- When traffic volume is too low to produce statistically significant metrics.
- When changes are all-or-nothing (schema migrations, protocol changes).

## Anti-Patterns

- Setting canary weight too high initially — defeats the purpose of gradual rollout.
- Monitoring only error rates while ignoring latency and business metrics.
- Promoting automatically without human review for critical services.

## Related Patterns

- [cicd/blue-green](blue-green.md) — atomic switch instead of gradual traffic shift.
- [cicd/feature-flags](feature-flags.md) — runtime toggle without infrastructure-level routing.
- [cicd/rolling-update](rolling-update.md) — instance-by-instance replacement without traffic control.

## References

- Argo Rollouts — Canary Strategy: https://argoproj.github.io/argo-rollouts/features/canary/
- Google SRE Book — Release Engineering: https://sre.google/sre-book/release-engineering/
