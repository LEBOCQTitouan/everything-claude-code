---
name: runbook-gen
description: Generate operational runbooks from config extraction and failure mode data — structured procedures for incident response and routine operations.
origin: ECC
---

# Runbook Generation

Generation skill for producing operational runbooks that help on-call engineers respond to incidents and perform routine operations. Combines config-extraction data (what knobs exist) with failure-modes data (what can break) into actionable procedures.

## When to Activate

- After config-extraction and failure-modes analysis
- When documenting operational procedures
- When building incident response documentation
- Before production deployments (pre-flight runbook)

## Methodology

### 1. Runbook Types

Generate runbooks by category:

| Type | Source Data | Purpose |
|------|-----------|---------|
| **Incident Response** | failure-modes | What to do when things break |
| **Deployment** | config-extraction + git-narrative | How to deploy and rollback |
| **Routine Operations** | config-extraction | Scheduled maintenance tasks |
| **Troubleshooting** | failure-modes + behaviour-extraction | Diagnostic decision trees |

### 2. Incident Response Runbook Structure

For each failure scenario from failure-modes extraction:

```markdown
## Incident: [Failure Name]

**Severity:** [Critical/Major/Minor]
**Detection:** [How you know this is happening — alert, log pattern, user report]
**Impact:** [What's broken, who's affected, what still works]

### Diagnosis

1. Check [specific log/metric/dashboard]
2. Verify [specific condition]
3. If [condition A] → go to Recovery Option 1
4. If [condition B] → go to Recovery Option 2

### Recovery Option 1: [Name]

**When to use:** [condition]
**Time to recover:** [estimate]

1. [Step 1 — specific command or action]
2. [Step 2]
3. **Verify:** [how to confirm it's fixed]

### Recovery Option 2: [Name]

**When to use:** [condition]

1. [Steps...]

### Post-Incident

- [ ] Update status page
- [ ] Write incident report
- [ ] Create follow-up tickets for root cause
```

### 3. Deployment Runbook Structure

```markdown
## Deployment: [Service Name]

### Pre-Deployment Checklist

- [ ] All tests pass on the branch
- [ ] Database migrations reviewed (if any)
- [ ] Feature flags configured
- [ ] Rollback plan verified
- [ ] On-call engineer notified

### Deploy Steps

1. [Step-by-step deployment procedure]
2. [Include specific commands]

### Post-Deployment Verification

1. Check health endpoint: `curl https://[service]/health`
2. Verify key metrics in [dashboard]
3. Run smoke tests: `[command]`

### Rollback

**Trigger:** [When to rollback — metric threshold, error rate, user reports]

1. [Rollback steps]
2. **Verify:** [How to confirm rollback succeeded]
```

### 4. Troubleshooting Decision Trees

For complex diagnostic scenarios, generate decision trees:

```markdown
## Troubleshooting: [Symptom]

**Symptom:** [What the user/operator observes]
```
Is the service responding?
├── No → Check process status (`systemctl status app`)
│   ├── Not running → Check logs, restart: `systemctl restart app`
│   └── Running → Check network/firewall
└── Yes, but slow
    ├── Check database connection pool
    │   ├── Pool exhausted → Restart app, investigate leaked connections
    │   └── Pool healthy → Check slow query log
    └── Check external service latency
        ├── External service slow → Enable circuit breaker
        └── All fast → Profile application code
```
```

### 5. Environment-Specific Sections

Use config-extraction data to document per-environment details:

| Variable | Production | Staging | Development |
|----------|-----------|---------|-------------|
| DATABASE_URL | [redacted] | [redacted] | localhost:5432 |
| LOG_LEVEL | warn | info | debug |
| FEATURE_X | false | true | true |

### 6. Output Structure

Place runbooks in `docs/runbooks/` (or `docs/runbooks/operations.md` for small projects):

```
docs/runbooks/
  INDEX.md              — table of contents
  incident-response.md  — all incident procedures
  deployment.md         — deployment and rollback
  troubleshooting.md    — diagnostic decision trees
  routine-ops.md        — scheduled maintenance
```

## Quality Rules

- Every step must be a concrete action (command to run, button to click, value to check)
- Never use vague language ("check if everything is okay") — specify what to check and what "okay" looks like
- Include expected output for verification commands
- Link to dashboards, log queries, and documentation where relevant
- Include escalation paths (who to contact if the procedure doesn't work)

## Related

- Config extraction: `skills/config-extraction/SKILL.md`
- Failure modes: `skills/failure-modes/SKILL.md`
- Behaviour extraction: `skills/behaviour-extraction/SKILL.md`
- Doc generator agent: `agents/doc-generator.md`
