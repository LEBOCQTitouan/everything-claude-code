#!/usr/bin/env bash
set -euo pipefail

# ECC Statusline — receives JSON from Claude Code via stdin.
# Outputs: Model [########--------] 42% | repo | branch | dir | ecc vX.Y.Z

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

# --- Git repo ---
REPO="n/a"
REMOTE_URL=$(git remote get-url origin 2>/dev/null || true)
if [ -n "$REMOTE_URL" ]; then
  # Strip .git suffix, then extract last two path segments (org/repo)
  CLEAN_URL="${REMOTE_URL%.git}"
  REPO=$(echo "$CLEAN_URL" | sed 's|.*[:/]\([^/]*/[^/]*\)$|\1|')
fi

# --- Git branch ---
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "n/a")

# --- Current directory (last 2 path segments) ---
REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
if [ -n "$REPO_ROOT" ] && [ "$PWD" != "$REPO_ROOT" ]; then
  REL_PATH="${PWD#"$REPO_ROOT"/}"
  # Show last 2 segments of relative path
  DIR=$(echo "$REL_PATH" | awk -F/ '{if(NF<=2) print $0; else print $(NF-1)"/"$NF}')
else
  DIR=$(basename "$PWD")
fi

# --- Output ---
printf '\033[1m%s\033[0m [%s] %s | %s | %s | %s | ecc %s' \
  "$MODEL" "$BAR" "$PCT_DISPLAY" "$REPO" "$BRANCH" "$DIR" "$ECC_VERSION"
