# openeruka — CLAUDE.md

## Stack

- **Language:** Rust stable
- **HTTP:** reqwest with rustls (no OpenSSL dependency)
- **Serialization:** serde + serde_json everywhere
- **Error handling:** thiserror crates, no panics in library code
- **Async runtime:** tokio (for openeruka-client)
- **No unsafe code**

## What this crate IS

Types and HTTP client for consumers of a hosted Eruka instance. Think of it as the "SDK" for Eruka.

## What this crate IS NOT

The Eruka engine (database layer, quality scoring, knowledge state enforcement, gardener, ingest). Those are proprietary and live in dirmacs/eruka (private).

## Key invariant (never relax this)

`KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Confirmed)` must return `false`. This is the core epistemological model. If you see code that returns `true` here, that is a bug.

## Publishing

```bash
# From workspace root
cargo publish -p openeruka
cargo publish -p openeruka-client  # after openeruka is indexed
```

Set `CARGO_REGISTRY_TOKEN` in the CI environment.

## Test discipline

- Unit tests: `cargo test --workspace`
- Integration tests against a live Eruka instance: set `ERUKA_TEST_URL` + `ERUKA_TEST_KEY` env vars
- If env vars not set, integration tests are skipped (not failed)
- Every PR must keep `cargo test --workspace` green

## Never add

- Database dependencies (sqlx, diesel, postgres) — openeruka has no DB access
- The proprietary scoring engine logic (quality.rs patterns)
- TOON files or other proprietary ARES/Eruka config
- Any code that duplicates the knowledge state enforcement logic (that's the server's job)
