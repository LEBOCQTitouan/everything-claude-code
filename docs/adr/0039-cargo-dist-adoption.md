# ADR 0039: cargo-dist Adoption for Binary Distribution

## Status
Accepted

## Context
The ECC release pipeline was a hand-rolled 267-line GitHub Actions workflow managing cross-compilation for 5 targets, tarball packaging with content directories, checksum generation, and optional cosign signing. It had been substantially rewritten 11 times, with fragile conditional logic (silent failures for missing binaries and unsigned releases). cargo-dist provides a declarative alternative with auto-generated CI, built-in checksums, installer generation, and SBOM support.

## Decision
Adopt cargo-dist as the release pipeline tool. Configuration lives in `dist.toml` at the workspace root. The generated `release.yml` follows cargo-dist's plan/build/host/announce pipeline structure. Content directories (agents, commands, skills, rules, hooks, etc.) are bundled via the `include` configuration in dist.toml. Custom cosign signing is added as a post-build job (see ADR 0040).

## Consequences
- Release pipeline reduced from hand-maintained YAML to declarative config
- Gains: shell/PowerShell installers, standardized archive structure, future SBOM support
- Trade-off: less fine-grained CI control than hand-rolled workflow
- cargo-dist regeneration (`cargo dist generate-ci`) may overwrite custom modifications — custom jobs declared in dist.toml `plan.jobs` to mitigate
- First release with cargo-dist must be a pre-release (v*-rc1) for verification
