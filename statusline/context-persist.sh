#!/usr/bin/env bash
set -euo pipefail

# ECC Context Persist — side-channel writer for context window percentage.
# Reads JSON from stdin, extracts used_percentage, writes to runtime file.
# Best-effort: exits silently on any error.

trap 'exit 0' ERR

# --- Read JSON from stdin ---
INPUT=$(cat)

# --- Extract percentage via jq ---
USED_PCT=$(echo "$INPUT" | jq -r '.context_window.used_percentage // empty')
if [ -z "${USED_PCT:-}" ]; then
  exit 0
fi
# Truncate decimal to integer (API may send 85.7, reader expects integer)
USED_PCT="${USED_PCT%.*}"

# --- Sanitize session ID ---
RAW_SESSION="${CLAUDE_SESSION_ID:-$PPID}"
SESSION_ID=$(printf '%s' "$RAW_SESSION" | tr -dc 'a-zA-Z0-9_-')
if [ -z "$SESSION_ID" ]; then
  exit 0
fi

# --- Resolve runtime directory ---
FALLBACK_BASE="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}"
RUNTIME_DIR="${ECC_RUNTIME_DIR:-${FALLBACK_BASE}/ecc-$(id -u)}"
mkdir -p "$RUNTIME_DIR"
chmod 700 "$RUNTIME_DIR"

# --- Atomic write via mktemp + mv ---
TARGET="${RUNTIME_DIR}/ecc-context-${SESSION_ID}.pct"
TMPFILE=$(mktemp "${RUNTIME_DIR}/ecc-context-XXXXXX.tmp")
printf '%s' "$USED_PCT" > "$TMPFILE"
mv "$TMPFILE" "$TARGET"
