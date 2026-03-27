#!/usr/bin/env bash
set -uo pipefail

# ECC Statusline — receives JSON from Claude Code via stdin.
# Output: ◆ Model │ ctx: [████░░░░] 42% │ 5h: [██░░] 23% 7d: [█░░░] 12% │ ↑15k ↓4k │ +50 -10 │ 2m 0s │ ⎇ main │ ecc 4.2.0

ECC_VERSION="__ECC_VERSION__"
MIN_WIDTH=40
BAR_WIDTH=8

# --- ANSI color codes ---
RST='\033[0m'
DIM='\033[2m'
BOLD='\033[1m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
GREEN_ADD='\033[32m'
RED_DEL='\033[31m'

# --- jq check ---
command -v jq >/dev/null 2>&1 || { echo "ECC"; exit 0; }

# --- Read stdin ---
INPUT=$(cat)
if [ -z "$INPUT" ] || ! echo "$INPUT" | jq empty 2>/dev/null; then
  echo "ECC"
  exit 0
fi

# --- Single jq extraction (1 fork instead of 10+) ---
eval "$(echo "$INPUT" | jq -r '
  @sh "DISPLAY_NAME=\(.model.display_name // "")",
  @sh "USED_PCT=\(.context_window.used_percentage // 0)",
  @sh "TOTAL_INPUT=\(.context_window.total_input_tokens // 0)",
  @sh "TOTAL_OUTPUT=\(.context_window.total_output_tokens // 0)",
  @sh "COST_USD=\(.cost.total_cost_usd // 0)",
  @sh "DURATION_MS=\(.cost.total_duration_ms // 0)",
  @sh "LINES_ADDED=\(.cost.total_lines_added // 0)",
  @sh "LINES_REMOVED=\(.cost.total_lines_removed // 0)",
  @sh "RL_5H=\(.rate_limits.five_hour.used_percentage // "")",
  @sh "RL_7D=\(.rate_limits.seven_day.used_percentage // "")",
  @sh "HAS_RATE_LIMITS=\(if .rate_limits then "1" else "" end)"
' 2>/dev/null)" || {
  echo "ECC"
  exit 0
}

# --- Progress bar helper ---
# Usage: make_bar <pct_int> <width>
make_bar() {
  local pct="${1:-0}" width="${2:-$BAR_WIDTH}"
  local filled=$(( (pct * width + 50) / 100 ))
  [ "$filled" -gt "$width" ] && filled=$width
  local empty=$(( width - filled ))
  local bar=""
  local i
  for (( i=0; i<filled; i++ )); do bar+="█"; done
  for (( i=0; i<empty; i++ )); do bar+="░"; done
  printf '%s' "$bar"
}

# --- Color for threshold ---
# Usage: threshold_color <pct_int> <yellow_at> <red_at>
threshold_color() {
  local pct="${1:-0}" yellow="${2:-60}" red="${3:-80}"
  if [ "$pct" -ge "$red" ] 2>/dev/null; then
    printf '%s' "$RED"
  elif [ "$pct" -ge "$yellow" ] 2>/dev/null; then
    printf '%s' "$YELLOW"
  else
    printf '%s' "$GREEN"
  fi
}

# --- Context bar ---
USED_INT=${USED_PCT%.*}
CTX_COLOR=$(threshold_color "$USED_INT" 60 80)
CTX_BAR_INNER=$(make_bar "$USED_INT" "$BAR_WIDTH")
SEG_CTX="${DIM}ctx:${RST} ${CTX_COLOR}[${CTX_BAR_INNER}]${RST} ${USED_INT}%"
SEG_CTX_NARROW="${DIM}ctx:${RST} ${CTX_COLOR}${USED_INT}%${RST}"

