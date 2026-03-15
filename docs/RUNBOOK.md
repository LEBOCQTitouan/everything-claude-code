# Runbook — Everything Claude Code

Operational procedures for building, installing, troubleshooting, and maintaining ECC.

## Build

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
# Binary at: target/release/ecc
```

### Cross-Compilation

The CI pipeline builds for:
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-unknown-linux-gnu` (Linux x64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)

## Install

### From Release (end users)

```bash
curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh | bash
```

This installs the binary to `~/.ecc/bin/ecc` and adds it to PATH.

### From Source (developers)

```bash
cargo install --path crates/ecc-cli
```

### Install Content to ~/.claude/

```bash
ecc install typescript          # common + TypeScript rules
ecc install typescript python   # multiple language stacks
```

### Per-Project Setup

```bash
cd /your/project
ecc init                        # auto-detect language
ecc init golang                 # specify language
ecc init --template go-microservice golang
```

## Testing

### Run All Tests

```bash
cargo test
# Expected: 999 passed, 0 failed, 3 ignored
```

### Run Tests for a Specific Crate

```bash
cargo test -p ecc-domain       # 515 tests
cargo test -p ecc-app          # 466 tests
cargo test -p ecc-cli          # 13 tests
cargo test -p ecc-ports        # 3 tests (3 ignored — require OS)
cargo test -p ecc-infra        # 2 tests
```

### Lint

```bash
cargo clippy -- -D warnings    # Must produce zero warnings
```

### Markdown Lint

```bash
npm run lint                   # markdownlint on all .md files
```

## Common Issues

### Build Fails: "edition 2024 not supported"

**Cause:** Rust toolchain too old. Edition 2024 requires Rust 1.85+.

**Fix:**
```bash
rustup update stable
rustc --version  # Should be 1.85.0+
```

### `ecc install` Fails: Permission Denied

**Cause:** `~/.claude/` directory has wrong permissions.

**Fix:**
```bash
chmod -R u+rwX ~/.claude/
```

### `ecc hook` Fails: "hook not found"

**Cause:** Hook ID doesn't match any entry in `hooks.json`.

**Fix:**
```bash
ecc validate hooks   # List valid hook IDs
```

### Tests Fail: 3 Ignored Tests

**Expected.** The 3 ignored tests in `ecc-ports` require real OS filesystem access and are skipped in CI. They pass when run manually:

```bash
cargo test -p ecc-ports -- --ignored
```

### Clippy Warnings After Dependency Update

**Fix:**
```bash
cargo update          # Update Cargo.lock
cargo clippy -- -D warnings
```

## Configuration

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `ECC_ROOT` | Override ECC package root directory | Auto-detected |
| `ECC_HOOK_PROFILE` | Hook execution mode: `minimal`, `standard`, `strict` | `standard` |
| `ECC_DISABLED_HOOKS` | Comma-separated list of hook IDs to skip | (none) |

### Key Files

| File | Location | Purpose |
|------|----------|---------|
| `~/.ecc/bin/ecc` | User home | ECC binary |
| `~/.claude/agents/` | User home | Installed agent definitions |
| `~/.claude/commands/` | User home | Installed slash commands |
| `~/.claude/settings.json` | User home | Claude Code settings (includes hooks) |
| `.ecc-manifest.json` | User home | Tracks installed artifacts |
| `CLAUDE.md` | Project root | Project-specific instructions |

## Rollback

### Rollback Content Installation

ECC tracks installed artifacts in `.ecc-manifest.json`. To remove all ECC-managed files:

```bash
ecc install --clean-all
```

For surgical removal (only ECC-tracked files):

```bash
ecc install --clean
```

### Rollback Binary

```bash
rm ~/.ecc/bin/ecc
# Remove PATH entry from ~/.zshrc or ~/.bashrc
```

### Rollback to Previous Version

```bash
# Check current version
ecc version

# Install specific version via curl
curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/v3.0.0/scripts/get-ecc.sh | bash
```

## Health Check

### Audit Installed Configuration

```bash
ecc audit
```

Checks for:
- Stale or missing agents/commands/skills
- Outdated hooks
- Manifest integrity
- Configuration drift

### Validate Content Files

```bash
ecc validate agents     # Check agent markdown files
ecc validate commands   # Check command files
ecc validate hooks      # Check hook definitions
ecc validate skills     # Check skill directories
ecc validate rules      # Check rule files
ecc validate paths      # Check all file paths
```

## Release Process

1. Update version in `Cargo.toml` workspace
2. Run full test suite: `cargo test`
3. Run lint: `cargo clippy -- -D warnings`
4. Build release: `cargo build --release`
5. Tag release: `git tag v4.0.0`
6. Push tag: `git push origin v4.0.0`
7. CI builds and publishes binaries to GitHub Releases
