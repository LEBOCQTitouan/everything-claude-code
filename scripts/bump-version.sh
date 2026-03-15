#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 4.0.0-alpha.2"
  exit 1
fi

VERSION="$1"

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-.+)?$ ]]; then
  echo "Error: Invalid version format: $VERSION (expected X.Y.Z or X.Y.Z-suffix)"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

echo "Bumping version to $VERSION..."

# Cargo.toml workspace version
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak
echo "  Updated Cargo.toml"

echo ""
echo "Version updated to $VERSION"
echo "Next: git add Cargo.toml Cargo.lock && git commit -m 'chore: bump version to $VERSION'"
