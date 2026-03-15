#!/usr/bin/env bash
# install.sh — Manage Claude Code configuration
#
# COMMANDS
#
#   install [<language> ...]
#     Install agents, commands, skills, rules, and hooks globally into ~/.claude/
#     Auto-detects the project language if none specified.
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

AGENTS_DIR="$SCRIPT_DIR/agents"
COMMANDS_DIR="$SCRIPT_DIR/commands"
SKILLS_DIR="$SCRIPT_DIR/skills"
RULES_DIR="$SCRIPT_DIR/rules"
HOOKS_FILE="$SCRIPT_DIR/hooks/hooks.json"
EXAMPLES_DIR="$SCRIPT_DIR/examples"

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
    [[ -d "$RULES_DIR/$lang" ]] || die "rules/$lang/ does not exist. Available languages:$(echo; list_languages)"
}

# Merge hooks from source hooks.json into a settings.json file
merge_hooks() {
    local settings_file="$1"
    if ! command -v python3 &>/dev/null; then
        echo "Warning: python3 not found — skipping hooks merge. Add hooks manually from hooks/hooks.json." >&2
        return
    fi
    python3 - "$settings_file" "$HOOKS_FILE" <<'PYMERGE'
import json, os, sys

settings_path, hooks_path = sys.argv[1], sys.argv[2]

if os.path.exists(settings_path):
    with open(settings_path) as f:
        existing = json.load(f)
else:
    existing = {}

with open(hooks_path) as f:
    source = json.load(f)

merged = dict(existing)
merged.setdefault("hooks", {})

# Remove legacy/stale ECC hooks
legacy_patterns = [
    "@lebocqtitouan/ecc/", "everything-claude-code/",
    "${ECC_ROOT}", "${CLAUDE_PLUGIN_ROOT}",
    "/dist/hooks/run-with-flags.js",
    "/scripts/hooks/run-with-flags-shell.sh",
]
legacy_node_patterns = [
    "dev-server", "tmux", "git push", "console.log",
    "check-console", "pr-created", "build-complete",
]

def is_legacy(cmd):
    if cmd.startswith("ecc-hook ") or cmd.startswith("ecc-shell-hook "):
        return False
    for p in legacy_patterns:
        if p in cmd:
            return True
    if "scripts/hooks/" in cmd and "run-with-flags-shell.sh" not in cmd:
        return True
    if "node -e" in cmd and any(lp in cmd for lp in legacy_node_patterns):
        return True
    return False

for event, entries in list(merged["hooks"].items()):
    if not isinstance(entries, list):
        continue
    merged["hooks"][event] = [
        entry for entry in entries
        if not isinstance(entry.get("hooks"), list)
        or not any(is_legacy(h.get("command", "")) for h in entry["hooks"])
    ]

# Merge source hooks
for event, entries in (source.get("hooks") or {}).items():
    merged["hooks"].setdefault(event, [])
    for entry in entries:
        key = json.dumps(entry.get("hooks"), sort_keys=True)
        already = any(
            json.dumps(e.get("hooks"), sort_keys=True) == key
            for e in merged["hooks"][event]
        )
        if not already:
            merged["hooks"][event].append(entry)

os.makedirs(os.path.dirname(settings_path) or ".", exist_ok=True)
with open(settings_path, "w") as f:
    json.dump(merged, f, indent=2)
    f.write("\n")
PYMERGE
}

