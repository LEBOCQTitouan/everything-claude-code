---
description: "Run mutation testing via cargo-mutants"
allowed-tools: [Bash, Read, Grep, Glob, TodoWrite, TodoRead]
---

# Mutants

Run mutation testing against ECC crates using cargo-mutants. Detects untested code paths by injecting mutations and verifying test failures.

## Arguments

`$ARGUMENTS` supports:
- `<package>` — target a specific crate (e.g., `ecc-domain`, `ecc-app`)
- `--diff` — run only on code changed vs origin/main (fast, PR-scoped)
- No arguments — run against default crates (ecc-domain + ecc-app)

## Prerequisites

cargo-mutants must be installed: `cargo install cargo-mutants`

## Steps

1. Check cargo-mutants is installed. If missing, print:
   > "cargo-mutants not installed. Run: `cargo install cargo-mutants`"
   Then STOP.

2. Run mutation testing via xtask:
   - Default: `cargo xtask mutants`
   - Targeted: `cargo xtask mutants --package <package>`
   - Diff-scoped: `cargo xtask mutants --in-diff`

3. Parse the output and present a summary:
   ```
   Mutation Testing Results
   ━━━━━━━━━━━━━━━━━━━━━━━
     Package: <package>
     Total mutants: N
     Killed: N (N%)
     Survived: N (N%)
     Timed out: N (N%)
     Unviable: N
   ━━━━━━━━━━━━━━━━━━━━━━━
   ```

4. List top surviving mutants (if any) with file paths and line numbers.

5. Suggest next steps:
   - If survived > 0: "Write tests to catch these surviving mutants"
   - If all killed: "All mutations caught — test suite is strong"

## Notes

- Mutation testing is compute-intensive. Expect 15-60 minutes for full runs.
- Use `--diff` mode for fast feedback during development.
- Results are stored in `mutants.out/` (gitignored).
