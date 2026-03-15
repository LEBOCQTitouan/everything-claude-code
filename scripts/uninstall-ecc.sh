#!/usr/bin/env bash
# uninstall-ecc.sh — Remove ECC (Everything Claude Code) from the system
#
# Usage:
#   scripts/uninstall-ecc.sh [--force] [--keep-config]
#
# Flags:
#   --force        Skip confirmation prompt
#   --keep-config  Only remove ~/.ecc/ and PATH entries; keep ~/.claude/ artifacts

set -euo pipefail

INSTALL_DIR="${ECC_INSTALL_DIR:-$HOME/.ecc}"
CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"

# ---------------------------------------------------------------------------
# Parse flags
# ---------------------------------------------------------------------------
FORCE=""
KEEP_CONFIG=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --force)       FORCE=1; shift ;;
        --keep-config) KEEP_CONFIG=1; shift ;;
        -h|--help)
            echo "Usage: uninstall-ecc.sh [--force] [--keep-config]"
            echo ""
            echo "Flags:"
            echo "  --force        Skip confirmation prompt"
            echo "  --keep-config  Only remove ~/.ecc/ and PATH; keep ~/.claude/ artifacts"
            exit 0
            ;;
        *)
            echo "Error: Unknown flag '$1'" >&2
            exit 1
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Shell RC files that may contain PATH entries
# ---------------------------------------------------------------------------
RC_FILES=(
    "$HOME/.zshrc"
    "$HOME/.bashrc"
    "$HOME/.bash_profile"
    "$HOME/.config/fish/config.fish"
    "$HOME/.profile"
)

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo "ECC Uninstaller"
echo "==============="
echo ""
echo "This will remove:"
echo "  - ${INSTALL_DIR}/  (binary + bundled content)"

if [[ -z "$KEEP_CONFIG" ]]; then
    echo "  - ECC artifacts from ${CLAUDE_DIR}/ (agents, commands, skills, rules, manifest)"
fi

# Find RC files with PATH entries
RC_MATCHES=()
for rc in "${RC_FILES[@]}"; do
    if [[ -f "$rc" ]] && grep -q '\.ecc/bin' "$rc" 2>/dev/null; then
        RC_MATCHES+=("$rc")
    fi
done

if [[ ${#RC_MATCHES[@]} -gt 0 ]]; then
    echo "  - PATH entries from:"
    for rc in "${RC_MATCHES[@]}"; do
        echo "      $rc"
    done
fi

echo ""

# ---------------------------------------------------------------------------
# Confirmation
# ---------------------------------------------------------------------------
if [[ -z "$FORCE" ]]; then
    printf "Proceed? [y/N] "
    read -r answer </dev/tty
    case "${answer:-}" in
        [Yy]*) ;;
        *)
            echo "Aborted."
            exit 0
            ;;
    esac
    echo ""
fi

# ---------------------------------------------------------------------------
# Step 1: Clean ECC artifacts from ~/.claude/ (unless --keep-config)
# ---------------------------------------------------------------------------
if [[ -z "$KEEP_CONFIG" ]]; then
    if command -v ecc &>/dev/null; then
        echo "Cleaning ECC artifacts from ${CLAUDE_DIR}/ via ecc..."
        ecc install --clean-all --no-interactive 2>/dev/null || true
    else
        # Manual cleanup if ecc binary is already gone
        echo "Cleaning ECC artifacts from ${CLAUDE_DIR}/ manually..."
        rm -rf "${CLAUDE_DIR}/agents" \
               "${CLAUDE_DIR}/commands" \
               "${CLAUDE_DIR}/skills" \
               "${CLAUDE_DIR}/rules"
        rm -f "${CLAUDE_DIR}/.ecc-manifest.json"
    fi
    echo "  Done."
fi

# ---------------------------------------------------------------------------
# Step 2: Remove ~/.ecc/ directory
# ---------------------------------------------------------------------------
if [[ -d "$INSTALL_DIR" ]]; then
    echo "Removing ${INSTALL_DIR}/..."
    rm -rf "$INSTALL_DIR"
    echo "  Done."
else
    echo "Skipping ${INSTALL_DIR}/ (not found)."
fi

# ---------------------------------------------------------------------------
# Step 3: Remove PATH entries from shell RC files
# ---------------------------------------------------------------------------
for rc in "${RC_MATCHES[@]}"; do
    echo "Cleaning PATH entry from ${rc}..."
    # Remove the ECC comment line and the PATH/set line that follows it
    if [[ "$(uname -s)" == "Darwin" ]]; then
        sed -i '' '/# ECC (Everything Claude Code)/d' "$rc"
        sed -i '' '/\.ecc\/bin/d' "$rc"
    else
        sed -i '/# ECC (Everything Claude Code)/d' "$rc"
        sed -i '/\.ecc\/bin/d' "$rc"
    fi
    echo "  Done."
done

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
echo "ECC has been uninstalled."
if [[ -n "$KEEP_CONFIG" ]]; then
    echo "Note: ~/.claude/ artifacts were preserved (--keep-config)."
fi
echo "Open a new terminal or source your shell RC file to update PATH."
