---
id: BL-141
title: "Monitor serde-saphyr for 0.1.0 stability release"
scope: LOW
target: "direct edit"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [deps, yaml, stability]
---

## Context

serde_yaml is deprecated. serde-saphyr is the actively maintained successor but at 0.0.x pre-stability. ECC uses serde-saphyr 0.0.22 for YAML frontmatter parsing.

## Prompt

Monitor serde-saphyr releases for 0.1.0 stabilization. When reached, bump the version and verify all frontmatter parsing tests pass. If API breaks occur before stabilization, evaluate serde_yaml_bw as a fallback.

## Acceptance Criteria

- [ ] Version bumped when 0.1.0 released
- [ ] All frontmatter parsing tests pass
- [ ] Fallback plan documented if API breaks
