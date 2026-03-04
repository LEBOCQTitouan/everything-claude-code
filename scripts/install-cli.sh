#!/usr/bin/env bash
# install-cli.sh — Install the ecc CLI on Linux (or Mac without Homebrew)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/install-cli.sh | bash
#
# Environment variables:
#   ECC_HOME   Where to install source files. Default: ~/.local/share/ecc
#   BIN_DIR    Where to install the ecc binary.  Default: ~/.local/bin

set -euo pipefail

REPO="LEBOCQTitouan/everything-claude-code"
BRANCH="main"
ECC_HOME="${ECC_HOME:-$HOME/.local/share/ecc}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[ecc]${RESET} $*"; }
success() { echo -e "${GREEN}[ecc]${RESET} $*"; }
die()     { echo -e "${RED}[ecc] Error:${RESET} $*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Check dependencies
# ---------------------------------------------------------------------------
command -v curl &>/dev/null || die "curl is required. Install it and retry."
command -v tar  &>/dev/null || die "tar is required. Install it and retry."

# ---------------------------------------------------------------------------
# Download
# ---------------------------------------------------------------------------
TARBALL_URL="https://github.com/$REPO/archive/refs/heads/$BRANCH.tar.gz"

info "Downloading everything-claude-code ($BRANCH)..."
mkdir -p "$ECC_HOME"

curl -fsSL "$TARBALL_URL" \
    | tar -xz --strip-components=1 -C "$ECC_HOME"

chmod +x "$ECC_HOME/install.sh"

# ---------------------------------------------------------------------------
# Create ecc binary
# ---------------------------------------------------------------------------
mkdir -p "$BIN_DIR"

cat > "$BIN_DIR/ecc" <<EOF
#!/usr/bin/env bash
exec "$ECC_HOME/install.sh" "\$@"
EOF

chmod +x "$BIN_DIR/ecc"

# ---------------------------------------------------------------------------
# PATH reminder
# ---------------------------------------------------------------------------
success "ecc installed to $BIN_DIR/ecc"

if ! echo ":$PATH:" | grep -q ":$BIN_DIR:"; then
    echo ""
    echo "  Add $BIN_DIR to your PATH:"
    echo ""
    if [[ -n "${ZSH_VERSION:-}" ]] || [[ "$SHELL" == */zsh ]]; then
        echo "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
    else
        echo "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
    fi
    echo ""
fi

success "Done. Run 'ecc' to get started."