# --- Rate limit bars (only for subscribers) ---
SEG_RL=""
SEG_RL_NARROW=""
if [ -n "$HAS_RATE_LIMITS" ]; then
  rl_parts=""
  rl_narrow_parts=""
  if [ -n "$RL_5H" ]; then
    RL_5H_INT=${RL_5H%.*}
    RL_5H_COLOR=$(threshold_color "$RL_5H_INT" 60 80)
    RL_5H_BAR=$(make_bar "$RL_5H_INT" "$BAR_WIDTH")
    rl_parts="${DIM}5h:${RST} ${RL_5H_COLOR}[${RL_5H_BAR}]${RST} ${RL_5H_INT}%"
    rl_narrow_parts="${DIM}5h:${RST} ${RL_5H_COLOR}${RL_5H_INT}%${RST}"
  fi
  if [ -n "$RL_7D" ]; then
    RL_7D_INT=${RL_7D%.*}
    RL_7D_COLOR=$(threshold_color "$RL_7D_INT" 60 80)
    RL_7D_BAR=$(make_bar "$RL_7D_INT" "$BAR_WIDTH")
    if [ -n "$rl_parts" ]; then
      rl_parts+=" ${DIM}7d:${RST} ${RL_7D_COLOR}[${RL_7D_BAR}]${RST} ${RL_7D_INT}%"
      rl_narrow_parts+=" ${DIM}7d:${RST} ${RL_7D_COLOR}${RL_7D_INT}%${RST}"
    else
      rl_parts="${DIM}7d:${RST} ${RL_7D_COLOR}[${RL_7D_BAR}]${RST} ${RL_7D_INT}%"
      rl_narrow_parts="${DIM}7d:${RST} ${RL_7D_COLOR}${RL_7D_INT}%${RST}"
    fi
  fi
  SEG_RL="$rl_parts"
  SEG_RL_NARROW="$rl_narrow_parts"
fi

# --- Cost (hidden for subscribers, shown for API billing) ---
SEG_COST=""
if [ -z "$HAS_RATE_LIMITS" ]; then
  COST_FMT=$(printf '$%.2f' "$COST_USD" 2>/dev/null || echo '$0.00')
  SEG_COST="${DIM}cost:${RST} ${COST_FMT}"
fi

# --- Token counts ---
IN_K=$(awk "BEGIN{printf \"%.1f\", ${TOTAL_INPUT}/1000}" 2>/dev/null || echo "0.0")
OUT_K=$(awk "BEGIN{printf \"%.1f\", ${TOTAL_OUTPUT}/1000}" 2>/dev/null || echo "0.0")
SEG_TOKENS="${GREEN}↑${RST}${IN_K}k ${RED}↓${RST}${OUT_K}k"

# --- Lines diff ---
SEG_LINES="${GREEN_ADD}+${LINES_ADDED}${RST} ${RED_DEL}-${LINES_REMOVED}${RST}"

# --- Duration ---
DUR_S=$(( DURATION_MS / 1000 ))
DUR_M=$(( DUR_S / 60 ))
DUR_REMAIN=$(( DUR_S % 60 ))
SEG_DURATION="${DUR_M}m ${DUR_REMAIN}s"

# --- Git branch with caching ---
CACHE_DIR="/tmp"
PWD_HASH=$(echo "$PWD" | md5sum 2>/dev/null | cut -c1-8 || md5 -q -s "$PWD" 2>/dev/null | cut -c1-8 || echo "nohash")
CACHE_FILE="${CACHE_DIR}/ecc-sl-cache-${PWD_HASH}"

BRANCH=""
TTL_REF=$(mktemp "${CACHE_DIR}/ecc-sl-ttl-XXXXXX")
touch -d '-5 seconds' "$TTL_REF" 2>/dev/null || touch -A -000005 "$TTL_REF" 2>/dev/null || true
if [ -f "$CACHE_FILE" ] && find "$CACHE_FILE" -newer "$TTL_REF" 2>/dev/null | grep -q .; then
  BRANCH=$(cat "$CACHE_FILE" 2>/dev/null || true)
fi
rm -f "$TTL_REF" 2>/dev/null || true

