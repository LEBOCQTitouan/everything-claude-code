#!/usr/bin/env bash
set -uo pipefail

# Memory writer for cross-session persistence (BL-027)
# Usage:
#   memory-writer.sh action <action_type> <description> <outcome> <artifacts_json>
#   memory-writer.sh work-item <phase> <description> <concern>
#
# Action types: plan, solution, implement, verify, fix, audit, review, other
# Phases: plan, solution, implementation

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-.}"
MEMORY_DIR="$PROJECT_DIR/docs/memory"
ACTION_LOG="$MEMORY_DIR/action-log.json"
WORK_ITEMS_DIR="$MEMORY_DIR/work-items"

# Ensure memory directory exists
ensure_dirs() {
  mkdir -p "$MEMORY_DIR" "$WORK_ITEMS_DIR"
  if [ ! -f "$ACTION_LOG" ]; then
    echo "[]" > "$ACTION_LOG"
  fi
}

# Generate slug from description: lowercase, hyphenated, max 40 chars, alphanumeric only
make_slug() {
  local desc="$1"
  echo "$desc" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9 ]//g' \
    | sed 's/  */ /g' \
    | sed 's/ /-/g' \
    | cut -c1-40 \
    | sed 's/-$//'
}

# Write an action log entry (append-only, atomic)
write_action() {
  local action_type="$1"
  local description="$2"
  local outcome="$3"
  local artifacts_json="$4"
  local timestamp
  local session_id

  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  session_id="${CLAUDE_SESSION_ID:-unknown}"

  ensure_dirs

  local entry
  entry=$(jq -n \
    --arg ts "$timestamp" \
    --arg sid "$session_id" \
    --arg at "$action_type" \
    --arg desc "$description" \
    --arg out "$outcome" \
    --argjson arts "$artifacts_json" \
    '{
      timestamp: $ts,
      session_id: $sid,
      action_type: $at,
      description: $desc,
      artifacts: $arts,
      outcome: $out,
      tags: []
    }')

  # Atomic append via mktemp+mv
  local tmpfile
  tmpfile=$(mktemp "${MEMORY_DIR}/action-log.XXXXXX") || return 1
  jq ". += [$entry]" "$ACTION_LOG" > "$tmpfile" || { rm -f "$tmpfile"; return 1; }
  mv "$tmpfile" "$ACTION_LOG"
}

# Write a work item file (plan.md, solution.md, or implementation.md)
write_work_item() {
  local phase="$1"
  local description="$2"
  local concern="$3"
  local today
  local slug
  local item_dir
  local target_file

  today=$(date -u +"%Y-%m-%d")
  slug=$(make_slug "$description")
  item_dir="$WORK_ITEMS_DIR/${today}-${slug}"

  ensure_dirs
  mkdir -p "$item_dir"

  target_file="$item_dir/${phase}.md"

  local timestamp
  timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  # If file exists (re-entry), append a revision block instead of overwriting
  if [ -f "$target_file" ]; then
    cat >> "$target_file" <<EOF

## Revision

Date: $timestamp

Re-entry detected. This phase was re-executed.
EOF
    return 0
  fi

  # Write new file with deterministic H2 sections
  case "$phase" in
    plan)
      cat > "$target_file" <<EOF
# Plan: $description

## Context

Concern: $concern
Created: $timestamp

## Decisions

(Populated by /plan-* command output)

## User Stories

(Populated by /plan-* command output)

## Outcome

Phase completed at $timestamp
EOF
      ;;
    solution)
      cat > "$target_file" <<EOF
# Solution: $description

## Context

Concern: $concern
Created: $timestamp

## File Changes

(Populated by /solution command output)

## Pass Conditions

(Populated by /solution command output)

## Outcome

Phase completed at $timestamp
EOF
      ;;
    implementation)
      cat > "$target_file" <<EOF
# Implementation: $description

## Context

Concern: $concern
Created: $timestamp

## Changes Made

(Populated from implement-done.md)

## Test Results

(Populated from implement-done.md)

## Outcome

Phase completed at $timestamp
EOF
      ;;
    *)
      echo "ERROR: Unknown phase '$phase'. Use: plan, solution, implementation" >&2
      return 1
      ;;
  esac
}

# Resolve project memory dir for daily files (BL-047)
# Uses ~/.claude/projects/<project-hash>/memory/daily/
resolve_daily_dir() {
  local home="${HOME:-}"
  local proj_dir="${CLAUDE_PROJECT_DIR:-.}"
  if [ -z "$home" ]; then
    echo "ERROR: HOME not set" >&2
    return 1
  fi
  # Project hash: absolute path with / replaced by -
  local abs_path
  abs_path=$(cd "$proj_dir" 2>/dev/null && pwd) || abs_path="$proj_dir"
  # Remove leading / and replace remaining / with -
  local project_hash
  project_hash=$(echo "$abs_path" | sed 's|^/||' | sed 's|/|-|g')
  local daily_dir="$home/.claude/projects/$project_hash/memory/daily"
  echo "$daily_dir"
}

