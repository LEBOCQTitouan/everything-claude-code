# ADR-0037: Unify ecc-hook, ecc-workflow, and ecc CLI into a single binary

## Status
Accepted (2026-04-02)

## Context
The ECC system had three separate binaries: `ecc` (CLI), `ecc-hook` (hook dispatch), and `ecc-workflow` (workflow state machine). This caused:
- No shared process context between hook enforcement and workflow state
- Two divergent I/O strategies (hexagonal vs raw filesystem)
- Triple deployment surface

## Decision
Merge all functionality into the single `ecc` binary:
- `ecc hook <id>` replaces `ecc-hook <id>`
- `ecc workflow <subcommand>` replaces `ecc-workflow <subcommand>`
- Old binaries become thin wrappers during migration, removed in cleanup PR

## Consequences
- Single binary to deploy and manage
- Shared process context enables hooks to read workflow state directly
- Larger binary size (mitigated by LTO)
- Migration requires thin wrapper compatibility period
