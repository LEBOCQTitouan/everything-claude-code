# ADR 0067: Clap-derive diagram deny-list

Status: **Accepted** (2026-04-19)

Source: BL-132 Revision 2026-04-18, Decision #4

## Context

The `ascii-doc-diagrams` skill adds ASCII-art diagrams and `# Pattern`
annotations to `///` doc-comments across the workspace. ECC's `ecc-cli`
and `ecc-workflow` crates use `#[derive(Parser | Subcommand | Args |
ValueEnum)]` extensively — 54+ derive sites across ~25 files.

Clap's derive macros promote the **first line** of `///` above a
derive-target (struct / enum variant / field) into the
`--help` short description, and subsequent paragraphs into
`--help --long`. An ASCII state-transition or flow diagram embedded in
such a `///` block will render verbatim inside user-visible CLI help
output — corrupting the UX.

Example of the forbidden pattern:

```text
/// Arguments for the widget command.
///
/// ```text
/// [widget cmd]
///    +---> [name]
/// ```
#[derive(Parser)]
pub struct Args { ... }
```

When a user runs `ecc widget --help --long`, the diagram appears
verbatim after the one-line description — noise that cannot be
explained in a terminal.

## Decision

In files containing any of `#[derive(Parser)]`, `#[derive(Subcommand)]`,
`#[derive(Args)]`, or `#[derive(ValueEnum)]`:

1. Inline fenced `text` diagrams and `# Pattern` annotations are
   **forbidden** on `///` doc-comments directly above a struct, enum
   variant, or field (the derive-target positions).
2. Diagrams **remain permitted** at:
   - Module level (`//!` comments) — clap never consumes these.
   - Inside `impl` blocks — clap sees only the struct/variant/field
     attributes, not inherent-impl method docs.

The deny-list is enforced mechanically by `scripts/check-clap-derive-diagrams.sh`,
which walks the AST-lite via an awk brace-depth state machine. Fixtures
under `scripts/fixtures/bl132/` lock the classifier behaviour:

| Fixture | Scenario | Expected |
|---------|----------|----------|
| `clap_impl_ok.rs` | diagram inside `impl` block | PASS |
| `clap_module_ok.rs` | diagram at `//!` header | PASS |
| `clap_struct_fail.rs` | diagram on `///` above `#[derive(Parser)]` struct | FAIL |

Classifier false-positives or false-negatives regress fixtures and
abort the run.

## Consequences

### Enforcement

- CI validate job gains a `--help` smoke step (PC-021) running
  `./target/release/ecc --help | grep -q 'Usage:'` and the equivalent
  for `ecc-workflow`. Any --help corruption breaks CI before merge.
- `scripts/check-clap-derive-diagrams.sh` runs in CI as PC-015-v3,
  backstopping the smoke test with direct source-level detection.

### Coverage trade-off

- ~25 files across `ecc-cli` and `ecc-workflow` are partially off-limits
  to inline diagrams. Authors wanting to document these crates must use
  module-level `//!` blocks or attach docs to `impl` blocks. The skill's
  coverage ceiling (115 eligible items) is reduced in these crates;
  this loss is accepted because the alternative (corrupted `--help`) is
  far worse than minor doc-surface reduction.

### Extension

- Rule extends to any future clap derive added (e.g., `Args`, `ValueEnum`
  variants).
- If ECC later migrates off clap (unlikely — clap is the de-facto Rust
  CLI crate), the classifier can be deleted and the deny-list rule
  retired via a follow-up ADR.

## Alternatives considered

### A. Snapshot-gate `--help` with `insta`

Take golden snapshots of `ecc --help` and each subcommand's `--help
--long` output. CI diffs snapshots against live output; any diff
fails.

**Rejected**: 54+ subcommands × 2 (short + long) = 108+ snapshots.
Maintenance burden is high; every `--help` wording change requires
snapshot regeneration. Snapshot files are a secondary source of truth
that must be kept consistent with CLI wording — doubling editorial
load. Direct source-level detection is cheaper.

### B. Per-derive-item review

Allow diagrams on derive-targets but require reviewer sign-off on
each one that ships. Verify manually during PR review.

**Rejected**: doesn't scale; relies on reviewer vigilance; fails open
when the reviewer is absent or tired. "Agent discipline" is not a
verification strategy.

### C. Write classifiers in Rust (`xtask`)

Use `syn` for AST parsing instead of awk. Live in
`xtask/src/check_diagrams.rs`, invoked via `cargo xtask check-diagrams`.

**Rejected (for now)**: awk classifiers run against raw source text
without requiring a successful `cargo build`. This matters during the
sweep itself — implementation agents can author diagrams on an
uncompiled branch and validate locally with a single shell invocation.
A `cargo xtask` classifier would require cyclic build-then-validate
iterations. Additionally, the existing `scripts/` tree already holds
language-agnostic repo tooling (`bump-version.sh`, `get-ecc.sh`,
`audit-mcp-versions.sh`); adding three more shell scripts matches
precedent.

**Migration path**: if a future ECC phase unifies all lint tooling
under `xtask/`, these three classifiers follow. The rule (R-1) is
independent of the classifier implementation — only the script paths
change.

### D. Move all `///` comments in clap-derive files to `impl` blocks

Ban `///` on derive-targets altogether; allow only `//!` and `impl`-
attached doc-comments.

**Rejected**: breaks clap's `--help` generation. Clap needs the `///`
above derive-targets for short descriptions of subcommands and args.
Removing those breaks the CLI UX entirely.

## Related

- **Spec**: `docs/specs/2026-04-17-bl132-ascii-diagrams/spec.md`
  Revision 2026-04-18 § R-1
- **Design**: `docs/specs/2026-04-17-bl132-ascii-diagrams/design.md`
  Revision 2026-04-18 § PC-015-v3
- **Skill**: `skills/ascii-doc-diagrams/SKILL.md`
- **Classifier**: `scripts/check-clap-derive-diagrams.sh`
- **Fixtures**: `scripts/fixtures/bl132/clap_{impl,module,struct}_*.rs`
