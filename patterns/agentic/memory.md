---
name: memory
category: agentic
tags: [agentic, memory, persistence, context]
languages: [python, typescript, rust]
difficulty: advanced
---

## Intent

Enable agents to retain and retrieve information across interactions, overcoming the context window limitation and enabling long-term learning.

## Problem

LLM agents are stateless between sessions — they forget previous interactions, decisions, and discoveries. The context window is finite, so even within a session, older information is lost. Agents repeatedly re-discover the same facts and make the same mistakes.

## Solution

Implement a tiered memory system: short-term (within session, context window), working memory (session-scoped storage), and long-term (persistent across sessions). Use semantic search to retrieve relevant memories. Support memory promotion (episodic to semantic) and garbage collection of stale entries.

## Language Implementations

### Python

```python
from dataclasses import dataclass
from enum import Enum

class MemoryTier(Enum):
    EPISODIC = "episodic"
    SEMANTIC = "semantic"
    PROCEDURAL = "procedural"

@dataclass(frozen=True)
class Memory:
    id: str
    content: str
    tier: MemoryTier
    tags: tuple[str, ...]
    embedding: tuple[float, ...] | None = None

def search_memories(
    store: list[Memory], query_embedding: tuple[float, ...], top_k: int = 5
) -> list[Memory]:
    scored = [(m, cosine_sim(query_embedding, m.embedding)) for m in store if m.embedding]
    scored.sort(key=lambda x: x[1], reverse=True)
    return [m for m, _ in scored[:top_k]]

def promote(memory: Memory) -> Memory:
    return Memory(
        id=memory.id, content=memory.content,
        tier=MemoryTier.SEMANTIC, tags=memory.tags, embedding=memory.embedding
    )
```

### Typescript

```typescript
type MemoryTier = "episodic" | "semantic" | "procedural";

interface Memory {
  readonly id: string;
  readonly content: string;
  readonly tier: MemoryTier;
  readonly tags: readonly string[];
}

function searchMemories(
  store: readonly Memory[], query: string, topK = 5
): readonly Memory[] {
  // In practice: use vector similarity search
  return store
    .filter(m => m.tags.some(t => query.includes(t)))
    .slice(0, topK);
}

function promote(memory: Memory): Memory {
  return { ...memory, tier: "semantic" };
}
```

### ECC Integration

ECC implements a full memory system via the `ecc memory` CLI commands (add, search, list, promote, gc, migrate). Memories are stored in SQLite with FTS5 full-text search. The three tiers (episodic, semantic, procedural) map to `--type` flags. The `ecc memory promote` command moves episodic memories to semantic tier. Agent frontmatter supports `memory: project` for cross-session concerns per `rules/ecc/development.md`. The Claude Code MEMORY.md in `.claude/projects/` provides user-level persistent memory.

## When to Use

- When agents need to retain discoveries across sessions (learned patterns, user preferences).
- When context windows are too small for all relevant information.
- When repeated work can be avoided by remembering previous results.

## When NOT to Use

- When tasks are fully self-contained with no cross-session value.
- When memory staleness is dangerous (rapidly changing environments).
- When privacy constraints prevent storing interaction data.

## Anti-Patterns

- Storing everything without curation — memory bloats and retrieval quality degrades.
- No garbage collection — stale memories pollute search results.
- Retrieving too many memories — flooding the context window with irrelevant history.

## Related Patterns

- [agentic/planning](planning.md) — retrieve relevant memories during plan creation.
- [agentic/reflection](reflection.md) — store reflection outcomes as memories for future reference.
- [agentic/react](react.md) — inject retrieved memories into reasoning context.

## References

- Park et al. — Generative Agents: Interactive Simulacra (2023): https://arxiv.org/abs/2304.03442
- Anthropic — Building effective agents: https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching
