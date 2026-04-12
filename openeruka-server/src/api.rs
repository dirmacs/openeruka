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

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/context", get(get_context).post(write_context))
        .route("/api/v1/entities", get(get_entities))
        .with_state(state)
}

/// GET /health
async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "openeruka",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[derive(Debug, Deserialize)]
struct ContextQuery {
    workspace_id: String,
    path: Option<String>,
}

/// GET /api/v1/context?workspace_id=&path=
async fn get_context(
    State(store): State<AppState>,
    Query(params): Query<ContextQuery>,
) -> Response {
    let path = params.path.as_deref().unwrap_or("*");

    let fields = if path.ends_with('*') || path == "/" {
        store.get_prefix(&params.workspace_id, path)
    } else {
        store.get_field(&params.workspace_id, path).map(|opt| {
            opt.map(|f| vec![f]).unwrap_or_default()
        })
    };

    match fields {
        Ok(fields) => Json(serde_json::json!({ "fields": fields })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

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
                    "field '{}' is {}, cannot overwrite with {}",
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
