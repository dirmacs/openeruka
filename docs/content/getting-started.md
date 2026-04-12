+++
title = "Getting Started"
description = "Install openeruka and connect it to your AI agent in 5 minutes"
+++

## Requirements

- Rust 1.75+ and Cargo
- No other dependencies (SQLite is bundled)

## Install the server

```bash
cargo install openeruka-server
```

This installs the `openeruka` binary.

## Start the server

```bash
openeruka serve
```

Default: listens on `http://0.0.0.0:8080`, stores data in `./eruka.db` (SQLite).

Options:

```bash
openeruka serve --port 9090          # custom port
openeruka --db /data/ctx.db serve    # custom DB path
```

## Connect eruka-mcp

Install [eruka-mcp](https://github.com/dirmacs/eruka-mcp) — the MCP server that bridges openeruka to Claude Desktop, Claude Code, and any MCP client:

```bash
cargo install eruka-mcp
```

**Claude Code** (one command):
```bash
claude mcp add eruka eruka-mcp
```

**Claude Desktop** — add to `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "eruka": {
      "command": "eruka-mcp"
    }
  }
}
```

`eruka-mcp` defaults to `http://localhost:8080` — no extra config needed when running openeruka locally.

## Test via CLI

```bash
# Read all context for the default workspace
eruka-mcp get "*"

# Write a field
eruka-mcp write "identity/company" "ACME Corp"

# Check health
eruka-mcp health
```

## Use the REST API directly

```bash
# Write a field
curl -s http://localhost:8080/fields -X POST \
  -H "Content-Type: application/json" \
  -d '{"workspace_id":"default","path":"identity/name","value":"ACME","knowledge_state":"confirmed","confidence":1.0,"source":"user_input"}'

# Read it back
curl -s "http://localhost:8080/fields/default/identity/name"
```

## Switching backends

openeruka ships two storage backends:

| Backend | Command | Notes |
|---------|---------|-------|
| SQLite (default) | `openeruka serve` | Zero setup, file-based |
| redb | `openeruka --backend redb serve` | Embedded KV, requires `--features redb` build |

To use redb, build from source with the feature enabled:

```bash
git clone https://github.com/dirmacs/openeruka
cd openeruka
cargo build --release --features redb
./target/release/openeruka --backend redb serve
```

## Next steps

- [Server API reference](/server/) — all REST endpoints
- [Types reference](/types/) — KnowledgeState, ErukaField, FieldPath
- [HTTP client library](/client/) — use openeruka-client in your Rust app
- [Ecosystem](/ecosystem/) — related projects and managed tier