# Merge missing ## sections from a template file into an existing CLAUDE.md.
# Prints the number of sections added.
merge_claude_md() {
    local existing="$1"
    local template="$2"

    local existing_headings
    existing_headings=$(grep "^## " "$existing" 2>/dev/null)

    local added=0
    local in_section=0
    local section_heading=""
    local section_buf=""

    while IFS= read -r line || [[ -n "$line" ]]; do
        if [[ "$line" =~ ^##\  ]]; then
            if [[ $in_section -eq 1 ]]; then
                if ! echo "$existing_headings" | grep -qF "$section_heading"; then
                    printf '\n%s' "$section_buf" >> "$existing"
                    added=$((added + 1))
                fi
            fi
            section_heading="$line"
            section_buf="${line}"$'\n'
            in_section=1
        elif [[ $in_section -eq 1 ]]; then
            section_buf+="${line}"$'\n'
        fi
    done < "$template"

    # Flush last section
    if [[ $in_section -eq 1 ]]; then
        if ! echo "$existing_headings" | grep -qF "$section_heading"; then
            printf '\n%s' "$section_buf" >> "$existing"
            added=$((added + 1))
        fi
    fi

    echo "$added"
}

# If project_dir is inside a git repo, ask user whether to commit the written files.
maybe_git_commit() {
    local project_dir="$1"
    shift
    local files=("$@")

    git -C "$project_dir" rev-parse --git-dir &>/dev/null 2>&1 || return

    printf "Commit changes to git? [Y/n] "
    local answer
    read -r answer </dev/tty
    [[ "${answer:-Y}" =~ ^[Yy]$ ]] || return

    for f in "${files[@]}"; do
        [[ -e "$project_dir/$f" ]] && git -C "$project_dir" add -- "$project_dir/$f"
    done
    git -C "$project_dir" commit -m "chore: initialize Claude Code configuration"
    echo "Committed."
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
    [[ -f "$dir/Package.swift" ]]         && { echo "swift"; return; }
    [[ -n "$(ls "$dir"/*.xcodeproj 2>/dev/null)" ]] && { echo "swift"; return; }
    [[ -f "$dir/Podfile" ]]               && { echo "swift"; return; }
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
    # Language-specific generic fallbacks
    local detected_lang
    detected_lang="$(detect_language "$dir")"
    case "$detected_lang" in
        typescript) echo "typescript"; return ;;
        python)     echo "python";     return ;;
        swift)      echo "swift";      return ;;
    esac
    echo "default"
}

# ---------------------------------------------------------------------------
# COMMAND: install
# ---------------------------------------------------------------------------
cmd_install() {
    # Delegate to Rust CLI if available
    if command -v ecc >/dev/null 2>&1; then
        exec ecc install "$@"
    fi

    local dry_run=""
    local force=""
    local no_interactive=""
    local clean=""
    local clean_all=""
    local langs=()

    # Parse flags and languages
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --dry-run)        dry_run="--dry-run"; shift ;;
            --force)          force="--force"; shift ;;
            --no-interactive) no_interactive="--no-interactive"; shift ;;
            --clean)          clean="--clean"; shift ;;
            --clean-all)      clean_all="--clean-all"; shift ;;
            -*)               die "Unknown flag: $1" ;;
            *)                langs+=("$1"); shift ;;
        esac
    done

    # Auto-detect language if none provided
    if [[ ${#langs[@]} -eq 0 ]]; then
        local detected
        detected="$(detect_language "$(pwd)")"
        if [[ -n "$detected" ]]; then
            echo "Detected language: $detected"
            langs+=("$detected")
        else
            echo "Usage: $0 install [<language> ...] [--dry-run] [--force]"
            echo ""
            echo "No language specified and auto-detection failed."
            echo "Available languages:"
            list_languages
            exit 1
        fi
    fi

    for lang in "${langs[@]}"; do validate_lang "$lang"; done

    # Fallback: legacy cp-based install (no detection/merge/manifest)
    if [[ -n "$clean" ]]; then
        echo "Warning: --clean requires the Node.js orchestrator (manifest-aware). Use --clean-all instead." >&2
    fi
    if [[ -n "$clean_all" ]]; then
        echo "Cleaning all ECC artifacts (--clean-all, legacy fallback)..." >&2
        rm -rf "$CLAUDE_DIR/agents" "$CLAUDE_DIR/commands" "$CLAUDE_DIR/skills" "$CLAUDE_DIR/rules"
        rm -f "$CLAUDE_DIR/.ecc-manifest.json"
        echo "Cleaned." >&2
    fi
    echo "Warning: Node.js orchestrator not available — using legacy install (overwrites all)." >&2
    echo ""

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
    echo "Installing rules    -> $RULES_DEST/common/"
    mkdir -p "$RULES_DEST/common"
    cp -r "$RULES_DIR/common/." "$RULES_DEST/common/"

    for lang in "${langs[@]}"; do
        echo "Installing rules    -> $RULES_DEST/$lang/"
        mkdir -p "$RULES_DEST/$lang"
        cp -r "$RULES_DIR/$lang/." "$RULES_DEST/$lang/"
    done

    echo "Merging hooks       -> $CLAUDE_DIR/settings.json"
    merge_hooks "$CLAUDE_DIR/settings.json"

    echo ""
    echo "Done. Installed to $CLAUDE_DIR/"
}

# ---------------------------------------------------------------------------
# COMMAND: init
# ---------------------------------------------------------------------------
cmd_init() {
    # Delegate to Rust CLI if available
    if command -v ecc >/dev/null 2>&1; then
        exec ecc init "$@"
    fi

    local template=""
    local lang=""
    local no_gitignore=""
    local dry_run=""
    local force=""
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
            --no-gitignore)
                no_gitignore="--no-gitignore"; shift ;;
            --dry-run)
                dry_run="--dry-run"; shift ;;
            --force)
                force="--force"; shift ;;
            -*)
                die "Unknown flag: $1" ;;
            *)
                [[ -z "$lang" ]] || die "Too many arguments. Usage: $0 init [--template <name>] [<language>] [--no-gitignore] [--dry-run] [--force]"
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

    local tpl_file
    if [[ -n "$template" ]] && [[ -f "$EXAMPLES_DIR/${template}-CLAUDE.md" ]]; then
        tpl_file="$EXAMPLES_DIR/${template}-CLAUDE.md"
    else
        tpl_file="$EXAMPLES_DIR/CLAUDE.md"
    fi

    if [[ -f "$claude_md" ]]; then
        echo "CLAUDE.md already exists — merging missing sections from template '${template:-default}'."
        local added
        added=$(merge_claude_md "$claude_md" "$tpl_file")
        if [[ "$added" -eq 0 ]]; then
            echo "No new sections to add — CLAUDE.md is already up to date."
        else
            echo "Added $added new section(s) to CLAUDE.md."
        fi
    else
        echo "Creating CLAUDE.md from template '${template:-default}'."
        cp "$tpl_file" "$claude_md"
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

    # --- .gitignore management ---
    if [[ -z "$no_gitignore" ]]; then
        # Fallback: legacy single-entry gitignore
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
    fi

    maybe_git_commit "$project_dir" \
        "CLAUDE.md" \
        ".claude/settings.json" \
        ".gitignore"
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
  ecc install [<language> ...] [--dry-run] [--force] [--clean] [--clean-all]

