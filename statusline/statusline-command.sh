#!/usr/bin/env bash
set -euo pipefail

# ECC Statusline — receives JSON from Claude Code via stdin.
# Outputs: Model [########--------] 42% | branch | project | ecc vX.Y.Z

ECC_VERSION="__ECC_VERSION__"
BAR_WIDTH=8

# Read JSON from stdin
INPUT=$(cat)

# --- Model name ---
MODEL=$(echo "$INPUT" | jq -r '.model.display_name // "unknown"')

# --- Context usage ---
USED_PCT=$(echo "$INPUT" | jq -r '.context_window.used_percentage // 0')
# Integer truncation for bar math
USED_INT=${USED_PCT%.*}
FILLED=$(( (USED_INT * BAR_WIDTH + 50) / 100 ))
if [ "$FILLED" -gt "$BAR_WIDTH" ]; then
  FILLED=$BAR_WIDTH
fi
EMPTY=$(( BAR_WIDTH - FILLED ))
BAR=$(printf '%0.s#' $(seq 1 "$FILLED" 2>/dev/null) ; printf '%0.s-' $(seq 1 "$EMPTY" 2>/dev/null))
PCT_DISPLAY="${USED_INT}%"

# --- Git branch ---
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "n/a")

# --- Project name ---
PROJECT=""
if [ -f "Cargo.toml" ]; then
  PROJECT=$(grep -m1 '^name' Cargo.toml | sed 's/^name[[:space:]]*=[[:space:]]*"\(.*\)"/\1/' 2>/dev/null || true)
fi
if [ -z "$PROJECT" ] && [ -f "package.json" ]; then
  PROJECT=$(jq -r '.name // empty' package.json 2>/dev/null || true)
fi
if [ -z "$PROJECT" ]; then
  PROJECT=$(basename "$PWD")
fi

# --- Output ---
printf '\033[1m%s\033[0m [%s] %s | %s | %s | ecc %s' \
  "$MODEL" "$BAR" "$PCT_DISPLAY" "$BRANCH" "$PROJECT" "$ECC_VERSION"
