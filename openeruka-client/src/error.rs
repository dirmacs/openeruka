//! Client error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("unauthorized — check your API key")]
    Unauthorized,

    #[error("conflict — write rejected by knowledge state invariant (409)")]
    Conflict,

    #[error("not found")]
    NotFound,

    #[error("server error: {status} — {body}")]
    Server { status: u16, body: String },

    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
}
