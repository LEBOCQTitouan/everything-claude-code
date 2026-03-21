---
name: ubiquitous-language
description: Domain language extraction skill. Reads source code to extract types, function names, doc comments, test names, and error variants into a structured glossary. Use when the user says "ubiquitous language", "domain glossary", "bounded context terms", or "extract domain vocabulary".
origin: ECC
---

# Ubiquitous Language Extraction

Extract the domain vocabulary from a bounded context or module path. Output a structured glossary — extraction only, never synthesis.

## When to Activate

- User says "ubiquitous language", "domain glossary", "bounded context terms"
- User says "extract domain vocabulary", "what terms does this module use"
- User wants to document the ubiquitous language of a specific code area

## Input

`$ARGUMENTS` is a bounded context name or module path (e.g., `crates/ecc-domain`, `src/auth`).

If `$ARGUMENTS` is empty, scan the entire project root.

If the path does not exist, report an error and stop: "Path not found: `<path>`. Provide a valid module path or bounded context name."

## Extraction Steps

1. **Scan** the target path for source files using Glob
2. **Extract** domain-relevant identifiers using Grep:
   - Type/struct/class/interface/enum names
   - Public function and method names
   - Doc comment summaries (first line)
   - Test function names (strip `test_` prefix for the underlying concept)
   - Error variant names
3. **Deduplicate** — group aliases (e.g., `Order` type and `order_id` field refer to the same concept)
4. **Define** each term from its doc comment or usage context (do not invent definitions — use what the code says)
5. **Format** as a Markdown glossary table

## Output Format

Write to `docs/domain/glossary-{context}.md` where `{context}` is the directory name or bounded context slug. If scanning the full project, write to `docs/domain/glossary-all.md`.

```markdown
# Glossary: {context}

Extracted from `{path}` on {date}.

| Term | Definition | Source | Aliases |
|------|-----------|--------|---------|
| Order | A customer purchase with line items | src/domain/order.rs:12 | OrderAggregate |
| OrderId | Value object wrapping order identifier | src/domain/order.rs:5 | order_id |
```

## Anti-Patterns

- DO NOT invent terms that don't exist in the code — extraction only, never synthesis
- DO NOT include infrastructure terms (database columns, HTTP routes) unless they are also domain concepts
- DO NOT guess definitions — if no doc comment exists, describe based on usage context and mark as "(inferred)"
- DO NOT include every identifier — focus on domain-meaningful terms that appear in 2+ files or are central types
