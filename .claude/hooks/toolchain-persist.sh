#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

# Usage: bash .claude/hooks/toolchain-persist.sh "<test_cmd>" "<lint_cmd>" "<build_cmd>"
# Writes detected toolchain commands to state.json. Requires jq.

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

TEST_CMD="${1:-}"
LINT_CMD="${2:-}"
BUILD_CMD="${3:-}"

if [ ! -f "$STATE_FILE" ]; then
  echo "WARNING: state.json not found at $STATE_FILE — toolchain not persisted." >&2
  exit 0
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "WARNING: jq unavailable: toolchain not persisted." >&2
  exit 0
fi

TMPFILE=$(mktemp "${PROJECT_DIR}/.claude/workflow/state.XXXXXX") || exit 1

jq --arg test "$TEST_CMD" \
   --arg lint "$LINT_CMD" \
   --arg build "$BUILD_CMD" \
   '.toolchain.test = $test | .toolchain.lint = $lint | .toolchain.build = $build' \
   "$STATE_FILE" > "$TMPFILE"

mv "$TMPFILE" "$STATE_FILE"
echo "Toolchain persisted: test=$TEST_CMD, lint=$LINT_CMD, build=$BUILD_CMD"
