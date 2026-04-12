# openeruka — AGENTS.md

## For AI agents (Claude Code, ARES, pawan, etc.)

This crate provides Rust types and an async HTTP client for consuming a hosted Eruka context memory instance.

## What you can do with this crate

```rust
// Connect to an Eruka instance
let client = ErukaClient::new("https://eruka.dirmacs.com", &api_key);

// Read a field
client.get_field(workspace_id, "identity/company_name").await?;

// Read all fields under a prefix
client.get_prefix(workspace_id, "products").await?;

// Write a field (may return Conflict if Confirmed is protected)
client.write_field(workspace_id, &ErukaFieldWrite { ... }).await?;

// Knowledge graph
client.get_entities(workspace_id).await?;
client.get_edges(workspace_id).await?;
```

## Eruka MCP tools (via eruka-mcp)

If your agent uses the Eruka MCP server ([github.com/dirmacs/eruka-mcp](https://github.com/dirmacs/eruka-mcp)), you can call these tools:

- `eruka_get_context` — read a field or prefix
- `eruka_write_context` — write a field (knowledge-state-aware)
- `eruka_search_context` — semantic search across fields
- `eruka_get_related` — traverse the knowledge graph
- `eruka_detect_gaps` — find what's missing in a workspace

## Knowledge state model (critical for agents)

When writing context, always specify the correct knowledge state:
- `CONFIRMED` — you verified this from a reliable source (human, official doc)
- `INFERRED` — you derived this from reasoning or LLM output (may be wrong)
- `UNCERTAIN` — conflicting signals, use with low confidence
- `UNKNOWN` — you need this fact but don't have it (gap detection)

**The server will reject an `INFERRED` write to a `CONFIRMED` field.** This is intentional — it prevents LLM hallucinations from corrupting verified facts.

## Related

- [eruka-mcp](https://github.com/dirmacs/eruka-mcp) — MCP server
- [Eruka API docs](https://eruka.dirmacs.com)
- [openeruka docs](https://dirmacs.github.io/openeruka)
