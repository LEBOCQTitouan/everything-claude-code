# Contributing to Everything Claude Code

## Prerequisites

- Rust 1.85+ (edition 2024)
- Cargo (comes with Rust)
- Claude Code CLI v2.1.0+
- Git

## Getting Started

```bash
git clone https://github.com/LEBOCQTitouan/everything-claude-code.git
cd everything-claude-code
cargo build
cargo test
```

## Project Structure

The project is a Cargo workspace with 6 crates following Hexagonal Architecture:

| Crate | Role | Can Depend On |
|-------|------|---------------|
| `ecc-domain` | Pure business logic, zero I/O | Nothing (leaf) |
| `ecc-ports` | Trait definitions for I/O | `ecc-domain` |
| `ecc-app` | Use case orchestration | `ecc-domain`, `ecc-ports` |
| `ecc-infra` | Production OS adapters | `ecc-ports` |
| `ecc-cli` | CLI binary entry point | `ecc-app`, `ecc-infra` |
| `ecc-test-support` | Test doubles | `ecc-ports` |

**Dependency rule:** Dependencies flow inward. Domain has no dependencies. Infra depends on ports, never on app or domain directly.

## Development Workflow

### 1. Plan First

For non-trivial changes, use `/plan` to create an implementation plan before writing code.

### 2. Test-Driven Development

All changes must follow TDD:

```bash
# Write tests first (RED)
cargo test --lib -- my_new_test  # Should fail

# Implement (GREEN)
cargo test --lib -- my_new_test  # Should pass

# Refactor (IMPROVE)
cargo test  # All tests should pass
```

### 3. Build and Lint

```bash
cargo build            # Check compilation
cargo clippy -- -D warnings  # Zero-warning lint
cargo test             # All 999 tests pass
```

### 4. Commit

Follow conventional commits:

```
feat: add new validation rule for agents
fix: handle empty config in audit
refactor: extract hook profile logic
test: add edge case for manifest parsing
docs: update architecture diagram
chore: bump serde to 1.0.200
```

One concern per commit. See `rules/common/git-workflow.md` for details.

## Content Contributions

### Agents (`agents/*.md`)

Markdown files with YAML frontmatter:

```yaml
---
name: my-agent
description: What this agent does
tools: ["Read", "Grep", "Glob"]
model: sonnet
---
```

### Skills (`skills/*/SKILL.md`)

Domain knowledge in a directory with a `SKILL.md` file. Include sections for when to use, how it works, and examples.

### Commands (`commands/*.md`)

Slash commands that orchestrate agents and skills. Keep to 6 active commands maximum.

### Rules (`rules/<language>/*.md`)

Always-follow guidelines grouped by language. `common/` rules apply to all projects.

## Testing Requirements

- Minimum 80% test coverage
- Unit tests in each module (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directories
- All I/O abstracted behind port traits for testability

## Code Quality Checklist

Before submitting a PR:

- [ ] `cargo test` passes (999+ tests)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo build --release` succeeds
- [ ] No hardcoded secrets or paths
- [ ] Functions < 50 lines, files < 800 lines
- [ ] Immutable data patterns (no mutation)
- [ ] Error handling is explicit (no silent swallows)
- [ ] Doc comments (`///`) on all public items

## Pull Request Process

1. Create a branch from `main`
2. Make atomic commits (one concern per commit)
3. Ensure all checks pass
4. Write a clear PR description with test plan
5. Request review

## Architecture Decisions

Significant architectural decisions should be discussed in an issue before implementation. The project follows:

- **Hexagonal Architecture** — all I/O behind port traits
- **Domain-Driven Design** — pure domain logic in `ecc-domain`
- **Clean Code** — SOLID principles, small functions, meaningful names
