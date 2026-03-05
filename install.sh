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
        echo "Next: run 'ecc install $lang' once to set up global rules/agents/skills."
    fi

    # --- .gitignore prompt ---
    local gitignore_file="$project_dir/.gitignore"
    local gitignore_entry=".claude/settings.local.json"
    local already_ignored=false

    if [[ -f "$gitignore_file" ]] && grep -qF "$gitignore_entry" "$gitignore_file" 2>/dev/null; then
        already_ignored=true
    fi

    if [[ "$already_ignored" == false ]]; then
        echo ""
        printf "Add '%s' to .gitignore? [Y/n] " "$gitignore_entry"
        read -r answer </dev/tty
        case "${answer:-Y}" in
            [Yy]*)
                echo "" >> "$gitignore_file"
                echo "# Claude Code local settings (machine-specific, never commit)" >> "$gitignore_file"
                echo "$gitignore_entry" >> "$gitignore_file"
                echo "  .gitignore             — $gitignore_entry added"
                ;;
            *)
                echo "  Skipped .gitignore update."
                ;;
        esac
    fi
}

# ---------------------------------------------------------------------------
# COMMAND: help
# ---------------------------------------------------------------------------
cmd_help() {
    local cmd="${1:-}"
    case "$cmd" in
        install)
            cat <<EOF
USAGE
  ecc install <language> [<language> ...]

DESCRIPTION
  Installs agents, commands, skills, rules, and hooks into ~/.claude/.
  Always installs common (language-agnostic) rules alongside each language.

ARGUMENTS
  <language>    One or more language names to install rules for.

OPTIONS
  (none)

EXAMPLES
  ecc install typescript
  ecc install typescript python golang

AVAILABLE LANGUAGES
$(list_languages)

WHAT GETS INSTALLED
  ~/.claude/agents/       — subagents (architect, uncle-bob, planner, ...)
  ~/.claude/commands/     — slash commands (/tdd, /plan, /code-review, ...)
  ~/.claude/skills/       — domain knowledge (tdd-workflow, security-review, ...)
  ~/.claude/rules/        — always-follow guidelines (common + <language>)
  ~/.claude/settings.json — hooks merged non-destructively
EOF
            ;;
        init)
            cat <<EOF
USAGE
  ecc init [--template <name>] [<language>]

DESCRIPTION
  Sets up Claude configuration for the current project directory.
  Auto-detects the language and template from project files if not specified.

ARGUMENTS
  <language>          Language for rule hints in the next-steps message.
                      Auto-detected from package.json, go.mod, Cargo.toml, etc.

OPTIONS
  --template <name>   CLAUDE.md template to use. Auto-detected if omitted.

EXAMPLES
  ecc init
  ecc init golang
  ecc init --template go-microservice golang

AVAILABLE TEMPLATES
$(list_templates)

AUTO-DETECTION
  Language   package.json → typescript, go.mod → golang, Cargo.toml → rust, ...
  Template   next in package.json → saas-nextjs, manage.py → django-api, ...

WHAT GETS CREATED
  CLAUDE.md               — project instructions, pre-filled from template
  .claude/settings.json   — project-local hooks merged non-destructively
EOF
            ;;
        ""|help)
            cat <<EOF
USAGE
  ecc <command> [options]

COMMANDS
  install <language> [<language> ...]
      Install agents, commands, skills, rules, and hooks globally into ~/.claude/

  init [--template <name>] [<language>]
      Set up Claude configuration for the current project.
      Creates CLAUDE.md and .claude/settings.json with hooks.

  help [<command>]
      Show help. Pass a command name for detailed usage.

  completion [bash|zsh|fish]
      Output shell completion script. Auto-detects shell if omitted.
      Add to shell: eval "\$(ecc completion)"

EXAMPLES
  ecc install typescript
  ecc install typescript python golang
  ecc init
  ecc init golang
  ecc init --template go-microservice golang
  ecc help install
  ecc help init

AVAILABLE LANGUAGES
$(list_languages)

AVAILABLE TEMPLATES
$(list_templates)
EOF
            ;;
        *)
            die "Unknown command '$cmd'. Run 'ecc help' for usage." ;;
    esac
}

# ---------------------------------------------------------------------------
# COMMAND: completion
# ---------------------------------------------------------------------------
cmd_completion() {
    local shell="${1:-}"

    # Auto-detect shell if not specified
    if [[ -z "$shell" ]]; then
        shell="$(basename "${SHELL:-bash}")"
    fi

    case "$shell" in
        zsh)
            cat <<'ZSHCOMP'
# ecc zsh completion — add to ~/.zshrc:
#   eval "$(ecc completion zsh)"

