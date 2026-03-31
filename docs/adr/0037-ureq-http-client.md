# ADR 0037: Use ureq as HTTP Client for ecc update

## Status

Accepted (2026-03-31)

## Context

The `ecc update` command needs an HTTP client to download release artifacts from GitHub Releases. Three options were evaluated:

- **reqwest**: Feature-rich async HTTP client, but pulls in tokio, hyper, and ~60 transitive dependencies. Adds ~15s to clean builds. Overkill for simple GET requests with no concurrent I/O needs.
- **self_update**: Purpose-built self-update crate, but tightly coupled to its own update flow (asset naming, archive format, binary replacement). Does not support custom verification steps like cosign signature checking. Last meaningful update was 2023.
- **ureq**: Minimal blocking HTTP client with ~15 dependencies. Supports TLS via rustls (no OpenSSL linking). Clean API for streaming response bodies to disk. 1.0-stable since 2021.

Our usage is limited: download a tarball, download a checksum file, download a cosign bundle. All sequential, no concurrency needed. The binary must stay small for fast self-replacement.

## Decision

Use `ureq` as the HTTP client for the update module.

## Consequences

- Binary size stays small — no async runtime or tower middleware pulled in
- Build times remain fast — ~15 dependencies vs ~60 for reqwest
- Blocking I/O is acceptable since `ecc update` is an interactive CLI command with sequential downloads
- rustls-based TLS avoids OpenSSL linking issues on cross-compiled targets
- If future features need async HTTP (e.g., parallel downloads), migration to reqwest would be required
- No built-in retry or connection pooling — acceptable for single-use download requests
