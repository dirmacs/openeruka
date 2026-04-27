//! Axum REST API handlers — compatible surface with eruka.dirmacs.com.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::store::{ContextStore, StoreError};
use openeruka::{ErukaFieldWrite, KnowledgeState, SourceType};

pub type AppState = Arc<dyn ContextStore>;

// ─── Bare category detection (BUG-1 fix) ─────────────────────────────────────
// A bare category path like 'identity' should route to prefix scan, not exact match.
// Previously: path='identity' → get_field (exact match) → returns empty
// Fixed: path='identity' → get_prefix with pattern → returns identity/* fields
fn is_bare_category(path: &str) -> bool {
    matches!(
        path,
        CATEGORY_IDENTITY
        | CATEGORY_PRODUCTS
        | CATEGORY_MARKET
        | CATEGORY_OPERATIONS
        | CATEGORY_CONTENT
        | CATEGORY_GAPS
        | CATEGORY_METADATA
    )
}

// Constants matching the 7-schema categories from openeruka field.rs ErukaCategory
const CATEGORY_IDENTITY: &str = "identity";
const CATEGORY_PRODUCTS: &str = "products";
const CATEGORY_MARKET: &str = "market";
const CATEGORY_OPERATIONS: &str = "operations";
const CATEGORY_CONTENT: &str = "content";
const CATEGORY_GAPS: &str = "gaps";
const CATEGORY_METADATA: &str = "metadata";

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/context", get(get_context).post(write_context))
        .route("/api/v1/entities", get(get_entities))
        .route("/api/v1/edges", get(get_edges))
        .route("/api/v1/tier", get(get_tier))
        .route("/api/v1/search", get(search_context))
        .route("/api/v1/gaps", get(get_gaps))
        .with_state(state)
}

// ─── GET /health ──────────────────────────────────────────────────────────────

async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "openeruka",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ─── GET /api/v1/context ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ContextQuery {
    workspace_id: String,
    path: Option<String>,
}

/// GET /api/v1/context?workspace_id=&path=
/// BUG-1 fix: bare category paths (identity, products, etc.) now route to prefix
/// scan instead of exact match, so 'path=identity' returns identity/* fields.
async fn get_context(
    State(store): State<AppState>,
    Query(params): Query<ContextQuery>,
) -> Response {
    let path = params.path.as_deref().unwrap_or("*");

    let fields = if path.ends_with('*') || path == "/" || is_bare_category(path) {
        // Route to prefix scan
        store.get_prefix(&params.workspace_id, path)
    } else {
        // Exact match
        store.get_field(&params.workspace_id, path).map(|opt| {
            opt.map(|f| vec![f]).unwrap_or_default()
        })
    };

    match fields {
        Ok(fields) => Json(serde_json::json!({ "fields": fields })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── POST /api/v1/context ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WriteContextRequest {
    workspace_id: String,
    path: String,
    value: Value,
    knowledge_state: Option<String>,
    confidence: Option<f64>,
    source: Option<String>,
}

/// POST /api/v1/context
async fn write_context(
    State(store): State<AppState>,
    Json(req): Json<WriteContextRequest>,
) -> Response {
    let ks = req.knowledge_state
        .as_deref()
        .and_then(|s| s.parse::<KnowledgeState>().ok())
        .unwrap_or(KnowledgeState::Inferred);

    let source = match req.source.as_deref() {
        Some("user_input") => SourceType::UserInput,
        Some("document_extraction") => SourceType::DocumentExtraction,
        Some("web_search") => SourceType::WebSearch,
        _ => SourceType::AgentInference,
    };

    let write_req = ErukaFieldWrite {
        workspace_id: req.workspace_id.clone(),
        path: req.path,
        value: req.value,
        knowledge_state: ks,
        confidence: req.confidence.unwrap_or(1.0),
        source,
    };

    match store.write_field(&req.workspace_id, &write_req) {
        Ok(field) => Json(serde_json::json!({ "field": field, "status": "ok" })).into_response(),
        Err(StoreError::KnowledgeStateConflict { path, existing_state, incoming_state }) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "knowledge_state_conflict",
                "message": format!(
                    "field {} is {}, cannot overwrite with {}",
                    path, existing_state, incoming_state
                ),
                "path": path,
                "existing_state": existing_state,
                "incoming_state": incoming_state,
            })),
        ).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── GET /api/v1/entities ─────────────────────────────────────────────────────

/// GET /api/v1/entities?workspace_id=
async fn get_entities(
    State(store): State<AppState>,
    Query(params): Query<ContextQuery>,
) -> Response {
    match store.get_entities(&params.workspace_id) {
        Ok(entities) => Json(serde_json::json!({ "entities": entities })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── GET /api/v1/edges ────────────────────────────────────────────────────────

/// GET /api/v1/edges?workspace_id=
async fn get_edges(
    State(store): State<AppState>,
    Query(params): Query<ContextQuery>,
) -> Response {
    match store.get_edges(&params.workspace_id) {
        Ok(edges) => Json(serde_json::json!({ "edges": edges })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ─── GET /api/v1/tier ─────────────────────────────────────────────────────────

/// GET /api/v1/tier
/// Returns workspace tier for eruka-mcp compatibility.
/// openeruka (local, single-tenant) always returns tier: "local".
async fn get_tier() -> Json<Value> {
    Json(serde_json::json!({
        "tier": "local",
        "note": "openeruka local mode — no multi-tenant billing",
    }))
}

// ─── GET /api/v1/search ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SearchQuery {
    workspace_id: String,
    query: String,
    limit: Option<usize>,
}

/// GET /api/v1/search?workspace_id=&query=&limit=
/// Simple text-match search over all workspace fields.
/// No vector similarity (local mode — managed eruka has pgvector for semantic search).
async fn search_context(
    State(store): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Response {
    let fields = match store.get_prefix(&params.workspace_id, "*") {
        Ok(f) => f,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let query_lower = params.query.to_lowercase();
    let limit = params.limit.unwrap_or(10);

    let matches: Vec<_> = fields.iter()
        .filter(|f| {
            let path_match = f.field_path.to_lowercase().contains(&query_lower);
            let value_str = serde_json::to_string(&f.value).unwrap_or_default();
            let value_match = value_str.to_lowercase().contains(&query_lower);
            path_match || value_match
        })
        .take(limit)
        .cloned()
        .collect();

    Json(serde_json::json!({
        "fields": matches,
        "query": params.query,
        "count": matches.len(),
        "mode": "text_match",
        "note": "no vector similarity — use managed eruka for semantic search",
    })).into_response()
}

// ─── GET /api/v1/gaps ─────────────────────────────────────────────────────────

/// GET /api/v1/gaps?workspace_id=
/// Gap tracking is not implemented in openeruka local mode.
/// Returns an empty list with a note.
async fn get_gaps(
    State(_store): State<AppState>,
    Query(params): Query<ContextQuery>,
) -> Json<Value> {
    Json(serde_json::json!({
        "gaps": [],
        "workspace_id": params.workspace_id,
        "note": "gap tracking not available in openeruka local mode — use managed eruka at eruka.dirmacs.com",
    }))
}
