---
name: blue-green
category: cicd
tags: [cicd, deployment, zero-downtime, rollback]
languages: [all]
difficulty: intermediate
---

## Intent

Eliminate deployment downtime by maintaining two identical production environments and switching traffic between them atomically.

## Problem

Traditional deployments require taking the application offline or running degraded during updates. Rolling back a failed deployment is slow and error-prone because the previous version must be redeployed from scratch.

## Solution

Maintain two identical environments (blue and green). Deploy the new version to the idle environment, run smoke tests, then switch the load balancer to route all traffic to the newly deployed environment. The previous environment remains intact for instant rollback.

## Language Implementations

### GitHub Actions

```yaml
name: Blue-Green Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Determine target environment
        id: target
        run: |
          CURRENT=$(aws elbv2 describe-target-groups --query "current")
          TARGET=$([ "$CURRENT" = "blue" ] && echo "green" || echo "blue")
          echo "env=$TARGET" >> "$GITHUB_OUTPUT"

      - name: Deploy to ${{ steps.target.outputs.env }}
        run: ./scripts/deploy.sh ${{ steps.target.outputs.env }}

      - name: Smoke test
        run: ./scripts/smoke-test.sh ${{ steps.target.outputs.env }}

      - name: Switch traffic
        run: ./scripts/switch-traffic.sh ${{ steps.target.outputs.env }}
```

### Terraform

```hcl
resource "aws_lb_listener_rule" "blue_green" {
  listener_arn = aws_lb_listener.main.arn
  priority     = 100

  action {
    type             = "forward"
    target_group_arn = var.active_color == "blue" ? aws_lb_target_group.blue.arn : aws_lb_target_group.green.arn
  }

  condition {
    path_pattern { values = ["/*"] }
  }
}

variable "active_color" {
  type    = string
  default = "blue"
}
```

## When to Use

- When zero-downtime deployments are a hard requirement.
- When you need instant rollback capability (seconds, not minutes).
- When the application can run two versions simultaneously without conflict.

## When NOT to Use

- When infrastructure costs must be minimized — two full environments doubles cost.
- When database schema changes are tightly coupled to application code.
- When traffic volume is too low to justify the complexity.

## Anti-Patterns

- Sharing mutable state (databases, caches) between blue and green without migration strategy.
- Skipping smoke tests before switching traffic.
- Leaving the idle environment out of date for too long, causing configuration drift.

## Related Patterns

- [cicd/canary](canary.md) — gradual traffic shift instead of all-at-once switch.
- [cicd/rolling-update](rolling-update.md) — incremental instance replacement without dual environments.
- [cicd/feature-flags](feature-flags.md) — decouple deployment from release using runtime toggles.

## References

- Martin Fowler — BlueGreenDeployment: https://martinfowler.com/bliki/BlueGreenDeployment.html
- AWS Blue/Green Deployments: https://docs.aws.amazon.com/whitepapers/latest/blue-green-deployments/welcome.html
