# Knowledge Sources

## Inbox

<!-- Add new sources here. During `ecc sources reindex`, entries are moved to the correct quadrant. -->

## Adopt

### rust-patterns
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) — type: doc | subject: rust-patterns | added: 2026-03-29 | by: human | checked: 2026-03-29

### error-handling
- [thiserror crate](https://github.com/dtolnay/thiserror) — type: package | subject: error-handling | added: 2026-03-29 | by: human | checked: 2026-03-29

## Trial

### cli-design
- [clap documentation](https://docs.rs/clap/latest/clap/) — type: doc | subject: cli-design | added: 2026-03-29 | by: human | checked: 2026-03-29

### testing
- [cargo-nextest](https://nexte.st/) — type: package | subject: testing | added: 2026-03-29 | by: human | checked: 2026-03-29

## Assess

### ai-coding
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code) — type: doc | subject: ai-coding | added: 2026-03-29 | by: human | checked: 2026-03-29

### knowledge-management
- [ThoughtWorks Technology Radar](https://www.thoughtworks.com/radar) — type: doc | subject: knowledge-management | added: 2026-03-29 | by: human | checked: 2026-03-29

## Hold

### legacy-patterns
- [anyhow in library crates](https://github.com/dtolnay/anyhow) — type: package | subject: legacy-patterns | added: 2026-03-29 | by: human | checked: 2026-03-29 | deprecated: Use thiserror enums per ADR-0029

## Module Mapping

| Module | Subjects |
|--------|----------|
| crates/ecc-domain/ | rust-patterns, error-handling, domain-modeling |
| crates/ecc-app/ | rust-patterns, testing, app-patterns |
| crates/ecc-cli/ | cli-design, rust-patterns |
| crates/ecc-infra/ | rust-patterns, testing |
| crates/ecc-ports/ | rust-patterns, domain-modeling |
| agents/ | ai-coding |
| commands/ | ai-coding |
| skills/ | ai-coding, knowledge-management |
