#!/usr/bin/env bash
# install.sh — Install claude rules while preserving directory structure.
#
# Usage:
#   ./install.sh <language> [<language> ...]
#
# Examples:
#   ./install.sh typescript
#   ./install.sh typescript python golang
#
# This script copies rules into ~/.claude/rules/ keeping the common/ and
# language-specific subdirectories intact so that:
#   1. Files with the same name in common/ and <language>/ don't overwrite
#      each other.
#   2. Relative references (e.g. ../common/coding-style.md) remain valid.

set -euo pipefail

# Resolve symlinks — needed when invoked as `ecc-install` via npm/bun bin symlink
SCRIPT_PATH="$0"
while [ -L "$SCRIPT_PATH" ]; do
    link_dir="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
    SCRIPT_PATH="$(readlink "$SCRIPT_PATH")"
    # Resolve relative symlinks
    [[ "$SCRIPT_PATH" != /* ]] && SCRIPT_PATH="$link_dir/$SCRIPT_PATH"
done
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
RULES_DIR="$SCRIPT_DIR/06-rules"

# --- Usage ---
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <language> [<language> ...]"
    echo ""
    echo "Available languages:"
    for dir in "$RULES_DIR"/*/; do
        name="$(basename "$dir")"
        [[ "$name" == "common" ]] && continue
        echo "  - $name"
    done
    exit 1
fi

DEST_DIR="${CLAUDE_RULES_DIR:-$HOME/.claude/rules}"

# Warn if destination already exists (user may have local customizations)
if [[ -d "$DEST_DIR" ]] && [[ "$(ls -A "$DEST_DIR" 2>/dev/null)" ]]; then
    echo "Note: $DEST_DIR/ already exists. Existing files will be overwritten."
    echo "      Back up any local customizations before proceeding."
fi

# Always install common rules
echo "Installing common rules -> $DEST_DIR/common/"
mkdir -p "$DEST_DIR/common"
cp -r "$RULES_DIR/common/." "$DEST_DIR/common/"

# Install each requested language
for lang in "$@"; do
    # Validate language name to prevent path traversal
    if [[ ! "$lang" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        echo "Error: invalid language name '$lang'. Only alphanumeric, dash, and underscore allowed." >&2
        continue
    fi
    lang_dir="$RULES_DIR/$lang"
    if [[ ! -d "$lang_dir" ]]; then
        echo "Warning: 06-rules/$lang/ does not exist, skipping." >&2
        continue
    fi
    echo "Installing $lang rules -> $DEST_DIR/$lang/"
    mkdir -p "$DEST_DIR/$lang"
    cp -r "$lang_dir/." "$DEST_DIR/$lang/"
done

echo "Done. Rules installed to $DEST_DIR/"
