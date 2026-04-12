+++
title = "Ecosystem & Backlinks"
description = "How openeruka fits into the DIRMACS ecosystem and related projects"
+++

openeruka is the open-source foundation of the Eruka knowledge memory stack. Here's how everything connects.

## The stack

```
┌─────────────────────────────────────────────────────┐
│                  AI clients                          │
│  Claude Desktop · Claude Code · Cursor · VS Code    │
└────────────────────┬────────────────────────────────┘
                     │ MCP (JSON-RPC / SSE)
┌────────────────────▼────────────────────────────────┐
│                  eruka-mcp                           │
│  MCP server — bridges AI tools to Eruka backends    │
│  github.com/dirmacs/eruka-mcp · crates.io           │
└──────────┬──────────────────────┬───────────────────┘
           │ HTTP (local)         │ HTTP (managed)
┌──────────▼──────────┐  ┌───────▼───────────────────┐
│   openeruka         │  │   eruka.dirmacs.com        │
│   (this project)    │  │   Managed tier             │
│   SQLite / redb     │  │   PostgreSQL + pgvector    │
│   No account needed │  │   Quality scoring (B6)     │
│   github.com/       │  │   Knowledge decay          │
│   dirmacs/openeruka │  │   Multi-tenant auth        │
└─────────────────────┘  └────────────────────────────┘
```

## DIRMACS projects

### eruka-mcp
**[github.com/dirmacs/eruka-mcp](https://github.com/dirmacs/eruka-mcp)** · [docs](https://dirmacs.github.io/eruka-mcp) · [crates.io](https://crates.io/crates/eruka-mcp)

MCP server that bridges any MCP-compatible AI tool to openeruka or eruka.dirmacs.com. Provides 17 tools including context read/write, gap detection, constraint injection, and graph traversal.

Default URL is `http://localhost:8080` — connects to openeruka with zero configuration.

### eruka.dirmacs.com
**[eruka.dirmacs.com](https://eruka.dirmacs.com)**

The managed Eruka service. Same REST API surface as openeruka, plus:
- 6-layer quality scoring pipeline (B6)
- Knowledge decay (confidence degrades over time)
- Multi-tenant isolation
- PostgreSQL + pgvector backend
- OAuth and service key auth

Use this when you need enterprise-grade memory without managing infrastructure.

### ARES
**[github.com/dirmacs/ares](https://github.com/dirmacs/ares)** · [crates.io](https://crates.io/crates/ares-server)

Multi-agent runtime with LLM routing, RAG, tool calling, MCP, and workflow orchestration. ARES agents can read from and write to openeruka to maintain persistent grounded memory across sessions.

### deagle
**[github.com/dirmacs/deagle](https://github.com/dirmacs/deagle)** · [crates.io](https://crates.io/crates/deagle)

Code intelligence CLI built on tree-sitter. AST-based search, symbol graph, and codebase statistics for any language. Used internally to navigate and audit the Eruka codebase.

### pawan
**[github.com/dirmacs/pawan](https://github.com/dirmacs/pawan)**

CLI coding agent with 29 tools, LSP integration, and tiered model registry. Uses openeruka via eruka-mcp for cross-session memory — it reads project context on startup so it doesn't repeat itself.

### dstack
**[github.com/dirmacs/dstack](https://github.com/dirmacs/dstack)** · [crates.io](https://crates.io/crates/dstack)

Dev stack tooling: project scaffolding, multi-platform plugin system, CI audit gates, and swarm harness for parallel agent workloads.

## Crates

| Crate | Description | crates.io |
|-------|-------------|-----------|
| `openeruka` | Core types — KnowledgeState, ErukaField, FieldPath | [link](https://crates.io/crates/openeruka) |
| `openeruka-client` | Typed async HTTP client | [link](https://crates.io/crates/openeruka-client) |
| `openeruka-server` | Server binary (SQLite + redb backends) | [link](https://crates.io/crates/openeruka-server) |
| `eruka-mcp` | MCP server for Eruka | [link](https://crates.io/crates/eruka-mcp) |
| `ares-server` | Multi-agent runtime | [link](https://crates.io/crates/ares-server) |
| `deagle` | Code intelligence CLI | [link](https://crates.io/crates/deagle) |
| `dstack` | Dev stack tooling | [link](https://crates.io/crates/dstack) |

## Linking back to openeruka

If you build on top of openeruka, we'd appreciate a link back:

- README badge: `[![openeruka](https://img.shields.io/badge/memory-openeruka-6c8ebf)](https://dirmacs.github.io/openeruka)`
- Docs mention: "Uses [openeruka](https://github.com/dirmacs/openeruka) for grounded context memory"

## Company

openeruka is built and maintained by [DIRMACS](https://dirmacs.com), a Hyderabad-based AI infrastructure company. The managed tier at [eruka.dirmacs.com](https://eruka.dirmacs.com) funds continued development of the open-source core.