DESCRIPTION
  Installs agents, commands, skills, rules, and hooks into ~/.claude/.
  Auto-detects the project language if none specified (from package.json,
  go.mod, Cargo.toml, etc.). Detects existing setup and merges intelligently:
    - ECC-managed files are updated automatically
    - User-custom files prompt for conflict resolution
    - Smart merge with Claude is available for complex conflicts

ARGUMENTS
  <language>    One or more language names. Auto-detected if omitted.

OPTIONS
  --dry-run          Report what would change without writing any files
  --force            Overwrite all files without prompting
  --no-interactive   Accept all changes without interactive review
  --clean            Remove manifest-tracked ECC files before reinstalling
                     (safe — only removes files ECC previously installed)
  --clean-all        Remove ALL ECC directories before reinstalling
                     (nuclear — wipes agents/, commands/, skills/, rules/,
                      removes ECC hooks from settings.json and manifest)

EXAMPLES
  ecc install                          (auto-detect language)
  ecc install typescript
  ecc install typescript python golang
  ecc install --dry-run
  ecc install typescript --force
  ecc install --no-interactive
  ecc install --clean                  (remove old ECC files, then reinstall)
  ecc install --clean --dry-run        (preview what --clean would remove)
  ecc install --clean-all --force      (full wipe + reinstall, no prompts)

AVAILABLE LANGUAGES
$(list_languages)

WHAT GETS INSTALLED
  ~/.claude/agents/           — subagents (architect, uncle-bob, planner, ...)
  ~/.claude/commands/         — slash commands (/tdd, /plan, /code-review, ...)
  ~/.claude/skills/           — domain knowledge (tdd-workflow, security-review, ...)
  ~/.claude/rules/            — always-follow guidelines (common + <language>)
  ~/.claude/settings.json     — hooks merged non-destructively
  ~/.claude/.ecc-manifest.json — tracks installed artifacts for future merges
EOF
            ;;
        init)
            cat <<EOF
USAGE
  ecc init [--template <name>] [<language>] [--no-gitignore] [--dry-run] [--force]

DESCRIPTION
  Sets up Claude configuration for the current project directory.
  Auto-detects the language and template from project files if not specified.
  Manages .gitignore to exclude ECC-generated runtime files.

ARGUMENTS
  <language>          Language for rule hints in the next-steps message.
                      Auto-detected from package.json, go.mod, Cargo.toml, etc.

