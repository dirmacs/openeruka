# openeruka

[![crates.io](https://img.shields.io/crates/v/openeruka.svg)](https://crates.io/crates/openeruka)
[![docs.rs](https://docs.rs/openeruka/badge.svg)](https://docs.rs/openeruka)
[![CI](https://github.com/dirmacs/openeruka/actions/workflows/ci.yml/badge.svg)](https://github.com/dirmacs/openeruka/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Open core types and HTTP client for the [Eruka](https://eruka.dirmacs.com) context memory system.

Eruka is a structured, knowledge-state-aware memory layer for AI agents. This crate gives you the Rust types and async HTTP client to integrate any Rust project with a hosted Eruka instance — without pulling in the full Eruka engine.

## What's in this crate

| Crate | What it provides |
|---|---|
| `openeruka` | Core types: `KnowledgeState`, `ErukaField`, `FieldPath`, `ErukaEntity`, `ErukaEdge` |
| `openeruka-client` | Typed async HTTP client for the [Eruka API](https://eruka.dirmacs.com) |

## Quick start

```toml
# Cargo.toml
[dependencies]
openeruka = "0.1"
openeruka-client = "0.1"
```

```rust
use openeruka_client::ErukaClient;
use openeruka::KnowledgeState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ErukaClient::new(
        "https://eruka.dirmacs.com",
        &std::env::var("ERUKA_API_KEY")?,
    );

    // Read a confirmed fact
    if let Some(field) = client.get_field("my-workspace-id", "identity/company_name").await? {
        println!("Company: {:?} ({})", field.value, field.knowledge_state);
    }

    // Write a new field
    // Note: Confirmed fields cannot be overwritten by Inferred writes — the
    // server enforces the knowledge state invariant (returns 409 Conflict).
    Ok(())
}
```

## The knowledge state invariant

The defining property of Eruka (and openeruka) is that not all facts are equal:

```
CONFIRMED > INFERRED > UNCERTAIN > UNKNOWN
```

A `Confirmed` fact (human-verified or system-certified) cannot be silently overwritten by an `Inferred` guess from an LLM. The server enforces this at the write path — no client-side configuration required.

```rust
use openeruka::KnowledgeState;

assert!( KnowledgeState::Confirmed.can_overwrite(&KnowledgeState::Inferred));
assert!(!KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Confirmed));
```

## DIRMACS ecosystem

openeruka connects to these projects:

- **[eruka-mcp](https://github.com/dirmacs/eruka-mcp)** — MCP server for Claude Desktop / Claude Code ([docs](https://dirmacs.github.io/eruka-mcp))
- **[Eruka hosted API](https://eruka.dirmacs.com)** — hosted Eruka instance
- **[ARES](https://github.com/dirmacs/ares)** — multi-agent runtime that consumes Eruka for context injection
- **[pawan](https://github.com/dirmacs/pawan)** — CLI coding agent with Eruka memory integration
- **[deagle](https://github.com/dirmacs/deagle)** — code intelligence engine
- **[dstack](https://github.com/dirmacs/dstack)** — dev stack tooling
- **[DIRMACS](https://dirmacs.com)** — the company building this stack

## Docs

Full documentation at **[dirmacs.github.io/openeruka](https://dirmacs.github.io/openeruka)**

## License

MIT — see [LICENSE](LICENSE)
