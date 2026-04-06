---
name: feature-flags
category: cicd
tags: [cicd, feature-toggle, release, runtime]
languages: [all]
difficulty: intermediate
---

## Intent

Decouple deployment from release by wrapping new functionality behind runtime toggles, enabling features to be activated or deactivated without redeployment.

## Problem

Deploying code and releasing features are treated as the same event. Long-lived feature branches diverge and create painful merges. Incomplete features block the main branch. You need a way to deploy code continuously while controlling feature visibility independently.

## Solution

Wrap new functionality behind conditional checks (flags). Flags are evaluated at runtime from a configuration source (environment variable, database, or feature flag service). Deploy code to production with the flag off, then enable it for specific users, percentages, or everyone when ready.

## Language Implementations

### Typescript

```typescript
interface FeatureFlags {
  readonly [key: string]: boolean | PercentageRollout;
}

interface PercentageRollout {
  readonly enabled: boolean;
  readonly percentage: number;
}

function isEnabled(flags: FeatureFlags, key: string, userId?: string): boolean {
  const flag = flags[key];
  if (typeof flag === "boolean") return flag;
  if (!flag?.enabled) return false;
  if (!userId) return false;
  const hash = simpleHash(userId + key);
  return (hash % 100) < flag.percentage;
}

// Usage
if (isEnabled(flags, "new-checkout-flow", user.id)) {
  renderNewCheckout();
} else {
  renderLegacyCheckout();
}
```

### GitHub Actions (flag-gated deploy)

```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Check feature flag
        id: flag
        run: |
          ENABLED=$(curl -s "$FLAG_SERVICE_URL/api/flags/new-search" | jq -r '.enabled')
          echo "enabled=$ENABLED" >> "$GITHUB_OUTPUT"

      - name: Deploy with flag
        if: steps.flag.outputs.enabled == 'true'
        run: ./deploy.sh --enable new-search
```

## When to Use

- When you want to deploy incomplete features behind a toggle.
- When you need gradual rollouts or A/B testing capability.
- When different users or tenants need different feature sets.

## When NOT to Use

- When the feature is trivial and a single deployment suffices.
- When flag cleanup discipline is lacking — stale flags become tech debt.
- When the flag introduces complex branching in business-critical paths.

## Anti-Patterns

- Never cleaning up old flags — leading to hundreds of stale conditionals.
- Nesting multiple flags creating combinatorial explosion of code paths.
- Using flags for permanent configuration instead of temporary feature gating.

## Related Patterns

- [cicd/canary](canary.md) — infrastructure-level progressive rollout without code changes.
- [cicd/blue-green](blue-green.md) — environment-level switching for deployment safety.
- [cicd/trunk-based-dev](trunk-based-dev.md) — flags enable trunk-based development with incomplete features.

## References

- Martin Fowler — Feature Toggles: https://martinfowler.com/articles/feature-toggles.html
- Pete Hodgson — Feature Flags Best Practices: https://launchdarkly.com/blog/feature-flag-best-practices/
