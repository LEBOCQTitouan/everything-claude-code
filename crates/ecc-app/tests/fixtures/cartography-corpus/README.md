# Cartography Corpus Fixtures

10 representative session-delta fixtures for regression testing of `is_noise_path` + `stop_cartography` filter.

## Signal vs Noise

A path is **noise** if it matches `ecc_domain::cartography::is_noise_path` — currently prefixes under
`.claude/workflow/`, `.claude/cartography/`, `.claude/worktrees/`, `docs/specs/`, `docs/backlog/`,
`docs/cartography/`, or exact matches `Cargo.lock` and `.claude/workflow`.

A path is **signal** otherwise (crate source, non-spec docs, CI config, etc.).

## Adding fixtures

1. Add a new JSON file with the `SessionDelta` shape (see existing fixtures)
2. Add a matching entry to `expected.yaml`
3. Run `cargo test -p ecc-app --test cartography_corpus` to verify
