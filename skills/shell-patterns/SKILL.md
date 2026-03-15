---
name: shell-patterns
description: Shell scripting patterns, safety practices, portability guidelines, and best practices for writing robust, maintainable Bash scripts.
origin: ECC
---

# Shell Scripting Patterns

Robust shell scripting patterns and best practices for writing safe, portable, and maintainable scripts.

## When to Activate

- Writing new shell scripts
- Reviewing shell scripts
- Debugging shell script failures
- Porting scripts between platforms

## Core Principles

### 1. Strict Mode

Every script starts with strict mode:

```bash
#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
```

### 2. Defensive Quoting

```bash
# Always quote variable expansions
echo "${variable}"
cp "${source_file}" "${dest_dir}/"

# Use arrays for commands with dynamic arguments
args=("-v" "--output" "${output_file}")
command "${args[@]}"

# Parameter expansion with defaults
name="${1:-default_value}"
dir="${OUTPUT_DIR:?'OUTPUT_DIR must be set'}"
```

### 3. Error Handling

```bash
# Trap for cleanup
tmpdir=$(mktemp -d) || exit 1
trap 'rm -rf "$tmpdir"' EXIT

# Die function for fatal errors
die() { echo "FATAL: $*" >&2; exit 1; }

# Check command existence
command -v jq >/dev/null 2>&1 || die "jq is required but not installed"
```

### 4. Portable Practices

```bash
# Use POSIX-compatible constructs when portability matters
# Prefer [[ ]] in bash-only scripts (safer than [ ])
# Use $(command) not backticks
# Use printf instead of echo for portable output

printf '%s\n' "$message"
```

### 5. Functions

```bash
# Good: Local variables, meaningful names, early returns
validate_input() {
    local -r input="$1"
    local -r max_length="${2:-255}"

    [[ -n "$input" ]] || { echo "Input is empty" >&2; return 1; }
    [[ ${#input} -le $max_length ]] || { echo "Input too long" >&2; return 1; }

    return 0
}
```

### 6. Logging

```bash
# Structured logging functions
log_info()  { echo "[INFO]  $(date '+%Y-%m-%d %H:%M:%S') $*"; }
log_warn()  { echo "[WARN]  $(date '+%Y-%m-%d %H:%M:%S') $*" >&2; }
log_error() { echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') $*" >&2; }
```

## Anti-Patterns

```bash
# BAD: Unquoted variables
cp $file $dest

# BAD: eval with user input
eval "$user_input"

# BAD: cd without error checking
cd /some/dir
do_stuff

# GOOD: cd with error handling
cd /some/dir || die "Cannot cd to /some/dir"

# BAD: Parsing ls output
for f in $(ls *.txt); do ...

# GOOD: Use globbing
for f in *.txt; do
    [[ -e "$f" ]] || continue
    ...
done
```

## Quick Reference

| Pattern | Description |
|---------|-------------|
| `set -euo pipefail` | Strict mode — exit on error, undefined vars, pipe failures |
| `"${var}"` | Always quote variable expansions |
| `local -r` | Read-only local variables |
| `mktemp` + `trap` | Safe temp file handling |
| `command -v` | Check if command exists |
| `"$@"` | Forward all arguments safely |
| `${var:-default}` | Default value expansion |
| `${var:?error}` | Required variable check |
