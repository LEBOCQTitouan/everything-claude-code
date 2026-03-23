#!/usr/bin/env bash
set -euo pipefail

# ECC Read Context Percentage — pure-function reader for context window percentage.
# Returns integer 0-100 or "unknown".

# --- Sanitize session ID ---
RAW_SESSION="${CLAUDE_SESSION_ID:-$PPID}"
SESSION_ID=$(printf '%s' "$RAW_SESSION" | tr -dc 'a-zA-Z0-9_-')
if [ -z "$SESSION_ID" ]; then
  echo "unknown"
  exit 0
fi

# --- Resolve runtime directory ---
FALLBACK_BASE="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}"
RUNTIME_DIR="${ECC_RUNTIME_DIR:-${FALLBACK_BASE}/ecc-$(id -u)}"

# --- Read file ---
TARGET="${RUNTIME_DIR}/ecc-context-${SESSION_ID}.pct"
if [ ! -f "$TARGET" ] || [ ! -r "$TARGET" ]; then
  echo "unknown"
  exit 0
fi

VALUE=$(cat "$TARGET" 2>/dev/null || true)

# --- Validate: integer 0-100 ---
if ! printf '%s' "$VALUE" | grep -qE '^[0-9]{1,3}$'; then
  echo "unknown"
  exit 0
fi

if [ "$VALUE" -lt 0 ] || [ "$VALUE" -gt 100 ]; then
  echo "unknown"
  exit 0
fi

echo "$VALUE"
