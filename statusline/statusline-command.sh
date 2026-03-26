#!/usr/bin/env bash
set -uo pipefail

# ECC Statusline — receives JSON from Claude Code via stdin.
# Outputs: Model [########--------] 42% | cost | tokens | lines | RL | branch | ecc vX.Y.Z

ECC_VERSION="__ECC_VERSION__"
MIN_WIDTH=40
BAR_WIDTH=8

# --- jq check ---
command -v jq >/dev/null 2>&1 || { echo "ECC"; exit 0; }

# --- Read stdin ---
INPUT=$(cat)
if [ -z "$INPUT" ] || ! echo "$INPUT" | jq empty 2>/dev/null; then
  echo "ECC"
  exit 0
fi

# --- JSON extraction ---
DISPLAY_NAME=$(echo "$INPUT" | jq -r '.model.display_name // ""')
USED_PCT=$(echo "$INPUT"     | jq -r '.context_window.used_percentage // 0')
TOTAL_INPUT=$(echo "$INPUT"  | jq -r '.context_window.total_input_tokens // 0')
TOTAL_OUTPUT=$(echo "$INPUT" | jq -r '.context_window.total_output_tokens // 0')
COST_USD=$(echo "$INPUT"     | jq -r '.cost.total_cost_usd // 0')
DURATION_MS=$(echo "$INPUT"  | jq -r '.cost.total_duration_ms // 0')
LINES_ADDED=$(echo "$INPUT"  | jq -r '.cost.total_lines_added // 0')
LINES_REMOVED=$(echo "$INPUT"| jq -r '.cost.total_lines_removed // 0')
RATE_LIMIT=$(echo "$INPUT"   | jq -r '.rate_limits.five_hour.used_percentage // ""')

# --- Context bar with ANSI colors ---
USED_INT=${USED_PCT%.*}
FILLED=$(( (USED_INT * BAR_WIDTH + 50) / 100 ))
[ "$FILLED" -gt "$BAR_WIDTH" ] && FILLED=$BAR_WIDTH
EMPTY=$(( BAR_WIDTH - FILLED ))
BAR_INNER=$(printf '%0.s#' $(seq 1 "$FILLED" 2>/dev/null); printf '%0.s-' $(seq 1 "$EMPTY" 2>/dev/null))

if [ "$USED_INT" -ge 80 ] 2>/dev/null; then
  COLOR='\033[31m'
elif [ "$USED_INT" -ge 60 ] 2>/dev/null; then
  COLOR='\033[33m'
else
  COLOR='\033[32m'
fi
CTX_BAR="${COLOR}[${BAR_INNER}]\033[0m ${USED_INT}%"

# --- Cost formatting ---
COST_FMT=$(printf '$%.2f' "$COST_USD" 2>/dev/null || echo '$0.00')

# --- Duration formatting ---
DUR_S=$(( DURATION_MS / 1000 ))
DUR_M=$(( DUR_S / 60 ))
DUR_REMAIN=$(( DUR_S % 60 ))
DURATION_FMT="${DUR_M}m ${DUR_REMAIN}s"

# --- Token counts ---
IN_K=$(printf '%.1f' "$(echo "$TOTAL_INPUT"  | awk '{printf "%.1f", $1/1000}')" 2>/dev/null || echo "0.0")
OUT_K=$(printf '%.1f' "$(echo "$TOTAL_OUTPUT" | awk '{printf "%.1f", $1/1000}')" 2>/dev/null || echo "0.0")
TOKENS_FMT="In:${IN_K}k Out:${OUT_K}k"

# --- Lines diff ---
LINES_FMT="+${LINES_ADDED}/-${LINES_REMOVED}"

# --- Rate limit ---
RL_FMT=""
[ -n "$RATE_LIMIT" ] && RL_FMT="RL:${RATE_LIMIT%.*}%"

# --- Git branch with caching ---
CACHE_DIR="/tmp"
PWD_HASH=$(echo "$PWD" | md5sum 2>/dev/null | cut -c1-8 || md5 -q -s "$PWD" 2>/dev/null | cut -c1-8 || echo "nohash")
CACHE_FILE="${CACHE_DIR}/ecc-sl-cache-${PWD_HASH}"

BRANCH=""
# TTL: cache age check — cache is valid if file mtime is within 5s
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

# --- Terminal width ---
TERM_WIDTH=${COLUMNS:-$(tput cols 2>/dev/null || echo 120)}

# --- Build output with priority-based truncation ---
# Model is NEVER dropped. Priority (lowest = first dropped):
# 8=ECC_VERSION 7=RL 6=DURATION 5=TOKENS 4=LINES 3=COST 2=BRANCH 1=CTX_BAR 0=MODEL(never)

SEG_MODEL="\033[1m${DISPLAY_NAME}\033[0m"
SEG_CTX="${CTX_BAR}"
SEG_BRANCH="${BRANCH}"
SEG_COST="${COST_FMT}"
SEG_LINES="${LINES_FMT}"
SEG_TOKENS="${TOKENS_FMT}"
SEG_DURATION="${DURATION_FMT}"
SEG_RL="${RL_FMT}"
SEG_ECC="ecc ${ECC_VERSION}"

# Build progressively, checking width (strip ANSI for length calc)
strip_ansi() { printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g'; }

if [ "$TERM_WIDTH" -lt "$MIN_WIDTH" ] 2>/dev/null; then
  printf '%b' "${SEG_MODEL}"
  exit 0
fi

# Assemble full line with all segments
build_line() {
  local parts=()
  [ -n "${1:-}" ] && parts+=("$1")
  [ -n "${2:-}" ] && parts+=("$2")
  [ -n "${3:-}" ] && parts+=("$3")
  [ -n "${4:-}" ] && parts+=("$4")
  [ -n "${5:-}" ] && parts+=("$5")
  [ -n "${6:-}" ] && parts+=("$6")
  [ -n "${7:-}" ] && parts+=("$7")
  [ -n "${8:-}" ] && parts+=("$8")
  local IFS=' | '
  printf '%b' "${parts[*]}"
}

# Try adding segments one at a time (drop from end if too wide)
CANDIDATES=(
  "$SEG_MODEL"
  "$SEG_CTX"
  "$SEG_BRANCH"
  "$SEG_COST"
  "$SEG_LINES"
  "$SEG_TOKENS"
  "$SEG_DURATION"
  "$SEG_RL"
  "$SEG_ECC"
)

ACTIVE=("${CANDIDATES[0]}")
for seg in "${CANDIDATES[@]:1}"; do
  [ -z "$seg" ] && continue
  CANDIDATE_LINE=$(build_line "${ACTIVE[@]}" "$seg")
  VISIBLE_LEN=${#$(strip_ansi "$CANDIDATE_LINE")}
  if [ "$VISIBLE_LEN" -le "$TERM_WIDTH" ] 2>/dev/null; then
    ACTIVE+=("$seg")
  fi
done

OUTPUT=$(build_line "${ACTIVE[@]}")
printf '%b' "$OUTPUT"
