# Command Reference

## Audit Commands

| Command | Domain |
|---------|--------|
| `/audit-full` | All domains — parallel run with cross-domain correlation |
| `/audit-archi` | Boundary integrity, dependency direction, DDD compliance |
| `/audit-code` | SOLID, clean code, naming, complexity |
| `/audit-convention` | Naming patterns, style consistency |
| `/audit-doc` | Coverage, staleness, drift |
| `/audit-errors` | Swallowed errors, taxonomy, boundary translation |
| `/audit-evolution` | Git hotspots, churn, bus factor, complexity trends |
| `/audit-observability` | Logging, metrics, tracing, health endpoints |
| `/audit-security` | OWASP top 10, secrets, attack surface |
| `/audit-test` | Coverage, classification, fixture ratios, E2E matrix |
| `/audit-web` | Web-based upgrade scan with Technology Radar output |

## Spec-Driven Pipeline

| Command | Purpose |
|---------|---------|
| `/spec` | Auto-classify intent (dev/fix/refactor) and delegate to matching `/spec-*` |
| `/spec-dev` | Spec a new feature — research, grill-me interview, adversarial review |
| `/spec-fix` | Spec a bug fix — investigation, blast radius, adversarial review |
| `/spec-refactor` | Spec a refactoring — smell catalog, current state analysis, adversarial review |
| `/design` | Technical design from spec — file changes, pass conditions, TDD order |
| `/implement` | Execute the design — TDD loop, code review, doc updates |

## Side Commands

| Command | Purpose |
|---------|---------|
| `/verify` | Build + tests + lint + code review + architecture review + drift check |
| `/review` | Robert professional conscience check |
| `/backlog` | Capture and manage implementation ideas |
| `/build-fix` | Fix build/type errors reactively |
| `/commit` | Git commit with intelligent staging, conventional message, atomic enforcement, pre-flight gates |
| `/catchup` | Session resumption — workflow state, git status, stale detection, memory |
| `/create-component` | Scaffold or update an ECC component (agent, command, skill, hook) |
| `/ecc-test-mode` | Isolated worktree for testing ECC config changes |
| `/comms` | Manage comms pipeline infrastructure — init repo, edit strategies, manage drafts, view calendar |
| `/comms-generate` | Generate multi-channel DevRel content from codebase with optional channel filtering |
| `/comms-plan` | Content ideation and scheduling — structured interview, web trend research, publication schedule |
