---
id: BL-046
title: Phase-gate hook blocks spec/plan/design file writes during active workflow phases
status: open
created: 2026-03-21
promoted_to: ""
tags: [hooks, phase-gate, workflow, spec, design, plan, bug]
scope: LOW
target_command: direct edit
---

## Optimized Prompt

Fix the phase-gate hook allowlist so that spec, plan, and design artifact files
can be written during active workflow phases (spec, design) without being blocked.

**Context — project:** Rust workspace (7 crates), Hexagonal Architecture. Hook logic
lives in `.claude/hooks/phase-gate.sh`. The spec-driven pipeline produces file
artifacts under `docs/specs/` (from `/spec`), `docs/plans/` (from `/plan`), and
`docs/designs/` (from `/design`). The current allowlist only permits writes to
`.claude/workflow/*`, `docs/audits/*`, `docs/backlog/*`, and `docs/user-stories/*`,
so any attempt to write a spec or design artifact during an active spec/design phase
is blocked with exit code 2.

**File to change:** `.claude/hooks/phase-gate.sh` (lines 41-54)

**Acceptance criteria:**
- During `spec` phase, writes to `docs/specs/*` are allowed
- During `design` phase, writes to `docs/specs/*` and `docs/designs/*` are allowed
- During any non-implement phase, writes to `docs/plans/*` are allowed
- The BLOCKED error message lists all newly allowed paths accurately
- All existing allowed paths (workflow, audits, backlog, user-stories) remain unchanged
- No other hook behaviour changes

**Implementation steps:**
1. Read `.claude/hooks/phase-gate.sh` in full
2. In the `Write|Edit|MultiEdit` case block (around line 42), add allow-patterns:
   - `*/docs/specs/*` and `docs/specs/*`
   - `*/docs/plans/*` and `docs/plans/*`
   - `*/docs/designs/*` and `docs/designs/*`
3. Update the BLOCKED echo message on line 52 to include the new paths
4. Verify: run `ECC_WORKFLOW_BYPASS=0 claude` and attempt a write to `docs/specs/WI-001-spec.md`
   during an active spec-phase workflow — confirm it passes
5. Verify: attempt a write to a source file (e.g., `crates/ecc-domain/src/lib.rs`) during
   the same phase — confirm it is still blocked

**Scope boundaries — do NOT:**
- Change the Bash destructive-command block logic
- Modify the stale-workflow warning logic
- Add phase-specific conditional logic (allow all three new paths in all active phases
  for simplicity — consistency beats granularity here)
- Touch `hooks.json` or any Rust crate

**Verification:**
```bash
# Confirm the hook file change looks correct
cat -n .claude/hooks/phase-gate.sh | grep -A2 "docs/specs"

# Run existing tests to confirm no regressions
cargo test
```

## Original Input

"Phase-gate hook is blocking plan file edits during plan phase (same issue as
before — the hook only allows workflow/audit/backlog files). The spec preview is
in conversation above. Exiting Plan Mode for approval"

Known recurring issue: the hook allowlist in `.claude/hooks/phase-gate.sh` does not
include `docs/specs/`, `docs/plans/`, or `docs/designs/` — paths produced by the
spec-driven pipeline (`/spec`, `/plan`, `/design` commands).

## Challenge Log

Q: Should the fix also allow `docs/adr/*` during active phases, or scope strictly
   to spec/plan/design artifact paths?

A: (pending — defaulted to spec/plan/design only; ADR writes are typically done
   in implement or post-implement phase so exclusion is safe)

Q: Should allowance be phase-conditional (e.g., only `docs/specs/*` in spec phase,
   only `docs/designs/*` in design phase) or blanket for all active phases?

A: (pending — defaulted to blanket for all active phases for simplicity and to
   avoid the gate becoming fragile as the pipeline evolves)

## Related Backlog Items

- BL-029 (Persist specs and designs as versioned file artifacts) — the artifact
  paths being blocked are exactly those introduced by BL-029
- BL-023 (Clean up stale workflow state) — same hook file, complementary concern
