#!/usr/bin/env bash
# Thin wrapper that resolves ECC_ROOT from the package location,
# then delegates to scripts/hooks/run-with-flags-shell.sh.
# Registered as a bin entry so hooks.json can use `ecc-shell-hook` instead of
# relying on an ECC_ROOT environment variable.

set -euo pipefail

# Resolve symlinks so we always find the real package root
SCRIPT_PATH="${BASH_SOURCE[0]}"
while [ -L "$SCRIPT_PATH" ]; do
    link_dir="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
    SCRIPT_PATH="$(readlink "$SCRIPT_PATH")"
    [[ "$SCRIPT_PATH" != /* ]] && SCRIPT_PATH="$link_dir/$SCRIPT_PATH"
done
BIN_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"

export ECC_ROOT="${ECC_ROOT:-$(dirname "$BIN_DIR")}"

exec "$ECC_ROOT/scripts/hooks/run-with-flags-shell.sh" "$@"
