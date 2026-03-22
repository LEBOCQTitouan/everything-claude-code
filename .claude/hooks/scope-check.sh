#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // "plan"' "$STATE_FILE" 2>/dev/null) || exit 0

# Only active during implement or done phases
[ "$PHASE" != "implement" ] && [ "$PHASE" != "done" ] && exit 0

# Read design path from state.json artifacts.design_path (BL-035)
DESIGN_PATH=$(jq -r '.artifacts.design_path // empty' "$STATE_FILE" 2>/dev/null) || true
SOLUTION_FILE=""
if [ -n "$DESIGN_PATH" ]; then
  RESOLVED=$(cd "$PROJECT_DIR" && realpath -m "$DESIGN_PATH" 2>/dev/null) || true
  case "${RESOLVED:-}" in
    "$PROJECT_DIR"/*) SOLUTION_FILE="$RESOLVED" ;;
    *) echo "WARNING: design_path escapes project directory, ignoring." >&2 ;;
  esac
fi

# Fallback to legacy path
if [ -z "$SOLUTION_FILE" ] || [ ! -f "$SOLUTION_FILE" ]; then
  SOLUTION_FILE="$PROJECT_DIR/.claude/workflow/solution.md"
fi

# Need design/solution file
[ ! -f "$SOLUTION_FILE" ] && exit 0

# Extract expected file paths from solution.md File Changes table
# Matches table rows: | N | path/to/file | action | ...
EXPECTED_FILES=$(grep -oE '\| [0-9]+ \| [^ |]+' "$SOLUTION_FILE" | sed 's/| [0-9]* | //' | sort -u)

# Get actual changed files
ACTUAL_FILES=$(cd "$PROJECT_DIR" && git diff --name-only HEAD 2>/dev/null | sort -u)

if [ -z "$ACTUAL_FILES" ]; then
  # Try against the workflow start commit if no uncommitted changes
  STARTED_AT=$(jq -r '.started_at // empty' "$STATE_FILE" 2>/dev/null) || true
  if [ -n "$STARTED_AT" ]; then
    # Use git log to find commits since workflow start
    ACTUAL_FILES=$(cd "$PROJECT_DIR" && git diff --name-only "$(git rev-list --before="$STARTED_AT" -1 HEAD 2>/dev/null || echo HEAD)" HEAD 2>/dev/null | sort -u)
  fi
fi

[ -z "$ACTUAL_FILES" ] && exit 0

# Filter out exceptions
UNEXPECTED=""
while IFS= read -r file; do
  [ -z "$file" ] && continue
  case "$file" in
    docs/*) continue ;;
    .claude/workflow/*) continue ;;
    CHANGELOG.md) continue ;;
    *test*|*_test*|*Test*|*spec*) continue ;;
    Cargo.lock|package-lock.json|go.sum|yarn.lock|pnpm-lock.yaml) continue ;;
  esac
  # Check if file is in expected list
  if ! echo "$EXPECTED_FILES" | grep -qF "$file"; then
    UNEXPECTED="${UNEXPECTED}${file}\n"
  fi
done <<< "$ACTUAL_FILES"

if [ -n "$UNEXPECTED" ]; then
  echo "WARNING: Unexpected files changed (not in solution.md File Changes table):" >&2
  echo -e "$UNEXPECTED" | while IFS= read -r f; do
    [ -n "$f" ] && echo "  - $f" >&2
  done
  echo "This may indicate scope creep. Review these changes." >&2
fi

# Always exit 0 — warning only
exit 0
