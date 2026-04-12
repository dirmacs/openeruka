# openeruka

[![crates.io](https://img.shields.io/crates/v/openeruka.svg)](https://crates.io/crates/openeruka)
[![docs.rs](https://docs.rs/openeruka/badge.svg)](https://docs.rs/openeruka)
[![CI](https://github.com/dirmacs/openeruka/actions/workflows/ci.yml/badge.svg)](https://github.com/dirmacs/openeruka/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**The open-source knowledge state memory server for AI agents.**

openeruka is a fully self-contained memory server that enforces the knowledge state invariant: **Confirmed facts cannot be overwritten by Inferred guesses.** Run it locally, connect [eruka-mcp](https://github.com/dirmacs/eruka-mcp) to it, and your AI agent gets grounded, protected memory. Ships as a single binary with SQLite bundled — production-ready in one command.

```bash
# Install the server binary
cargo install openeruka

# Start it
openeruka serve                  # default port 8080

# Connect Claude Code / Claude Desktop via eruka-mcp
# Default ERUKA_API_URL is http://localhost:8080 — no extra config needed
```

## The knowledge state invariant

```rust
use openeruka::KnowledgeState;

// The core contract — enforced server-side, not advisory
assert!( KnowledgeState::Confirmed.can_overwrite(&KnowledgeState::Inferred));
assert!(!KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Confirmed));
```

When your agent tries to overwrite a `Confirmed` fact with an `Inferred` guess, the server returns `409 Conflict`. LLM hallucinations cannot corrupt verified knowledge. **No other memory system enforces this.**

## Interaction surfaces

openeruka exposes three equivalent interfaces:

| Interface | Command | Use case |
|---|---|---|
| **REST API** | `openeruka serve` | Connect any HTTP client, agent, or `openeruka-client` |
| **MCP server** | `openeruka mcp` | Claude Desktop, Claude Code, Cursor via [eruka-mcp](https://github.com/dirmacs/eruka-mcp) |
| **CLI** | `openeruka get/set` | Quick reads and writes from the terminal |

All three share the same SQLite backend and enforce the same knowledge state rules.

## Quick start

```bash
# Install
cargo install openeruka

# Start REST server (SQLite backend, default ./eruka.db)
openeruka serve

# Write a confirmed fact
curl -X POST http://localhost:8080/api/v1/context \
  -H "Content-Type: application/json" \
  -d '{"workspace_id":"my-project","path":"identity/company_name","value":"Acme Corp","knowledge_state":"CONFIRMED","confidence":1.0,"source":"user_input"}'

# Try to overwrite with an inferred guess — rejected with 409
curl -X POST http://localhost:8080/api/v1/context \
  -H "Content-Type: application/json" \
  -d '{"workspace_id":"my-project","path":"identity/company_name","value":"Acme AI Labs","knowledge_state":"INFERRED","confidence":0.7,"source":"agent_inference"}'
# → 409 Conflict: field is CONFIRMED, write has lower knowledge state

# Read it back — still the original
curl "http://localhost:8080/api/v1/context?workspace_id=my-project&path=identity/company_name"
```

## Use as a library

```toml
[dependencies]
openeruka = { version = "0.2", default-features = false }
openeruka-client = "0.1"
```

```rust
// Embed SqliteContextStore in your own Rust agent
use openeruka::{SqliteContextStore, ContextStore, ErukaFieldWrite, KnowledgeState, SourceType};

let store = SqliteContextStore::open("./eruka.db").await?;

// Write enforces the invariant
store.write_field("my-ws", ErukaFieldWrite {
    path: "identity/name".to_string(),
    value: serde_json::json!("DIRMACS"),
    knowledge_state: KnowledgeState::Confirmed,
    confidence: 1.0,
    source: SourceType::UserInput,
}).await?;
```

## openeruka vs eruka.dirmacs.com

[eruka-mcp](https://github.com/dirmacs/eruka-mcp) can connect to either:

```bash
# Local openeruka (OSS, SQLite, single-tenant)
ERUKA_API_URL=http://localhost:8080 eruka-mcp

# Managed Eruka (PostgreSQL, multi-tenant, enterprise features)
ERUKA_API_URL=https://eruka.dirmacs.com ERUKA_API_KEY=sk:your-key eruka-mcp
```

| | openeruka (OSS) | eruka.dirmacs.com |
|---|---|---|
| Knowledge state enforcement | ✅ | ✅ |
| REST + MCP + CLI | ✅ | ✅ |
| SQLite (local) | ✅ | — |
| PostgreSQL (scalable) | — | ✅ |
| Multi-tenancy | — | ✅ |
| B6 quality scoring | — | ✅ |
| Datalog inference | — | ✅ |
| Gardener pipeline | — | ✅ |
| SLA / HIPAA / SSO | — | ✅ |

## DIRMACS ecosystem

- **[eruka-mcp](https://github.com/dirmacs/eruka-mcp)** — MCP client for openeruka and [eruka.dirmacs.com](https://eruka.dirmacs.com)
- **[ARES](https://github.com/dirmacs/ares)** — multi-agent runtime that uses Eruka for context
- **[pawan](https://github.com/dirmacs/pawan)** — CLI coding agent with Eruka memory
- **[deagle](https://github.com/dirmacs/deagle)** — code intelligence engine
- **[DIRMACS](https://dirmacs.com)** — the company behind this stack

## Docs

**[dirmacs.github.io/openeruka](https://dirmacs.github.io/openeruka)**

## License

MIT — see [LICENSE](LICENSE)
