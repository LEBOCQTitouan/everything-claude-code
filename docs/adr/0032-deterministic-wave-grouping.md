# ADR-032: Deterministic Wave Grouping Algorithm

## Status

Accepted (2026-03-29)

## Context

The `/implement` command groups Pass Conditions into parallel execution waves based on file overlap. Previously, the LLM manually evaluated file changes, identified overlaps, and formed groups — taking 5-10 seconds, producing non-deterministic results, and often under-parallelizing.

## Decision

- **Greedy first-fit bin-packing with non-adjacent grouping**: For each PC in input order, try to place it in the first existing wave where no file overlap exists and wave size < max_per_wave. If no wave fits, create a new one. Non-adjacent PCs can share waves for better parallelism.
- **Domain algorithm in `ecc-domain::spec::wave`**: Pure `(&[PassCondition], &[FileChange], usize) -> WavePlan` with zero I/O. PC-to-files mapping built via AC cross-reference between the Pass Conditions and File Changes tables.
- **CLI subcommand in `ecc-workflow`**: `ecc-workflow wave-plan <design-path>` reads the design file, calls the domain function, outputs JSON. Consistent with other implement-phase commands.
- **`max_per_wave` parameterized** (default 4): Domain function accepts configurable maximum. The CLI currently hardcodes 4.
- **Hard requirement**: `/implement` requires the binary — no manual LLM fallback.

## Consequences

- Same input always produces same output — deterministic across sessions
- Wave computation takes <50ms vs 5-10s for LLM analysis
- Non-adjacent grouping produces fewer waves than the previous adjacent-only scan
- File path normalization (backtick stripping) prevents false non-overlaps
- PCs with no file matches are treated as independent (no overlaps)
- `max_per_wave=0` is safely treated as `max_per_wave=1` (no panic)
