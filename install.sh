#!/usr/bin/env bash
# install.sh — Install everything into ~/.claude/
#
# Usage:
#   ./install.sh <language> [<language> ...]
#
# Examples:
#   ./install.sh typescript
#   ./install.sh typescript python golang
#
# What gets installed:
#   agents    -> ~/.claude/agents/
#   commands  -> ~/.claude/commands/
#   skills    -> ~/.claude/skills/
#   rules     -> ~/.claude/rules/common/ + ~/.claude/rules/<language>/
#   hooks     -> merged into ~/.claude/settings.json

set -euo pipefail

# ---------------------------------------------------------------------------
# Resolve symlinks so SCRIPT_DIR always points to the repo root
# ---------------------------------------------------------------------------
SCRIPT_PATH="$0"
while [ -L "$SCRIPT_PATH" ]; do
    link_dir="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"
    SCRIPT_PATH="$(readlink "$SCRIPT_PATH")"
    [[ "$SCRIPT_PATH" != /* ]] && SCRIPT_PATH="$link_dir/$SCRIPT_PATH"
done
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)"

AGENTS_DIR="$SCRIPT_DIR/03-agents"
COMMANDS_DIR="$SCRIPT_DIR/04-commands"
SKILLS_DIR="$SCRIPT_DIR/05-skills"
RULES_DIR="$SCRIPT_DIR/06-rules"
HOOKS_FILE="$SCRIPT_DIR/07-hooks/hooks.json"

CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"

# ---------------------------------------------------------------------------
# Usage
# ---------------------------------------------------------------------------
if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <language> [<language> ...]"
    echo ""
    echo "Available languages:"
    for dir in "$RULES_DIR"/*/; do
        name="$(basename "$dir")"
        [[ "$name" == "common" ]] && continue
        echo "  - $name"
    done
    exit 1
fi

# Validate language names (prevent path traversal)
for lang in "$@"; do
    if [[ ! "$lang" =~ ^[a-zA-Z0-9_-]+$ ]]; then
        echo "Error: invalid language name '$lang'. Only alphanumeric, dash, and underscore allowed." >&2
        exit 1
    fi
    if [[ ! -d "$RULES_DIR/$lang" ]]; then
        echo "Error: 06-rules/$lang/ does not exist." >&2
        echo "Available languages:"
        for dir in "$RULES_DIR"/*/; do
            name="$(basename "$dir")"
            [[ "$name" == "common" ]] && continue
            echo "  - $name"
        done
        exit 1
    fi
done

# ---------------------------------------------------------------------------
# Agents
# ---------------------------------------------------------------------------
echo "Installing agents -> $CLAUDE_DIR/agents/"
mkdir -p "$CLAUDE_DIR/agents"
cp "$AGENTS_DIR"/*.md "$CLAUDE_DIR/agents/"

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------
echo "Installing commands -> $CLAUDE_DIR/commands/"
mkdir -p "$CLAUDE_DIR/commands"
cp "$COMMANDS_DIR"/*.md "$CLAUDE_DIR/commands/"

# ---------------------------------------------------------------------------
# Skills (all of them)
# ---------------------------------------------------------------------------
echo "Installing skills -> $CLAUDE_DIR/skills/"
mkdir -p "$CLAUDE_DIR/skills"
cp -r "$SKILLS_DIR"/. "$CLAUDE_DIR/skills/"

# ---------------------------------------------------------------------------
# Rules — common + requested languages
# ---------------------------------------------------------------------------
RULES_DEST="$CLAUDE_DIR/rules"

if [[ -d "$RULES_DEST" ]] && [[ "$(ls -A "$RULES_DEST" 2>/dev/null)" ]]; then
    echo "Note: $RULES_DEST/ already exists. Existing files will be overwritten."
fi

echo "Installing common rules -> $RULES_DEST/common/"
mkdir -p "$RULES_DEST/common"
cp -r "$RULES_DIR/common/." "$RULES_DEST/common/"

for lang in "$@"; do
    echo "Installing $lang rules -> $RULES_DEST/$lang/"
    mkdir -p "$RULES_DEST/$lang"
    cp -r "$RULES_DIR/$lang/." "$RULES_DEST/$lang/"
done

# ---------------------------------------------------------------------------
# Hooks — merge into ~/.claude/settings.json
# ---------------------------------------------------------------------------
SETTINGS_FILE="$CLAUDE_DIR/settings.json"

echo "Merging hooks -> $SETTINGS_FILE"

if ! command -v node &>/dev/null; then
    echo "Warning: node not found — skipping hooks merge. Add hooks manually from 07-hooks/hooks.json." >&2
else
    node - "$SETTINGS_FILE" "$HOOKS_FILE" <<'NODE'
const fs = require('fs');
const [, , settingsPath, hooksPath] = process.argv;

const existing = fs.existsSync(settingsPath)
    ? JSON.parse(fs.readFileSync(settingsPath, 'utf8'))
    : {};

const source = JSON.parse(fs.readFileSync(hooksPath, 'utf8'));

// Deep-merge hooks: for each event type, append entries that aren't already present
const merged = { ...existing };
merged.hooks = merged.hooks || {};

for (const [event, entries] of Object.entries(source.hooks || {})) {
    merged.hooks[event] = merged.hooks[event] || [];
    for (const entry of entries) {
        const key = JSON.stringify(entry.hooks);
        const alreadyPresent = merged.hooks[event].some(
            e => JSON.stringify(e.hooks) === key
        );
        if (!alreadyPresent) {
            merged.hooks[event].push(entry);
        }
    }
}

fs.mkdirSync(require('path').dirname(settingsPath), { recursive: true });
fs.writeFileSync(settingsPath, JSON.stringify(merged, null, 2) + '\n');
console.log('Hooks merged successfully.');
NODE
fi

# ---------------------------------------------------------------------------
echo ""
echo "Done. Installed to $CLAUDE_DIR/"
echo "  agents:   $CLAUDE_DIR/agents/"
echo "  commands: $CLAUDE_DIR/commands/"
echo "  skills:   $CLAUDE_DIR/skills/"
echo "  rules:    $CLAUDE_DIR/rules/"
echo "  hooks:    $SETTINGS_FILE"
