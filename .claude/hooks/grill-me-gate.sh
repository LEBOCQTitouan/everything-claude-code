#!/usr/bin/env bash
set -uo pipefail
[ "${ECC_WORKFLOW_BYPASS:-}" = "1" ] && exit 0

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
STATE_FILE="$PROJECT_DIR/.claude/workflow/state.json"

# No workflow active
[ ! -f "$STATE_FILE" ] && exit 0

PHASE=$(jq -r '.phase // ""' "$STATE_FILE" 2>/dev/null) || exit 0

# Only check during plan or solution phases
if [ "$PHASE" != "plan" ] && [ "$PHASE" != "solution" ]; then
  exit 0
fi

SPEC_PATH=$(jq -r '.artifacts.spec_path // ""' "$STATE_FILE" 2>/dev/null) || exit 0
CAMPAIGN_PATH=$(jq -r '.artifacts.campaign_path // ""' "$STATE_FILE" 2>/dev/null) || exit 0

found_decision=0

# Check spec file for grill-me decision markers
if [ -n "$SPEC_PATH" ] && [ -f "$PROJECT_DIR/$SPEC_PATH" ]; then
  if grep -qi 'Grill-Me Decisions\|### Grill-Me' "$PROJECT_DIR/$SPEC_PATH" 2>/dev/null; then
    found_decision=1
  fi
fi

# Check campaign.md for grill-me decision markers
if [ "$found_decision" -eq 0 ] && [ -n "$CAMPAIGN_PATH" ] && [ -f "$PROJECT_DIR/$CAMPAIGN_PATH" ]; then
  if grep -qi 'Grill-Me Decisions\|### Grill-Me' "$PROJECT_DIR/$CAMPAIGN_PATH" 2>/dev/null; then
    found_decision=1
  fi
fi

if [ "$found_decision" -eq 0 ]; then
  echo "WARNING: Grill-me interview not completed or not found in spec output." >&2
fi

exit 0