if [ -z "$BRANCH" ]; then
  BRANCH=$(git --no-optional-locks rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
  if [ -n "$BRANCH" ]; then
    TMPFILE=$(mktemp "${CACHE_DIR}/ecc-sl-XXXXXX")
    printf '%s' "$BRANCH" > "$TMPFILE"
    mv "$TMPFILE" "$CACHE_FILE"
  fi
fi

SEG_BRANCH=""
[ -n "$BRANCH" ] && SEG_BRANCH="${DIM}⎇${RST} ${BRANCH}"

# --- Model + ECC version ---
SEG_MODEL="${BOLD}◆${RST} ${BOLD}${DISPLAY_NAME}${RST}"
SEG_ECC="${DIM}ecc ${ECC_VERSION}${RST}"

# --- Terminal width ---
TERM_WIDTH=${COLUMNS:-$(tput cols 2>/dev/null || echo 120)}

# --- Strip ANSI for length calculation ---
strip_ansi() { printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g'; }

# --- Build output with priority-based truncation ---
# Separator: │ (Unicode box-drawing U+2502)
SEP=" │ "

if [ "$TERM_WIDTH" -lt "$MIN_WIDTH" ] 2>/dev/null; then
  printf '%b' "${SEG_MODEL}"
  exit 0
fi

# Segments in priority order (highest = kept longest, lowest = first dropped)
# Model and context bar are always included if they fit
build_output() {
  # Priority order: model > context > rate limits > branch > tokens > lines > duration > cost > version
  # Rate limits are high priority since they show quota pressure
  local segments=()
  segments+=("$SEG_MODEL")
  segments+=("$SEG_CTX")
  [ -n "$SEG_RL" ]        && segments+=("$SEG_RL")
  [ -n "$SEG_BRANCH" ]   && segments+=("$SEG_BRANCH")
  segments+=("$SEG_TOKENS")
  segments+=("$SEG_LINES")
  segments+=("$SEG_DURATION")
  [ -n "$SEG_COST" ]      && segments+=("$SEG_COST")
  segments+=("$SEG_ECC")

  # Build progressively, checking width
  local active=("${segments[0]}")
  local seg candidate stripped
  for seg in "${segments[@]:1}"; do
    [ -z "$seg" ] && continue
    candidate=$(IFS=; printf '%b' "$(join_segments "${active[@]}" "$seg")")
    stripped=$(strip_ansi "$candidate")
    if [ "${#stripped}" -le "$TERM_WIDTH" ] 2>/dev/null; then
      active+=("$seg")
    fi
  done

  IFS=; printf '%b' "$(join_segments "${active[@]}")"
}

join_segments() {
  local result=""
  local first=1
  local s
  for s in "$@"; do
    if [ "$first" -eq 1 ]; then
      result="$s"
      first=0
    else
      result+="${SEP}${s}"
    fi
  done
  printf '%s' "$result"
}

# Try full-width first. If RL bars don't fit, try narrow RL (text-only).
# Then try narrow context bar. Each retry rebuilds from scratch.
OUTPUT=$(build_output)
STRIPPED=$(strip_ansi "$OUTPUT")

# Check if RL was included in output; if not and we have RL, try narrow version
if [ -n "$SEG_RL" ] && ! echo "$STRIPPED" | grep -q '5h:\|7d:'; then
  SEG_RL="$SEG_RL_NARROW"
  OUTPUT=$(build_output)
  STRIPPED=$(strip_ansi "$OUTPUT")
fi

if [ "${#STRIPPED}" -gt "$TERM_WIDTH" ] 2>/dev/null && [ -n "$SEG_RL_NARROW" ]; then
  SEG_RL="$SEG_RL_NARROW"
  OUTPUT=$(build_output)
  STRIPPED=$(strip_ansi "$OUTPUT")
fi
if [ "${#STRIPPED}" -gt "$TERM_WIDTH" ] 2>/dev/null; then
  SEG_CTX="$SEG_CTX_NARROW"
  OUTPUT=$(build_output)
fi

printf '%b' "$OUTPUT"