# Write a daily memory entry (BL-047)
write_daily() {
  local phase="$1"
  local feature="$2"
  local concern="$3"
  local daily_dir
  daily_dir=$(resolve_daily_dir) || return 0

  mkdir -p "$daily_dir" || return 0

  local today
  today=$(date -u +"%Y-%m-%d")
  local daily_file="$daily_dir/${today}.md"
  local now
  now=$(date -u +"%H:%M")

  # Init file if missing
  if [ ! -f "$daily_file" ]; then
    local tmpfile
    tmpfile=$(mktemp "${daily_dir}/daily.XXXXXX") || return 0

    {
      echo "# Daily: $today"
      echo ""

      # Link to recent previous sessions
      local recent_files
      recent_files=$(ls -r "$daily_dir"/*.md 2>/dev/null | head -n 3)
      if [ -n "$recent_files" ]; then
        echo "## Context from previous sessions"
        echo ""
        echo "$recent_files" | while IFS= read -r f; do
          local basename
          basename=$(basename "$f")
          echo "- [$basename]($basename)"
        done
        echo ""
      fi

      echo "## Activity"
      echo ""
      echo "## Insights"
      echo ""
    } > "$tmpfile"

    mv "$tmpfile" "$daily_file"
  fi

  # Ensure ## Activity and ## Insights sections exist
  if ! grep -q '## Activity' "$daily_file" 2>/dev/null; then
    echo "" >> "$daily_file"
    echo "## Activity" >> "$daily_file"
    echo "" >> "$daily_file"
  fi
  if ! grep -q '## Insights' "$daily_file" 2>/dev/null; then
    echo "" >> "$daily_file"
    echo "## Insights" >> "$daily_file"
    echo "" >> "$daily_file"
  fi

  # Append entry under ## Activity via atomic write
  local tmpfile
  tmpfile=$(mktemp "${daily_dir}/daily.XXXXXX") || return 0
  local entry="- [$now] **$phase** $feature — $concern"

  # Insert the entry after the ## Activity line
  awk -v entry="$entry" '
    /^## Activity/ { print; getline; print; print entry; next }
    { print }
  ' "$daily_file" > "$tmpfile" || { rm -f "$tmpfile"; return 0; }

  mv "$tmpfile" "$daily_file"
}

# Update MEMORY.md index with daily file link (BL-047)
write_memory_index() {
  local daily_dir
  daily_dir=$(resolve_daily_dir) || return 0
  local memory_dir
  memory_dir=$(dirname "$daily_dir")
  local memory_file="$memory_dir/MEMORY.md"

  # Create if missing
  if [ ! -f "$memory_file" ]; then
    mkdir -p "$memory_dir" || return 0
    echo "# Memory Index" > "$memory_file"
  fi

  # Ensure ## Daily section exists
  if ! grep -q '## Daily' "$memory_file" 2>/dev/null; then
    echo "" >> "$memory_file"
    echo "## Daily" >> "$memory_file"
    echo "" >> "$memory_file"
  fi

  local today
  today=$(date -u +"%Y-%m-%d")
  local link="- [$today](daily/$today.md)"

  # Skip if link already present (dedup)
  if grep -qF "$link" "$memory_file" 2>/dev/null; then
    return 0
  fi

  # Insert link in reverse chronological order (after ## Daily heading)
  local tmpfile
  tmpfile=$(mktemp "${memory_dir}/memory.XXXXXX") || return 0

  awk -v link="$link" '
    /^## Daily/ { print; getline; print; print link; next }
    { print }
  ' "$memory_file" > "$tmpfile" || { rm -f "$tmpfile"; return 0; }

  mv "$tmpfile" "$memory_file"
}

# Main dispatch
CMD="${1:-}"
shift || true

case "$CMD" in
  action)
    write_action "${1:-}" "${2:-}" "${3:-}" "${4:-[]}"
    ;;
  work-item)
    write_work_item "${1:-}" "${2:-}" "${3:-}"
    ;;
  daily)
    write_daily "${1:-}" "${2:-}" "${3:-}"
    ;;
  memory-index)
    write_memory_index
    ;;
  *)
    echo "Usage: memory-writer.sh action|work-item|daily|memory-index ..." >&2
    exit 1
    ;;
esac
