---
name: dependency-docs
description: Extract per-dependency purpose, usage breadth, version constraints, and lock file risk for documentation and maintenance awareness.
origin: ECC
---

# Dependency Documentation

Atomic extraction skill for understanding and documenting external dependencies — why each exists, how widely it's used, what version constraints apply, and what risks it carries.

## When to Activate

- When generating architecture documentation (dependency section)
- When auditing supply chain risk
- During onboarding (understanding why dependencies exist)
- Before major version upgrades (impact assessment)

## Methodology

### 1. Dependency Inventory

Parse the package manifest for the project's language:

| Language | Manifest | Lock File |
|----------|----------|-----------|
| TypeScript/JS | `package.json` | `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml` |
| Python | `pyproject.toml`, `requirements.txt`, `setup.py` | `poetry.lock`, `Pipfile.lock` |
| Go | `go.mod` | `go.sum` |
| Rust | `Cargo.toml` | `Cargo.lock` |
| Java | `pom.xml`, `build.gradle` | — |

Separate: production vs dev dependencies.

### 2. Usage Breadth Analysis

For each dependency, measure how deeply it's integrated:

1. **Import count**: How many source files import from this dependency?
2. **Usage pattern**: Is it used directly, or wrapped in an adapter?
3. **Replaceability**: Could it be swapped with a drop-in alternative?

Classify:

| Breadth | Criteria | Risk Level |
|---------|----------|------------|
| **Core** | Imported in 10+ files, deeply integrated | HIGH (hard to replace) |
| **Moderate** | Imported in 3-9 files | MEDIUM |
| **Peripheral** | Imported in 1-2 files, or dev-only | LOW (easy to replace) |

### 3. Purpose Inference

For each dependency, determine its role:

1. Check the dependency's package description (from registry)
2. Look at how it's imported and used in the codebase
3. Categorise:

| Category | Examples |
|----------|---------|
| Runtime framework | express, fastapi, gin |
| Data access | prisma, sqlalchemy, gorm |
| Validation | zod, pydantic, validator |
| Testing | jest, pytest, testify |
| Build tooling | typescript, webpack, esbuild |
| Linting/Formatting | eslint, prettier, golangci-lint |
| Utilities | lodash, pathlib, cobra |
| Security | helmet, bcrypt, jose |

### 4. Version Constraint Analysis

For each dependency, assess version health:

| Check | Finding |
|-------|---------|
| Pinned vs range | `"^1.2.3"` (range) vs `"1.2.3"` (pinned) |
| Major version lag | Current latest vs installed version |
| Lock file freshness | Last lock file update date |
| Known vulnerabilities | Check `npm audit`, `pip audit`, `govulncheck` output |
| Deprecation status | Is the package marked deprecated? |

### 5. Transitive Risk Assessment

For high-breadth dependencies:

- Count transitive dependencies (depth of dependency tree)
- Flag dependencies with 100+ transitive deps (supply chain risk)
- Note if alternatives exist with fewer transitive deps

## Output Format

```
# Dependency Documentation

## Summary
Total: 45 dependencies (32 production, 13 dev)
Core: 5 | Moderate: 12 | Peripheral: 28

## Production Dependencies

| Dependency | Version | Purpose | Breadth | Files | Risk |
|-----------|---------|---------|---------|-------|------|
| typescript | ^5.3.0 | Type system and compiler | Core | — (build) | LOW |
| zod | ^3.22.0 | Schema validation | Moderate | 8 | MEDIUM |
| commander | ^12.0.0 | CLI argument parsing | Peripheral | 1 | LOW |

## Dev Dependencies

| Dependency | Version | Purpose | Category |
|-----------|---------|---------|----------|
| jest | ^29.7.0 | Test runner | Testing |
| eslint | ^8.56.0 | Linting | Quality |

## Risk Assessment

| Risk | Dependencies | Action |
|------|-------------|--------|
| Core dependency, hard to replace | express, prisma, react | Monitor releases, plan upgrade path |
| 2+ major versions behind | webpack (v4 → v5 available) | Schedule upgrade |
| Deprecated | request | Replace with fetch/undici |
```

## Related

- Architecture generation: `skills/architecture-gen/SKILL.md`
- Doc analysis skill: `skills/doc-analysis/SKILL.md`
- Security review: `agents/security-reviewer.md`
- Doc analyzer agent: `agents/doc-analyzer.md`
