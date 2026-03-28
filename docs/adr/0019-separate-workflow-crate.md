# ADR 0019: Separate ecc-workflow Crate for Compiled Hook Replacement

## Status

Accepted

## Context

ECC's workflow state machine was implemented in 13 shell scripts under `.claude/hooks/` that depended on `jq` and a POSIX shell. This made ECC unusable on Windows and any environment without these tools. The scripts bypassed the hexagonal architecture entirely, reading/writing `state.json` directly with no port abstraction or domain modeling.

Two approaches were considered:
1. **Extend ecc-hook**: Add workflow subcommands to the existing `ecc-hook` binary
2. **Separate crate**: Create a new `ecc-workflow` crate with its own binary

## Decision

Create a separate `ecc-workflow` crate (8th in workspace) with a single binary and subcommand dispatch. The crate depends only on `ecc-domain` for WorkflowState and Phase types.

Key design choices:
- Single binary with clap subcommand dispatch (not 13 separate binaries)
- Domain types (WorkflowState, Phase) live in `ecc-domain` as pure domain logic
- `ecc-workflow` does NOT depend on `ecc-app`, `ecc-ports`, `ecc-infra`, or `ecc-cli`
- Dual invocation mode: CLI args (from `!bash` callers) + stdin JSON (from hooks.json runtime)
- Structured JSON output: `{"status":"pass"|"block"|"warn","message":"..."}`

## Consequences

- `ecc-workflow` can be built independently of the main `ecc` binary for any configuration
- No POSIX shell or `jq` dependency — works on Windows, Linux, macOS
- Compile-time enforcement of valid workflow transitions via Phase enum
- WorkflowState aggregate in ecc-domain enables domain-driven modeling
- Cross-script coupling (phase-transition calling memory-writer) becomes internal function calls
- 13 shell scripts (1113 lines) replaced by compiled Rust (~2000 lines with tests)
