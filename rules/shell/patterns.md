---
paths:
  - "**/*.sh"
  - "**/*.bash"
  - "**/*.zsh"
applies-to: { files: ["*.sh"] }
---
# Shell Patterns

> This file extends [common/patterns.md](../common/patterns.md) with shell specific content.

## Function Design

```bash
# Good: Small, focused functions with local variables
process_file() {
    local file="$1"
    local -r output_dir="$2"

    [[ -f "$file" ]] || { echo "File not found: $file" >&2; return 1; }

    cp "$file" "${output_dir}/"
}
```

## Error Handling

```bash
# Trap for cleanup
cleanup() {
    rm -rf "${tmpdir:-}"
}
trap cleanup EXIT

# Die function
die() {
    echo "ERROR: $*" >&2
    exit 1
}
```

## Argument Parsing

```bash
while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--verbose) verbose=true; shift ;;
        -o|--output)  output="$2"; shift 2 ;;
        -h|--help)    usage; exit 0 ;;
        --)           shift; break ;;
        -*)           die "Unknown option: $1" ;;
        *)            break ;;
    esac
done
```

## Reference

See skill: `shell-patterns` for comprehensive shell scripting patterns.
