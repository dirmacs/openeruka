//! Error types for openeruka.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErukaError {
    #[error("invalid field path: {0}")]
    InvalidFieldPath(String),

    #[error("invalid knowledge state: {0}")]
    InvalidKnowledgeState(String),

    #[error("invalid confidence value: {0}")]
    InvalidConfidence(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
