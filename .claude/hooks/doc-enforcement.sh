#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "plan"' "$STATE_FILE" 2>/dev/null) || exit 0

# Only enforce at done phase
[ "$PHASE" != "done" ] && exit 0

IMPL_FILE="$PROJECT_DIR/.claude/workflow/implement-done.md"

if [ ! -f "$IMPL_FILE" ]; then
  echo "BLOCKED: implement-done.md not found." >&2
  exit 2
fi

# Check for ## Docs Updated heading
if ! grep -q '^## Docs Updated' "$IMPL_FILE"; then
  echo "BLOCKED: implement-done.md missing '## Docs Updated' section." >&2
  echo "Add a '## Docs Updated' heading with at least one list item or table row." >&2
  exit 2
fi

# Check for at least one list item or table row after the heading
DOCS_SECTION=$(sed -n '/^## Docs Updated/,/^## /p' "$IMPL_FILE" | tail -n +2)

if ! echo "$DOCS_SECTION" | grep -qE '^\s*[-*]|^\|'; then
  echo "BLOCKED: '## Docs Updated' section has no list items or table rows." >&2
  echo "Add at least one entry documenting what was updated." >&2
  exit 2
fi

exit 0
