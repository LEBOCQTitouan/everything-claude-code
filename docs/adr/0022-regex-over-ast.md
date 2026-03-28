# 0022. Regex Over AST for Artifact Parsing

Date: 2026-03-27

## Status

Accepted

## Context

Spec and design artifacts contain structured patterns (AC-NNN.N definitions, PC tables) embedded in Markdown. Two approaches were considered for extracting these patterns:

1. **Full Markdown AST**: Parse the entire document into an AST using a library such as `pulldown-cmark` or `comrak`, then traverse nodes to locate headings, list items, and table rows containing the relevant patterns.
2. **Line-by-line regex**: Scan the document line by line and match patterns directly with regular expressions. AC definitions match `^- AC-\d{3}\.\d+:`. PC table rows match pipe-delimited Markdown table syntax.

The patterns targeted by `ecc validate spec` and `ecc validate design` are line-based with a stable, well-documented format controlled by the ECC spec pipeline itself.

## Decision

Use regex-based line parsing rather than a full Markdown AST.

AC definition lines match: `^- AC-\d{3}\.\d+:`

PC table rows match: pipe-delimited Markdown rows where the first non-whitespace cell starts with `PC-\d{3}`.

Code block exclusion uses explicit state tracking for fenced blocks (` ``` ` and `~~~`) rather than relying on AST node types.

## Consequences

**Positive:**

- No new dependency — no `pulldown-cmark` or `comrak` crate added to the workspace
- Parsing is fast: line-scanning is O(n) with trivial constant factors, comfortably under 1ms for typical spec files vs ~10ms for a full AST parse
- Implementation is transparent — the regex patterns document the expected format directly in code, serving as executable format documentation
- Easier to extend with new pattern types without understanding an AST visitor API

**Negative:**

- Fragile if Markdown format changes — e.g., switching from bullet list `- AC-` to numbered list `1. AC-` would break extraction silently
- Code block exclusion via state tracking can mis-handle edge cases (unclosed fences, fences inside HTML blocks) that an AST would handle correctly
- No semantic understanding of document structure — a regex match in an unexpected section (e.g., a code example) can produce false positives if state tracking fails
