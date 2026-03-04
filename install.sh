#!/usr/bin/env bash
# install.sh — Manage Claude Code configuration
#
# COMMANDS
#
#   install <language> [<language> ...]
#     Install agents, commands, skills, rules, and hooks globally into ~/.claude/
#
#   init [--template <name>] [<language>]
#     Set up Claude configuration for the current project directory.
#     Creates CLAUDE.md (from template) and .claude/settings.json (with hooks).
#
# EXAMPLES
#
#   ./install.sh install typescript
#   ./install.sh install typescript python golang
#   ./install.sh init
#   ./install.sh init golang
#   ./install.sh init --template go-microservice golang

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
EXAMPLES_DIR="$SCRIPT_DIR/02-examples"

CLAUDE_DIR="${CLAUDE_DIR:-$HOME/.claude}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
die() { echo "Error: $*" >&2; exit 1; }

list_languages() {
    for dir in "$RULES_DIR"/*/; do
        name="$(basename "$dir")"
        [[ "$name" == "common" ]] && continue
        echo "  - $name"
    done
}

list_templates() {
    for f in "$EXAMPLES_DIR"/*-CLAUDE.md; do
        [[ -f "$f" ]] || continue
        basename "$f" -CLAUDE.md | sed 's/^/  - /'
    done
}

validate_lang() {
    local lang="$1"
    [[ "$lang" =~ ^[a-zA-Z0-9_-]+$ ]] || die "Invalid language name '$lang'. Only alphanumeric, dash, and underscore allowed."
    [[ -d "$RULES_DIR/$lang" ]] || die "06-rules/$lang/ does not exist. Available languages:$(echo; list_languages)"
}

# Merge hooks from source hooks.json into a settings.json file
merge_hooks() {
    local settings_file="$1"
    if ! command -v node &>/dev/null; then
        echo "Warning: node not found — skipping hooks merge. Add hooks manually from 07-hooks/hooks.json." >&2
        return
    fi
    node - "$settings_file" "$HOOKS_FILE" "$SCRIPT_DIR" <<'NODE'
const fs = require('fs');
const path = require('path');
const [, , settingsPath, hooksPath, pluginRoot] = process.argv;

const existing = fs.existsSync(settingsPath)
    ? JSON.parse(fs.readFileSync(settingsPath, 'utf8'))
    : {};

// Replace ${CLAUDE_PLUGIN_ROOT} placeholder with the actual install path
const raw = fs.readFileSync(hooksPath, 'utf8')
    .replaceAll('${CLAUDE_PLUGIN_ROOT}', pluginRoot);
const source = JSON.parse(raw);

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

fs.mkdirSync(path.dirname(settingsPath), { recursive: true });
fs.writeFileSync(settingsPath, JSON.stringify(merged, null, 2) + '\n');
NODE
}

# ---------------------------------------------------------------------------
# Auto-detect language from project files
# ---------------------------------------------------------------------------
detect_language() {
    local dir="${1:-.}"
    # Check for lock files / manifests in priority order
    [[ -f "$dir/package.json" ]]          && { echo "typescript"; return; }
    [[ -f "$dir/go.mod" ]]                && { echo "golang"; return; }
    [[ -f "$dir/requirements.txt" ]]      && { echo "python"; return; }
    [[ -f "$dir/pyproject.toml" ]]        && { echo "python"; return; }
    [[ -f "$dir/Pipfile" ]]               && { echo "python"; return; }
    [[ -f "$dir/Cargo.toml" ]]            && { echo "rust"; return; }
    [[ -f "$dir/pom.xml" ]]               && { echo "java"; return; }
    [[ -f "$dir/build.gradle" ]]          && { echo "java"; return; }
    echo ""
}

# Auto-detect template from project files
detect_template() {
    local dir="${1:-.}"
    # Specific framework detection takes priority
    if [[ -f "$dir/package.json" ]]; then
        grep -q '"next"' "$dir/package.json" 2>/dev/null && { echo "saas-nextjs"; return; }
    fi
    [[ -f "$dir/go.mod" ]]                && { echo "go-microservice"; return; }
    [[ -f "$dir/manage.py" ]]             && { echo "django-api"; return; }
    [[ -f "$dir/Cargo.toml" ]]            && { echo "rust-api"; return; }
    echo "default"
}

# ---------------------------------------------------------------------------
# COMMAND: install
# ---------------------------------------------------------------------------
cmd_install() {
    [[ $# -eq 0 ]] && {
        echo "Usage: $0 install <language> [<language> ...]"
        echo ""
        echo "Available languages:"
        list_languages
        exit 1
    }

    for lang in "$@"; do validate_lang "$lang"; done

    echo "Installing agents   -> $CLAUDE_DIR/agents/"
    mkdir -p "$CLAUDE_DIR/agents"
    cp "$AGENTS_DIR"/*.md "$CLAUDE_DIR/agents/"

    echo "Installing commands -> $CLAUDE_DIR/commands/"
    mkdir -p "$CLAUDE_DIR/commands"
    cp "$COMMANDS_DIR"/*.md "$CLAUDE_DIR/commands/"

    echo "Installing skills   -> $CLAUDE_DIR/skills/"
    mkdir -p "$CLAUDE_DIR/skills"
    cp -r "$SKILLS_DIR"/. "$CLAUDE_DIR/skills/"

    RULES_DEST="$CLAUDE_DIR/rules"
    if [[ -d "$RULES_DEST" ]] && [[ "$(ls -A "$RULES_DEST" 2>/dev/null)" ]]; then
        echo "Note: $RULES_DEST/ already exists — files will be overwritten."
    fi

    echo "Installing rules    -> $RULES_DEST/common/"
    mkdir -p "$RULES_DEST/common"
    cp -r "$RULES_DIR/common/." "$RULES_DEST/common/"

    for lang in "$@"; do
        echo "Installing rules    -> $RULES_DEST/$lang/"
        mkdir -p "$RULES_DEST/$lang"
        cp -r "$RULES_DIR/$lang/." "$RULES_DEST/$lang/"
    done

    echo "Merging hooks       -> $CLAUDE_DIR/settings.json"
    merge_hooks "$CLAUDE_DIR/settings.json"

    echo ""
    echo "Done. Installed to $CLAUDE_DIR/"
    echo "  agents:   $CLAUDE_DIR/agents/"
    echo "  commands: $CLAUDE_DIR/commands/"
    echo "  skills:   $CLAUDE_DIR/skills/"
    echo "  rules:    $CLAUDE_DIR/rules/"
    echo "  hooks:    $CLAUDE_DIR/settings.json"
}

# ---------------------------------------------------------------------------
# COMMAND: init
# ---------------------------------------------------------------------------
cmd_init() {
    local template=""
    local lang=""
    local project_dir
    project_dir="$(pwd)"

    # Parse flags
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --template)
                [[ -z "${2:-}" ]] && die "--template requires a value."
                template="$2"; shift 2 ;;
            --template=*)
                template="${1#--template=}"; shift ;;
            -*)
                die "Unknown flag: $1" ;;
            *)
                [[ -z "$lang" ]] || die "Too many arguments. Usage: $0 init [--template <name>] [<language>]"
                lang="$1"; shift ;;
        esac
    done

    # Auto-detect language if not provided
    if [[ -z "$lang" ]]; then
        lang="$(detect_language "$project_dir")"
        if [[ -n "$lang" ]]; then
            echo "Detected language: $lang"
        else
            echo "Warning: could not detect language. CLAUDE.md will use the generic template." >&2
        fi
    else
        validate_lang "$lang"
    fi

    # Auto-detect template if not provided
    if [[ -z "$template" ]]; then
        template="$(detect_template "$project_dir")"
    fi

    # --- CLAUDE.md ---
    local claude_md="$project_dir/CLAUDE.md"
    local template_file="$EXAMPLES_DIR/${template}-CLAUDE.md"

    if [[ -f "$claude_md" ]]; then
        echo "Warning: CLAUDE.md already exists — skipping. Delete it first to regenerate." >&2
    else
        if [[ -f "$template_file" ]]; then
            echo "Creating CLAUDE.md from template '$template'."
            cp "$template_file" "$claude_md"
        else
            echo "Creating CLAUDE.md from generic template."
            cp "$EXAMPLES_DIR/CLAUDE.md" "$claude_md"
        fi
        echo "  -> Edit $claude_md to describe your project."
    fi

    # --- .claude/settings.json ---
    local settings_file="$project_dir/.claude/settings.json"
    echo "Merging hooks -> $settings_file"
    merge_hooks "$settings_file"

    echo ""
    echo "Done. Project configured at $project_dir"
    echo "  CLAUDE.md              — project instructions for Claude"
    echo "  .claude/settings.json  — project-local hooks"
    if [[ -n "$lang" ]] && [[ "$lang" != "$(detect_language "$project_dir")" || true ]]; then
        echo ""
        echo "Next: run './install.sh install $lang' once to set up global rules/agents/skills."
    fi
}

# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------
CMD="${1:-}"

case "$CMD" in
    install)
        shift
        cmd_install "$@" ;;
    init)
        shift
        cmd_init "$@" ;;
    "")
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  install <language> [<language> ...]"
        echo "      Install agents, commands, skills, rules, and hooks globally into ~/.claude/"
        echo ""
        echo "  init [--template <name>] [<language>]"
        echo "      Set up Claude configuration for the current project."
        echo "      Creates CLAUDE.md and .claude/settings.json with hooks."
        echo ""
        echo "Available languages:"
        list_languages
        echo ""
        echo "Available templates (for --template):"
        list_templates
        exit 1 ;;
    *)
        die "Unknown command '$CMD'. Run '$0' with no arguments to see usage." ;;
esac
