//! Edge types for the Eruka knowledge graph.

use serde::{Deserialize, Serialize};
use crate::field::KnowledgeState;

/// The semantic relationship type between two entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    FoundedBy,
    Employs,
    Offers,
    UsesTechnology,
    IntegratesWith,
    Consumes,
    PartnersWith,
    Evaluates,
    /// Custom relationship type not in the standard vocabulary.
    Custom(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::FoundedBy => write!(f, "FOUNDED_BY"),
            RelationType::Employs => write!(f, "EMPLOYS"),
            RelationType::Offers => write!(f, "OFFERS"),
            RelationType::UsesTechnology => write!(f, "USES_TECHNOLOGY"),
            RelationType::IntegratesWith => write!(f, "INTEGRATES_WITH"),
            RelationType::Consumes => write!(f, "CONSUMES"),
            RelationType::PartnersWith => write!(f, "PARTNERS_WITH"),
            RelationType::Evaluates => write!(f, "EVALUATES"),
            RelationType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// A directed edge in the Eruka knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErukaEdge {
    pub id: String,
    pub workspace_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub knowledge_state: KnowledgeState,
    pub confidence: f64,
}
