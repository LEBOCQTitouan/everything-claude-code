#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "plan"' "$STATE_FILE" 2>/dev/null) || exit 0

# Only active at done phase
[ "$PHASE" != "done" ] && exit 0

IMPL_FILE="$PROJECT_DIR/.claude/workflow/implement-done.md"
[ ! -f "$IMPL_FILE" ] && exit 0

WARNINGS=""

# Check CLAUDE.md line count (< 200 lines)
CLAUDE_MD="$PROJECT_DIR/CLAUDE.md"
if [ -f "$CLAUDE_MD" ]; then
  LINE_COUNT=$(wc -l < "$CLAUDE_MD" | tr -d ' ')
  if [ "$LINE_COUNT" -gt 200 ]; then
    WARNINGS="${WARNINGS}WARNING: CLAUDE.md is ${LINE_COUNT} lines (limit: 200). Consider moving details to lower-level docs.\n"
  fi
fi

# Check README.md line count (< 300 lines)
README_MD="$PROJECT_DIR/README.md"
if [ -f "$README_MD" ]; then
  LINE_COUNT=$(wc -l < "$README_MD" | tr -d ' ')
  if [ "$LINE_COUNT" -gt 300 ]; then
    WARNINGS="${WARNINGS}WARNING: README.md is ${LINE_COUNT} lines (limit: 300). Consider extracting sections to docs/.\n"
  fi
fi

# Check ARCHITECTURE.md for oversized code blocks (> 20 lines)
ARCH_MD="$PROJECT_DIR/docs/ARCHITECTURE.md"
if [ -f "$ARCH_MD" ]; then
  IN_BLOCK=0
  BLOCK_LINES=0
  BLOCK_START=0
  LINE_NUM=0
  while IFS= read -r line; do
    LINE_NUM=$((LINE_NUM + 1))
    if echo "$line" | grep -q '^```'; then
      if [ "$IN_BLOCK" -eq 0 ]; then
        IN_BLOCK=1
        BLOCK_LINES=0
        BLOCK_START=$LINE_NUM
      else
        IN_BLOCK=0
        if [ "$BLOCK_LINES" -gt 20 ]; then
          WARNINGS="${WARNINGS}WARNING: ARCHITECTURE.md has a code block starting at line ${BLOCK_START} with ${BLOCK_LINES} lines (limit: 20). Move to source or examples.\n"
        fi
      fi
    elif [ "$IN_BLOCK" -eq 1 ]; then
      BLOCK_LINES=$((BLOCK_LINES + 1))
    fi
  done < "$ARCH_MD"
fi

if [ -n "$WARNINGS" ]; then
  echo -e "$WARNINGS" >&2
fi

# Always exit 0 — warning only
exit 0
