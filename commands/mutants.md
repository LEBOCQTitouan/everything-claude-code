---
description: "Run mutation testing via cargo-mutants"
allowed-tools: [Bash, Read, Grep, Glob, TodoWrite, TodoRead]
---

# Mutants

Run mutation testing against ECC crates.

## Arguments

- `<package>` — target specific crate
- `--diff` — changed files vs origin/main only (fast)
- No args — ecc-domain + ecc-app

## Prerequisites

`cargo install cargo-mutants`

## Steps

1. Check installed. If missing → stop.
2. Run: `cargo xtask mutants [--package <pkg>] [--in-diff]`
3. Present summary: package, total/killed/survived/timed-out/unviable
4. List surviving mutants with file:line
5. Next steps: write tests for survivors or "test suite is strong"

## Notes

- 15-60 min for full runs. Use `--diff` for fast feedback.
- Results in `mutants.out/` (gitignored).
