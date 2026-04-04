# ADR 0042: Vision-Based Comparison as Primary Visual Testing Approach

Status: Accepted

Date: 2026-04-02

## Context

BL-103 introduces autonomous visual testing to the e2e-runner agent. The agent needs to detect visual regressions by comparing screenshots across test runs. Two fundamental approaches exist:

1. **Pixel-diff tools** (pixelmatch, reg-cli, Playwright's built-in `toHaveScreenshot()`): Precise pixel-level comparison that generates overlay images with highlighted differences. Deterministic and fast, but prone to false positives from anti-aliasing, font rendering differences, and sub-pixel shifts across platforms.

2. **AI vision-based comparison**: Using Claude's native image understanding (via the Read tool) to semantically compare screenshots. Understands layout, content, and structural relationships rather than individual pixels. Slower (~3s per comparison) and non-deterministic, but produces meaningful natural-language reports and ignores cosmetic noise.

The e2e-runner agent operates within Claude Code sessions where the Read tool already supports image analysis. No external API integration is required for vision-based comparison.

## Decision

Use AI vision-based comparison as the **primary** approach for visual regression detection in the e2e-runner agent. Provide pixel-diff tooling patterns (pixelmatch, reg-cli) as **supplementary** guidance for CI pipelines where deterministic, automated comparison is needed.

The vision-based approach:
- Reads current and baseline screenshots via the Read tool
- Produces natural-language descriptions of differences
- Classifies severity as cosmetic, functional, or breaking
- Operates within the existing agent tool set (no new dependencies)

The pixel-diff supplementary approach:
- Documented as patterns in the visual-testing skill
- Recommended for CI/CD pipelines where human review is deferred
- Uses pixelmatch for Node.js projects and reg-cli for standalone comparison
- Produces diff images with highlighted changes

## Consequences

### Positive

- No external service dependencies — uses existing Claude capabilities
- Semantic understanding of changes — catches functional regressions while ignoring cosmetic noise
- Natural-language reports — easier to understand than pixel-diff overlays
- Aligns with the "round-trip screenshot testing" pattern endorsed by Anthropic best practices

### Negative

- Non-deterministic — the same comparison may produce slightly different descriptions across runs
- Higher latency — ~3s per comparison vs milliseconds for pixel-diff
- Higher cost — ~1K tokens per comparison call
- Cannot produce pixel-level diff images — supplementary tooling needed for that use case

### Neutral

- Pixel-diff tooling remains available as a documented pattern for teams that need deterministic, automated comparison
- The dual-mode approach (vision primary + pixel-diff supplementary) matches the 2025-2026 industry consensus
