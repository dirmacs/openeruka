+++
title = "Rust Client Library"
description = "openeruka-client — typed async HTTP client for the Eruka API"
+++

`openeruka-client` is a fully async, typed Rust client for any Eruka-compatible REST API — local openeruka or managed eruka.dirmacs.com.

## Add to your project

```toml
[dependencies]
openeruka-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Connect to local openeruka

```rust
use openeruka_client::ErukaClient;

let client = ErukaClient::new("http://localhost:8080", "local");
```

## Connect to managed eruka.dirmacs.com

```rust
let client = ErukaClient::new("https://eruka.dirmacs.com", "eruka_sk_...");
```

## Writing context

```rust
use openeruka_client::ErukaClient;
use openeruka::KnowledgeState;

let client = ErukaClient::new("http://localhost:8080", "local");

// Write a confirmed fact (JSON string value)
let field = client.write_context(
    "identity/company_name",   // path
    "ACME Corp",               // value (auto-encoded as JSON string)
    "user_input",              // source
    1.0,                       // confidence
).await?;

println!("{} [{}]", field["field"]["path"], field["field"]["knowledge_state"]);
```

## Reading context

```rust
// Read a specific field
let result = client.get_context("identity/company_name", false).await?;
let fields = result["fields"].as_array().unwrap();

// Read all fields under a prefix
let result = client.get_context("identity", true).await?;

// Read everything
let result = client.get_context("*", true).await?;
```

## Searching context

```rust
// Keyword search across all fields
let result = client.search_context("company name", "*", 10).await?;
let hits = result["results"].as_array().unwrap();

for hit in hits {
    println!("{} = {}", hit["field_path"], hit["value"]);
}
```

## Completeness scoring

```rust
let report = client.get_completeness("*").await?;
let score = report["score"].as_f64().unwrap_or(0.0);
println!("Context completeness: {:.0}%", score * 100.0);
```

## Gap detection

```rust
let gaps = client.get_gaps(
    None,           // filter by category (None = all)
    None,           // filter by status (None = all)
    "impact_score", // sort by
    20,             // limit
).await?;

for gap in gaps["gaps"].as_array().unwrap() {
    println!("Missing: {}", gap["field_path"]);
}
```

## Health check

```rust
match client.health().await {
    Ok(true)  => println!("Connected"),
    Ok(false) => eprintln!("API unhealthy"),
    Err(e)    => eprintln!("Connection failed: {}", e),
}
```

## Full API reference

See [docs.rs/openeruka-client](https://docs.rs/openeruka-client) for the complete API.

## Error handling

All methods return `Result<serde_json::Value, ClientError>`. Errors include:

- `ClientError::Http(reqwest::Error)` — network or HTTP error
- `ClientError::Api { status, message }` — API-level error (e.g. `409 KnowledgeStateConflict`)

```rust
use openeruka_client::ClientError;

match client.write_context("identity/name", "New Name", "user_input", 0.5).await {
    Ok(field) => println!("Wrote: {:?}", field),
    Err(ClientError::Api { status: 409, message }) => {
        eprintln!("State conflict: {}", message);
        // The existing field has a higher-trust state
    }
    Err(e) => return Err(e.into()),
}
```
