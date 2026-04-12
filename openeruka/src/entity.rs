//! Entity types for the Eruka knowledge graph.

use serde::{Deserialize, Serialize};
use crate::field::KnowledgeState;

/// The kind of real-world thing an entity represents.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Person,
    Company,
    Product,
    Concept,
    Event,
    MarketSegment,
    Other(String),
}

/// A node in the Eruka knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErukaEntity {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub category: String,
    pub knowledge_state: KnowledgeState,
    pub description: Option<String>,
}
