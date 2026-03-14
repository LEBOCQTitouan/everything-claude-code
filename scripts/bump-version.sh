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

echo "Bumping all versions to $VERSION..."

# 1. Cargo.toml workspace version
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak
echo "  Updated Cargo.toml"

# 2. Root package.json
node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  pkg.version = '$VERSION';
  fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "  Updated package.json"

# 3. npm/ecc/package.json (version + optionalDependencies)
node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('npm/ecc/package.json', 'utf8'));
  pkg.version = '$VERSION';
  for (const dep of Object.keys(pkg.optionalDependencies || {})) {
    pkg.optionalDependencies[dep] = '$VERSION';
  }
  fs.writeFileSync('npm/ecc/package.json', JSON.stringify(pkg, null, 2) + '\n');
"
echo "  Updated npm/ecc/package.json"

# 4. Platform package.json files
PLATFORMS=(
  "ecc-darwin-arm64"
  "ecc-darwin-x64"
  "ecc-linux-x64"
  "ecc-linux-arm64"
  "ecc-win32-x64"
)

for pkg in "${PLATFORMS[@]}"; do
  PKG_FILE="npm/$pkg/package.json"
  if [ -f "$PKG_FILE" ]; then
    sed -i.bak "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" "$PKG_FILE"
    rm -f "$PKG_FILE.bak"
    echo "  Updated $PKG_FILE"
  else
    echo "  Warning: $PKG_FILE not found, skipping"
  fi
done

echo ""
echo "All versions updated to $VERSION"
echo "Next: git add -A && git commit -m 'chore: bump version to $VERSION'"
