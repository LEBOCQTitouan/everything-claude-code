#!/usr/bin/env bash
set -euo pipefail

HOOK_ID="${1:-}"
REL_SCRIPT_PATH="${2:-}"
PROFILES_CSV="${3:-standard,strict}"

# Resolve ECC root: env var → CLAUDE_PLUGIN_ROOT fallback → self-resolve from script location
if [[ -z "${ECC_ROOT:-}" ]]; then
  ECC_ROOT="${CLAUDE_PLUGIN_ROOT:-}"
fi
if [[ -z "$ECC_ROOT" ]]; then
  # Self-resolve: this script lives at scripts/hooks/run-with-flags-shell.sh
  _SELF="${BASH_SOURCE[0]}"
  while [ -L "$_SELF" ]; do
    _DIR="$(cd "$(dirname "$_SELF")" && pwd)"
    _SELF="$(readlink "$_SELF")"
    [[ "$_SELF" != /* ]] && _SELF="$_DIR/$_SELF"
  done
  ECC_ROOT="$(cd "$(dirname "$_SELF")/../.." && pwd)"
fi

# Preserve stdin for passthrough or script execution
INPUT="$(cat)"

if [[ -z "$HOOK_ID" || -z "$REL_SCRIPT_PATH" ]]; then
  printf '%s' "$INPUT"
  exit 0
fi

# Ask Rust CLI if this hook is enabled (fallback to "yes" if ecc not available)
if command -v ecc >/dev/null 2>&1; then
    ENABLED="$(echo "$HOOK_ID" | ecc hook check:hook:enabled 2>/dev/null | tr -d '[:space:]')"
    [ -z "$ENABLED" ] && ENABLED="yes"
else
    ENABLED="yes"
fi
if [[ "$ENABLED" != "yes" ]]; then
  printf '%s' "$INPUT"
  exit 0
fi

SCRIPT_PATH="${ECC_ROOT}/${REL_SCRIPT_PATH}"
if [[ ! -f "$SCRIPT_PATH" ]]; then
  echo "[Hook] Script not found for ${HOOK_ID}: ${SCRIPT_PATH}" >&2
  printf '%s' "$INPUT"
  exit 0
fi

printf '%s' "$INPUT" | "$SCRIPT_PATH"
