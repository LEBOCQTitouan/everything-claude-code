# ADR 0043: Conditional Rule Loading via applies-to Frontmatter

## Status
Accepted

## Context
ECC loads all rules unconditionally regardless of the target project's stack. This causes prompt bloat — Django skills loaded for Rust projects, Spring Boot patterns for Python codebases. Stripe's Minions demonstrate that conditional rule application based on subdirectories significantly improves agent focus. ECC already has 15+ language and 30+ framework detectors in `ecc-domain/src/detection/` but they weren't used for rule filtering.

## Decision
Add an `applies-to` frontmatter field to rules declaring applicability conditions (languages, frameworks, sentinel files). `ecc install` detects the project stack using existing detection infrastructure and filters rules at install time. OR semantics: any matching condition means the rule applies. Rules without `applies-to` install unconditionally (backwards compatible). Fail-open: zero stacks detected or detection error installs ALL rules with a warning. `--all-rules` flag overrides filtering.

## Consequences
- Reduced prompt bloat: only stack-relevant rules installed per project
- Backwards compatible: existing rules and custom rules continue to work
- Fail-open safety: detection failures never silently exclude rules
- Extensible: new languages/frameworks automatically work via existing detectors
- `--all-rules` provides escape hatch for power users