OPTIONS
  --template <name>   CLAUDE.md template to use. Auto-detected if omitted.
  --no-gitignore      Skip .gitignore management
  --dry-run           Report what would change without writing any files
  --force             Overwrite all files (including user-custom ones)

EXAMPLES
  ecc init
  ecc init golang
  ecc init --template go-microservice golang
  ecc init --no-gitignore

AVAILABLE TEMPLATES
$(list_templates)

AUTO-DETECTION
  Language   package.json → typescript, go.mod → golang, Cargo.toml → rust, ...
  Template   next in package.json → saas-nextjs, manage.py → django-api, ...

WHAT GETS CREATED/UPDATED
  CLAUDE.md                   — project instructions, pre-filled from template
  .claude/settings.json       — project-local hooks merged non-destructively
  .gitignore                  — ECC-generated files excluded from git
  .claude/.ecc-manifest.json  — tracks installed artifacts
EOF
            ;;
        uninstall)
            cat <<EOF
USAGE
  ecc uninstall [--force] [--keep-config]

DESCRIPTION
  Remove ECC from the system. Cleans up ~/.ecc/ (binary), ECC artifacts
  from ~/.claude/ (agents, commands, skills, rules, manifest), and PATH
  entries from shell RC files.

OPTIONS
  --force        Skip confirmation prompt
  --keep-config  Only remove ~/.ecc/ and PATH entries; keep ~/.claude/ artifacts

EXAMPLES
  ecc uninstall                  (interactive confirmation)
  ecc uninstall --force          (no confirmation)
  ecc uninstall --keep-config    (preserve ~/.claude/ artifacts)
EOF
            ;;
        version)
            cat <<EOF
USAGE
  ecc version

DESCRIPTION
  Print the installed ecc version number.

ALIASES
  ecc --version
  ecc -v
EOF
            ;;
        update)
            cat <<EOF
USAGE
  ecc update

DESCRIPTION
  Reinstall the latest version of ecc from GitHub Releases.
  Re-runs the curl installer to download the latest binary.
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

  version
      Print the installed ecc version.  Aliases: --version, -v

  update
      Reinstall the latest version of ecc from GitHub Releases.

  uninstall [--force] [--keep-config]
      Remove ECC from the system (binary, content, PATH entries).

  help [<command>]
      Show help. Pass a command name for detailed usage.

  completion [bash|zsh|fish|pwsh]
      Output shell completion script. Auto-detects shell if omitted.
      Add to shell: eval "\$(ecc completion)"

EXAMPLES
  ecc install typescript
  ecc install typescript python golang
  ecc init
  ecc init golang
  ecc init --template go-microservice golang
  ecc version
  ecc update
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
        'uninstall:Remove ECC from the system'
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
                uninstall)
                    _arguments \
                        '--force[Skip confirmation]' \
                        '--keep-config[Keep ~/.claude/ artifacts]' ;;
                help)
                    local help_cmds=('install' 'init' 'uninstall' 'version' 'update' 'completion')
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

    local commands="install init uninstall help version update completion"

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
        uninstall)
            COMPREPLY=($(compgen -W "--force --keep-config" -- "$cur")) ;;
        help)
            COMPREPLY=($(compgen -W "install init uninstall version update completion" -- "$cur")) ;;
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
complete -c ecc -n '__fish_use_subcommand' -a version    -d 'Print installed version'
complete -c ecc -n '__fish_use_subcommand' -a update     -d 'Update to latest version'
complete -c ecc -n '__fish_use_subcommand' -a uninstall  -d 'Remove ECC from the system'
complete -c ecc -n '__fish_use_subcommand' -a help       -d 'Show help'
complete -c ecc -n '__fish_use_subcommand' -a completion -d 'Output completion script'

# uninstall: --force and --keep-config flags
complete -c ecc -n '__fish_seen_subcommand_from uninstall' \
    -l force -d 'Skip confirmation'
complete -c ecc -n '__fish_seen_subcommand_from uninstall' \
    -l keep-config -d 'Keep ~/.claude/ artifacts'

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
    -a "install init uninstall version update completion"

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
    version|--version|-v)
        if command -v ecc &>/dev/null; then
            ecc version
        else
            grep -m1 '^version' "$(dirname "$0")/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
        fi
        ;;
    update)
        echo "Updating ECC to latest..."
        curl -fsSL "https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh" | bash
        ;;
    uninstall)
        shift; exec "$SCRIPT_DIR/scripts/uninstall-ecc.sh" "$@"
        ;;
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
