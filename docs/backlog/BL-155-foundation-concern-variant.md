---
id: BL-155
title: "Add Foundation variant to Concern domain enum for /project-foundation workflow"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-04-17"
source: "Code review HIGH finding from shell-eval injection fix implementation"
ring: trial
tags: [domain, workflow, project-foundation]
---

## Context

The `Concern` enum in `crates/ecc-domain/src/workflow/concern.rs` supports three variants: `Dev`, `Fix`, `Refactor`. The `/project-foundation` slash-command (added in BL-143) was designed to initialize workflow state with concern=`foundation`, but the domain enum was never updated. Running `ecc-workflow init foundation ...` returns:

```
{"status":"block","message":"Invalid concern: unknown concern: foundation (expected dev, fix, or refactor)"}
```

This is a pre-existing bug surfaced during the shell-eval injection fix code review. The immediate workaround applied in the shell-escape fix (commit `fabbb5af`): `/project-foundation` template now instructs `ecc-workflow init dev --feature-stdin` with an explicit note that `foundation` concern support is tracked here.

## Prompt

Add `Foundation` as a fourth variant to the `Concern` enum. Touch points:

1. `crates/ecc-domain/src/workflow/concern.rs` — add `Foundation` to the enum + `FromStr` + `Display` + any exhaustive `match` sites.
2. `crates/ecc-workflow/` — verify CLI accepts `foundation` as concern arg (clap will get it from the domain via parsing).
3. `crates/ecc-domain/tests/` — add unit test for `Concern::from_str("foundation")`.
4. `commands/project-foundation.md` — revert the workaround: instruct `ecc-workflow init foundation --feature-stdin` instead of `dev`.
5. `docs/specs/` — the foundation spec directory naming may want a `foundation-` prefix convention (vs current `dev-`/`fix-`/`refactor-` implicit) — decide as part of this ticket.

## Acceptance Criteria

- [ ] `Concern::Foundation` variant exists with `FromStr` parsing `"foundation"`.
- [ ] `ecc-workflow init foundation <feature>` succeeds (exit 0, writes state.json with concern=foundation).
- [ ] `ecc workflow init foundation --feature-stdin` delegator path works identically.
- [ ] `/project-foundation` template updated to use `init foundation` instead of the current `init dev` workaround.
- [ ] Unit test: Concern round-trip `"foundation" -> Foundation -> "foundation"`.
- [ ] Integration test: foundation workflow state reaches `done` phase via /project-foundation pipeline.
- [ ] Existing `dev | fix | refactor` variants unchanged.

## Out of Scope

- Foundation-specific phase transitions (e.g., skipping /design if foundation doesn't require it). Current pipeline uses same phases for all concerns.
- Backfilling concern=`foundation` into existing state.json files (none exist; BL-143 used `dev` as workaround).
- Spec directory naming convention overhaul — can be a separate ticket if foundation-specific slugging is desired.