_ecc() {
    local -a commands languages templates
    commands=(
        'install:Install agents, commands, skills, rules, and hooks into ~/.claude/'
        'init:Set up Claude configuration for the current project'
        'help:Show help for a command'
        'completion:Output shell completion script'
    )

    _ecc_languages() {
        local langs
        langs=(${(f)"$(ecc --list-languages 2>/dev/null)"})
        _describe 'language' langs
    }

    _ecc_templates() {
        local tpls
        tpls=(${(f)"$(ecc --list-templates 2>/dev/null)"})
        _describe 'template' tpls
    }

    local state
    _arguments \
        '1: :->command' \
        '*: :->args' && return

    case $state in
        command)
            _describe 'command' commands ;;
        args)
            case $words[2] in
                install)
                    _ecc_languages ;;
                init)
                    _arguments \
                        '--template[CLAUDE.md template]:template:_ecc_templates' \
                        '::language:_ecc_languages' ;;
                help)
                    local help_cmds=('install' 'init' 'completion')
                    _describe 'command' help_cmds ;;
                completion)
                    local shells=('bash' 'zsh' 'fish')
                    _describe 'shell' shells ;;
            esac ;;
    esac
}

compdef _ecc ecc
ZSHCOMP
            ;;
        bash)
            cat <<'BASHCOMP'
# ecc bash completion — add to ~/.bashrc:
#   eval "$(ecc completion bash)"

_ecc_completion() {
    local cur prev words
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    words=("${COMP_WORDS[@]}")

    local commands="install init help completion"

    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return
    fi

    local cmd="${words[1]}"
    case "$cmd" in
        install)
            local langs
            langs="$(ecc --list-languages 2>/dev/null | tr '\n' ' ')"
            COMPREPLY=($(compgen -W "$langs" -- "$cur")) ;;
        init)
            if [[ "$prev" == "--template" ]]; then
                local tpls
                tpls="$(ecc --list-templates 2>/dev/null | tr '\n' ' ')"
                COMPREPLY=($(compgen -W "$tpls" -- "$cur"))
            else
                local langs
                langs="$(ecc --list-languages 2>/dev/null | tr '\n' ' ')"
                COMPREPLY=($(compgen -W "--template $langs" -- "$cur"))
            fi ;;
        help)
            COMPREPLY=($(compgen -W "install init completion" -- "$cur")) ;;
        completion)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur")) ;;
    esac
}

complete -F _ecc_completion ecc
BASHCOMP
            ;;
        fish)
            cat <<'FISHCOMP'
# ecc fish completion — add to ~/.config/fish/config.fish:
#   ecc completion fish | source

complete -c ecc -f

# Commands
complete -c ecc -n '__fish_use_subcommand' -a install    -d 'Install into ~/.claude/'
complete -c ecc -n '__fish_use_subcommand' -a init       -d 'Set up current project'
complete -c ecc -n '__fish_use_subcommand' -a help       -d 'Show help'
complete -c ecc -n '__fish_use_subcommand' -a completion -d 'Output completion script'

# install: complete with languages
complete -c ecc -n '__fish_seen_subcommand_from install' \
    -a "(ecc --list-languages 2>/dev/null)"

# init: --template flag + languages
complete -c ecc -n '__fish_seen_subcommand_from init' \
    -l template -d 'CLAUDE.md template' \
    -a "(ecc --list-templates 2>/dev/null)"
complete -c ecc -n '__fish_seen_subcommand_from init' \
    -a "(ecc --list-languages 2>/dev/null)"

# help: complete with command names
complete -c ecc -n '__fish_seen_subcommand_from help' \
    -a "install init completion"

# completion: complete with shell names
complete -c ecc -n '__fish_seen_subcommand_from completion' \
    -a "bash zsh fish"
FISHCOMP
            ;;
        *)
            die "Unknown shell '$shell'. Supported: bash, zsh, fish" ;;
    esac
}

# Internal helpers used by completion scripts at runtime
cmd_list_languages() {
    for dir in "$RULES_DIR"/*/; do
        name="$(basename "$dir")"
        [[ "$name" == "common" ]] && continue
        echo "$name"
    done
}

cmd_list_templates() {
    for f in "$EXAMPLES_DIR"/*-CLAUDE.md; do
        [[ -f "$f" ]] || continue
        basename "$f" -CLAUDE.md
    done
}

# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------
CMD="${1:-}"

case "$CMD" in
    install)
        shift; cmd_install "$@" ;;
    init)
        shift; cmd_init "$@" ;;
    help|-h|--help)
        shift; cmd_help "${1:-}" ;;
    completion)
        shift; cmd_completion "${1:-}" ;;
    --list-languages)
        cmd_list_languages ;;
    --list-templates)
        cmd_list_templates ;;
    "")
        cmd_help; exit 1 ;;
    *)
        die "Unknown command '$CMD'. Run 'ecc help' for usage." ;;
esac
