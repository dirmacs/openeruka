# openeruka — AGENTS.md

## For AI agents (Claude Code, ARES, pawan, etc.)

This crate provides Rust types and an async HTTP client for consuming a hosted Eruka
context memory instance. The local-mode openeruka-server exposes the same API surface
with a SQLite backend.

## What you can do with this crate

"rust
// Connect to a hosted Eruka instance
let client = ErukaClient::new("https://eruka.dirmacs.com", &api_key);

// Or run locally (openeruka-server must be running at localhost:8080)
// No API key needed for local mode
let client = ErukaClient::new("http://localhost:8080", "local");

// Read a field
client.get_field(workspace_id, "identity/company_name").await?;

// Read all fields under a prefix (category prefix scan — BUG-1 fix)
client.get_prefix(workspace_id, "identity").await?;  // returns identity/* fields

// Write a field (may return Conflict if Confirmed is protected)
client.write_field(workspace_id, &ErukaFieldWrite { ... }).await?;

// Knowledge graph
client.get_entities(workspace_id).await?;
client.get_edges(workspace_id).await?;

// New in v0.2.1: server tier detection + compressed context stub
let tier = client.get_server_tier().await?;
let ctx = client.get_context_compressed(workspace_id, path, max_tokens).await?;
"

## API endpoints (v0.2.1)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/v1/context?workspace_id=&path=` | Read field or prefix (BUG-1: bare category paths now work) |
| POST | `/api/v1/context` | Write a field |
| GET | `/api/v1/entities?workspace_id=` | List knowledge graph entities |
| GET | `/api/v1/edges?workspace_id=` | List knowledge graph edges |
| GET | `/api/v1/tier` | Server tier (for eruka-mcp compatibility) |
| GET | `/api/v1/search?workspace_id=&query=&limit=` | Text-match search (no vector similarity) |
| GET | `/api/v1/gaps?workspace_id=` | Gap listing (stub — empty in local mode) |

## Knowledge state model (critical for agents)

When writing context, always specify the correct knowledge state:
- "CONFIRMED" — you verified this from a reliable source (human, official doc)
- "INFERRED" — you derived this from reasoning or LLM output (may be wrong)
- "UNCERTAIN" — conflicting signals, use with low confidence
- "UNKNOWN" — you need this fact but don't have it (gap detection)

**The server will reject an "INFERRED" write to a "CONFIRMED" field.**
This is intentional — it prevents LLM hallucinations from corrupting verified facts.

## Eruka MCP tools (via eruka-mcp)

eruka-mcp supports two backends, selected via environment variables:

**Local mode — openeruka:**
```bash
# Start openeruka server first
cargo install openeruka
openeruka serve  # starts at http://localhost:8080

# Then run eruka-mcp — connects to localhost:8080 by default (no API key needed)
eruka-mcp
```

**Managed mode — eruka.dirmacs.com:**
```bash
export ERUKA_API_URL=https://eruka.dirmacs.com
export ERUKA_API_KEY=eruka_sk_...
eruka-mcp
```

Tools available in local mode:
- `eruka_get_context` — read a field or prefix
- `eruka_write_context` — write a field (knowledge-state-aware)
- `eruka_search_context` — text match search (semantic via managed eruka)
- `eruka_get_related` — traverse the knowledge graph
- `eruka_get_gaps` — list knowledge gaps (stub — empty in local mode)
- `eruka_get_context_cached` — diff-based cached context
- `eruka_export_context` — export as portable JSON

## Limitations in local mode (openeruka-server)

openeruka is **single-tenant, local-mode, SQLite-based** — intentionally simpler than managed eruka:

| Feature | openeruka (local) | eruka.dirmacs.com (managed) |
|---------|-------------------|----------------------------|
| Vector search | No (text match only) | Yes (pgvector) |
| Gap tracking | Stub (empty) | Full |
| Quality scoring | No | Yes |
| Multi-workspace | No | Yes |
| Auth required | No | Yes (API key) |
| Backend | SQLite | PostgreSQL |

These are by design for lightweight local use. The hosted eruka provides full feature parity.

## Synced from managed eruka (V14.5, 2026-04-27)

This sync brings openeruka up to date with the managed eruka's dogfood QA fixes:
- **BUG-1**: category prefix scan — bare path "identity" now routes to prefix scan
- **BUG-3**: `/api/v1/tier` endpoint added for eruka-mcp startup detection
- New endpoints: `/api/v1/edges`, `/api/v1/search`, `/api/v1/gaps` (stub)
- `openeruka-client`: `get_server_tier()` and `get_context_compressed()` added

## Related

- [eruka-mcp](https://github.com/dirmacs/eruka-mcp) v0.2.x — MCP server (compatible with openeruka local mode)
- **eruka** — managed Eruka at eruka.dirmacs.com (V14.5, full feature parity — not public repo)
- [Eruka API docs](https://eruka.dirmacs.com)
- [openeruka docs](https://dirmacs.github.io/openeruka)
