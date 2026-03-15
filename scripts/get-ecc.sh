#!/bin/sh
# get-ecc.sh — Install ECC (Everything Claude Code) from GitHub Releases
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh | bash
#
# Environment variables:
#   ECC_VERSION  — specific version to install (default: latest)
#   ECC_INSTALL_DIR — install directory (default: $HOME/.ecc)

set -eu

REPO="LEBOCQTitouan/everything-claude-code"
INSTALL_DIR="${ECC_INSTALL_DIR:-$HOME/.ecc}"

# --- Detect platform ---
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin) OS_NAME="darwin" ;;
        Linux)  OS_NAME="linux" ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "Error: Windows is not supported by this installer. Download from GitHub Releases." >&2
            exit 1
            ;;
        *)
            echo "Error: Unsupported OS: $OS" >&2
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64) ARCH_NAME="x64" ;;
        arm64|aarch64) ARCH_NAME="arm64" ;;
        *)
            echo "Error: Unsupported architecture: $ARCH" >&2
            exit 1
            ;;
    esac

    ARTIFACT="ecc-${OS_NAME}-${ARCH_NAME}"
    echo "$ARTIFACT"
}

# --- Resolve version ---
resolve_version() {
    if [ -n "${ECC_VERSION:-}" ]; then
        echo "$ECC_VERSION"
        return
    fi

    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/')

    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version from GitHub." >&2
        exit 1
    fi
    echo "$VERSION"
}

# --- Main ---
main() {
    ARTIFACT=$(detect_platform)
    VERSION=$(resolve_version)
    TAG="v${VERSION}"

    TARBALL_URL="https://github.com/${REPO}/releases/download/${TAG}/${ARTIFACT}.tar.gz"

    echo "Installing ECC ${VERSION} (${ARTIFACT})..."
    echo "  From: ${TARBALL_URL}"
    echo "  To:   ${INSTALL_DIR}"
    echo ""

    # Create install directory
    mkdir -p "${INSTALL_DIR}/bin"

    # Download and extract
    TMPDIR_DL=$(mktemp -d)
    trap 'rm -rf "$TMPDIR_DL"' EXIT

    if ! curl -fsSL "$TARBALL_URL" -o "${TMPDIR_DL}/ecc.tar.gz"; then
        echo "Error: Failed to download ${TARBALL_URL}" >&2
        echo "Check that version ${VERSION} exists at https://github.com/${REPO}/releases" >&2
        exit 1
    fi

    tar -xzf "${TMPDIR_DL}/ecc.tar.gz" -C "${INSTALL_DIR}"

    # Ensure binaries are executable
    chmod +x "${INSTALL_DIR}/bin/ecc"
    if [ -f "${INSTALL_DIR}/bin/ecc-hook" ]; then
        chmod +x "${INSTALL_DIR}/bin/ecc-hook"
    fi
    if [ -f "${INSTALL_DIR}/bin/ecc-shell-hook.sh" ]; then
        chmod +x "${INSTALL_DIR}/bin/ecc-shell-hook.sh"
    fi

    echo "Installed ECC ${VERSION} to ${INSTALL_DIR}"
    echo ""

    # Add to PATH if not already present
    BIN_DIR="${INSTALL_DIR}/bin"
    add_to_path "$BIN_DIR"

    echo "Done! Run 'ecc version' to verify."
    echo ""
    echo "Next steps:"
    echo "  ecc install            # Install agents, skills, rules to ~/.claude/"
    echo "  ecc init               # Set up current project"
    echo "  ecc help               # Show all commands"
}

add_to_path() {
    BIN="$1"

    # Check if already in PATH
    case ":${PATH}:" in
        *":${BIN}:"*) return ;;
    esac

    SHELL_NAME="$(basename "${SHELL:-/bin/sh}")"
    RC_FILE=""

    case "$SHELL_NAME" in
        zsh)  RC_FILE="$HOME/.zshrc" ;;
        bash)
            if [ -f "$HOME/.bash_profile" ]; then
                RC_FILE="$HOME/.bash_profile"
            else
                RC_FILE="$HOME/.bashrc"
            fi
            ;;
        fish) RC_FILE="$HOME/.config/fish/config.fish" ;;
        *)    RC_FILE="$HOME/.profile" ;;
    esac

    if [ -n "$RC_FILE" ]; then
        LINE="export PATH=\"${BIN}:\$PATH\""
        if [ "$SHELL_NAME" = "fish" ]; then
            LINE="set -gx PATH ${BIN} \$PATH"
        fi

        if [ -f "$RC_FILE" ] && grep -qF "$BIN" "$RC_FILE" 2>/dev/null; then
            return
        fi

        echo "" >> "$RC_FILE"
        echo "# ECC (Everything Claude Code)" >> "$RC_FILE"
        echo "$LINE" >> "$RC_FILE"
        echo "Added ${BIN} to PATH in ${RC_FILE}"
        echo "Run 'source ${RC_FILE}' or open a new terminal to use ecc."
    fi
}

main
