---
name: github-actions-rust
description: Cross-compilation matrix, cargo caching, and release patterns for Rust GitHub Actions workflows.
origin: ECC
---

# GitHub Actions — Rust

Rust-specific patterns for CI/CD workflows: cross-compilation, caching, and release artifact packaging.

## Cross-Compilation Matrix

Use `strategy.matrix.include` for explicit target control:

```yaml
strategy:
  fail-fast: true
  matrix:
    include:
      - target: aarch64-apple-darwin
        os: macos-latest
      - target: x86_64-apple-darwin
        os: macos-latest
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: aarch64-unknown-linux-gnu
        os: ubuntu-latest
        cross: true
      - target: x86_64-pc-windows-msvc
        os: windows-latest
```

Use `cross: true` flag to select `cross build` vs native `cargo build`. The `houseabsolute/actions-rust-cross` action auto-detects this.

## Cargo Caching

Use `Swatinem/rust-cache@v2` — optimized for Rust, handles `target/` and `~/.cargo/` automatically:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    key: ${{ matrix.target }}
```

Cache key should include target triple. For workspaces, add `workspaces: "crate-name -> target"`.

## Release Artifacts

Package binaries with checksums for GitHub Releases:

```yaml
- name: Package
  run: |
    STAGING=$(mktemp -d)
    cp target/${{ matrix.target }}/release/mybin "$STAGING/"
    cd "$STAGING" && tar czf ../mybin-${{ matrix.target }}.tar.gz *
    sha256sum ../mybin-${{ matrix.target }}.tar.gz > ../mybin-${{ matrix.target }}.tar.gz.sha256

- uses: actions/upload-artifact@v4
  with:
    name: release-${{ matrix.target }}
    path: mybin-${{ matrix.target }}.tar.gz*
```

Use `softprops/action-gh-release` to publish artifacts to a GitHub Release from a tag-triggered workflow.

## Testing Matrix

```yaml
- run: cargo test --workspace
- run: cargo clippy --workspace -- -D warnings
- run: cargo fmt --check
```

Run `cargo deny check` for license + advisory audit. Add `cargo audit` as a separate job for supply chain security.
