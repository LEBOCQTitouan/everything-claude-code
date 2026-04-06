---
name: mutation
category: testing
tags: [testing, mutation-testing, test-quality, coverage]
languages: [rust, go, python, typescript]
difficulty: advanced
unsafe-examples: true
---

## Intent

Measure the effectiveness of a test suite by introducing small faults (mutations) into the source code and checking whether the tests detect them.

## Problem

Code coverage tells you which lines are executed, not whether the tests actually verify correct behaviour. A test suite can achieve 100% line coverage while asserting nothing meaningful. You need a way to measure whether tests catch real bugs.

## Solution

A mutation testing tool creates copies of the source code with small changes (replacing `+` with `-`, `>` with `>=`, removing statements). Each mutant is run against the test suite. If the tests fail, the mutant is "killed" (good). If tests still pass, the mutant "survived" (the tests missed a real fault). The mutation score (killed/total) measures test effectiveness.

## Language Implementations

### Rust

```bash
# Install and run cargo-mutants
cargo install cargo-mutants
cargo mutants --package my-crate

# Scope to changed files only
cargo mutants --in-diff main

# Example output:
# 42 mutants tested: 38 killed, 2 caught, 2 survived
# Mutation score: 95.2%
```

### Go

```bash
# Install and run gremlins
go install github.com/go-gremlins/gremlins/cmd/gremlins@latest
gremlins unleash ./...

# Or use ooze
go install github.com/gtramontina/ooze/cmd/ooze@latest
ooze ./...
```

### Python

```bash
# Install and run mutmut
pip install mutmut
mutmut run --paths-to-mutate=src/

# Or use cosmic-ray
pip install cosmic-ray
cosmic-ray init config.toml
cosmic-ray exec config.toml
```

### Typescript

```bash
# Install and run Stryker
npx stryker init
npx stryker run

# Configuration in stryker.conf.json
# {
#   "mutator": { "excludedMutations": ["StringLiteral"] },
#   "testRunner": "vitest",
#   "coverageAnalysis": "perTest"
# }
```

## When to Use

- When you need to verify test suite quality beyond line coverage.
- As a periodic audit tool (weekly CI job, not every commit).
- When refactoring critical business logic and need confidence in tests.

## When NOT to Use

- On every commit in CI -- mutation testing is slow (minutes to hours).
- On code with low test coverage -- fix coverage first.
- For generated code or trivial boilerplate.

## Anti-Patterns

- Targeting 100% mutation score -- some mutants are equivalent (semantically identical).
- Writing tests specifically to kill mutants rather than to verify behaviour.
- Running mutation testing on the full codebase instead of scoping to changed files.

## Related Patterns

- [testing/property-based](property-based.md) -- property tests tend to kill more mutants than example tests.
- [testing/aaa](aaa.md) -- well-structured tests with strong assertions kill mutants effectively.
- [testing/table-driven](table-driven.md) -- parameterised tests cover more input space, killing more mutants.

## References

- Richard Lipton, "Fault Diagnosis of Computer Programs" (1971).
- **Rust**: `cargo-mutants`
- **Go**: `gremlins`, `ooze`
- **Python**: `mutmut`, `cosmic-ray`
- **Kotlin/Java**: PITest
- **TypeScript**: Stryker Mutator
