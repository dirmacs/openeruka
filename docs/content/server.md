+++
title = "Server API Reference"
description = "REST API endpoints for the openeruka server binary"
+++

The `openeruka-server` crate provides the `openeruka` binary with a REST API compatible with the [eruka.dirmacs.com](https://eruka.dirmacs.com) managed service surface.

## Run the server

```bash
# Default: SQLite at ./eruka.db, port 8080
openeruka serve

# Custom port and database path
openeruka --db /var/lib/openeruka/ctx.db serve --port 9090

# redb backend (build with --features redb)
openeruka --backend redb serve
```

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENERUKA_BACKEND` | `sqlite` or `redb` | `sqlite` |
| `OPENERUKA_DB` | Database file path | `./eruka.db` |
| `RUST_LOG` | Log level | `openeruka=info` |

---

## Endpoints

All endpoints accept and return `application/json`.

### `GET /health`

Health check.

```bash
curl http://localhost:8080/health
# {"status":"ok","version":"0.1.0"}
```

---

### `POST /fields`

Write a context field. Enforces the knowledge state invariant — an `inferred` write to a `confirmed` field is rejected with `409 Conflict`.

**Request body:**

```json
{
  "workspace_id": "default",
  "path": "identity/company_name",
  "value": "ACME Corp",
  "knowledge_state": "confirmed",
  "confidence": 1.0,
  "source": "user_input"
}
```

**Fields:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `workspace_id` | string | yes | Namespaces fields — use any string |
| `path` | string | yes | Slash-separated, e.g. `identity/name` |
| `value` | any JSON | yes | String, number, object, array |
| `knowledge_state` | string | yes | `confirmed` / `inferred` / `uncertain` / `unknown` |
| `confidence` | float | yes | 0.0 – 1.0 |
| `source` | string | yes | `user_input` / `agent_inference` / `api_ingestion` / `system_default` |

**Response:** `200 OK` with the stored `ErukaField`.

**Error:** `409 Conflict` when the incoming state cannot overwrite the existing one.

---

### `GET /fields/:workspace_id/:path`

Read a single field by exact path.

```bash
curl http://localhost:8080/fields/default/identity/company_name
```

Response:

```json
{
  "id": "...",
  "workspace_id": "default",
  "path": "identity/company_name",
  "value": "ACME Corp",
  "knowledge_state": "confirmed",
  "confidence": 1.0,
  "source": "user_input",
  "created_at": "...",
  "updated_at": "..."
}
```

Returns `404` if not found.

---

### `GET /fields/:workspace_id`

List all fields for a workspace. Optionally filter by prefix.

```bash
# All fields
curl http://localhost:8080/fields/default

# Fields under identity/
curl "http://localhost:8080/fields/default?prefix=identity"
```

Response:

```json
{ "fields": [ { ... }, { ... } ] }
```

---

### `GET /entities/:workspace_id`

List all entity nodes in the knowledge graph for a workspace.

```bash
curl http://localhost:8080/entities/default
```

---

### `GET /edges/:workspace_id`

List all typed edges in the knowledge graph.

```bash
curl http://localhost:8080/edges/default
```

---

## CLI subcommands

The `openeruka` binary also exposes a CLI for scripting:

```bash
# Read a field
openeruka get <workspace_id> <path>

# Write a field
openeruka set <workspace_id> <path> <value_json> [--state confirmed|inferred|uncertain|unknown]
```

Examples:

```bash
# Write a confirmed fact
openeruka set default identity/company '"ACME Corp"' --state confirmed

# Read it back
openeruka get default identity/company
# "ACME Corp"
#   state: confirmed  confidence: 100%

# Write a JSON object value
openeruka set default market/tam '{"amount":4200000000,"currency":"USD"}' --state inferred
```
