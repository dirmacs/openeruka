//! Core field types — the primary unit of Eruka context memory.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The epistemological state of a piece of knowledge.
///
/// This is the defining abstraction of Eruka: not all facts are equal.
/// A `Confirmed` fact (entered by a human or verified system) cannot be
/// silently overwritten by an `Inferred` guess from an LLM.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KnowledgeState {
    /// Verified fact — human-entered or system-verified.
    Confirmed,
    /// LLM or algorithmic inference — believed but not verified.
    Inferred,
    /// Known to be uncertain — conflicting signals.
    Uncertain,
    /// Gap — this fact is needed but not yet known.
    Unknown,
}

impl KnowledgeState {
    /// Returns true if this state can overwrite `other`.
    ///
    /// The core invariant: `Confirmed` facts are protected from `Inferred` overwrites.
    pub fn can_overwrite(&self, other: &KnowledgeState) -> bool {
        match (self, other) {
            // Confirmed can overwrite anything
            (KnowledgeState::Confirmed, _) => true,
            // Inferred cannot overwrite Confirmed
            (KnowledgeState::Inferred, KnowledgeState::Confirmed) => false,
            // Everything else can overwrite itself or lower states
            _ => true,
        }
    }
}

impl std::fmt::Display for KnowledgeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeState::Confirmed => write!(f, "CONFIRMED"),
            KnowledgeState::Inferred => write!(f, "INFERRED"),
            KnowledgeState::Uncertain => write!(f, "UNCERTAIN"),
            KnowledgeState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl std::str::FromStr for KnowledgeState {
    type Err = crate::error::ErukaError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CONFIRMED" => Ok(KnowledgeState::Confirmed),
            "INFERRED" => Ok(KnowledgeState::Inferred),
            "UNCERTAIN" => Ok(KnowledgeState::Uncertain),
            "UNKNOWN" => Ok(KnowledgeState::Unknown),
            _ => Err(crate::error::ErukaError::InvalidKnowledgeState(s.to_string())),
        }
    }
}

/// Confidence score in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    /// Create a confidence value, clamping to [0.0, 1.0].
    pub fn new(v: f64) -> Self {
        Self(v.clamp(0.0, 1.0))
    }
    /// Confidence of 1.0 — certain.
    pub fn certain() -> Self { Self(1.0) }
    /// Confidence of 0.0 — completely uncertain.
    pub fn none() -> Self { Self(0.0) }
    /// Raw f64 value.
    pub fn value(self) -> f64 { self.0 }
}

impl Default for Confidence {
    fn default() -> Self { Self(1.0) }
}

/// The 7-category schema that organizes Eruka context.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErukaCategory {
    Identity,
    Products,
    Market,
    Operations,
    Content,
    Gaps,
    Metadata,
}

impl std::fmt::Display for ErukaCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ErukaCategory::Identity => "identity",
            ErukaCategory::Products => "products",
            ErukaCategory::Market => "market",
            ErukaCategory::Operations => "operations",
            ErukaCategory::Content => "content",
            ErukaCategory::Gaps => "gaps",
            ErukaCategory::Metadata => "metadata",
        };
        write!(f, "{}", s)
    }
}

/// How this field's value was obtained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    UserInput,
    AgentInference,
    DocumentExtraction,
    WebSearch,
}

/// A validated hierarchical field path (e.g. `identity/company_name`).
///
/// Rules: '/' separator, no spaces, max 256 chars, no leading/trailing '/'.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldPath(String);

impl FieldPath {
    /// Create a validated FieldPath.
    pub fn new(s: &str) -> Result<Self, crate::error::ErukaError> {
        if s.is_empty() {
            return Err(crate::error::ErukaError::InvalidFieldPath("empty path".into()));
        }
        if s.len() > 256 {
            return Err(crate::error::ErukaError::InvalidFieldPath("path exceeds 256 chars".into()));
        }
        if s.contains(' ') {
            return Err(crate::error::ErukaError::InvalidFieldPath("path contains spaces".into()));
        }
        if s.starts_with('/') || s.ends_with('/') {
            return Err(crate::error::ErukaError::InvalidFieldPath("path has leading/trailing '/'".into()));
        }
        Ok(Self(s.to_string()))
    }

    /// Path string reference.
    pub fn as_str(&self) -> &str { &self.0 }

    /// Check if this is a prefix of `other`.
    pub fn is_prefix_of(&self, other: &FieldPath) -> bool {
        other.0.starts_with(&self.0)
    }
}

impl std::fmt::Display for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A context field read from an Eruka instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErukaField {
    pub id: String,
    pub workspace_id: String,
    pub field_path: String,
    pub category: String,
    pub value: serde_json::Value,
    pub knowledge_state: KnowledgeState,
    pub confidence: f64,
    pub source_type: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A write request for creating or updating a context field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErukaFieldWrite {
    pub workspace_id: String,
    pub path: String,
    pub value: serde_json::Value,
    pub knowledge_state: KnowledgeState,
    pub confidence: f64,
    pub source: SourceType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_state_overwrite_rules() {
        assert!(KnowledgeState::Confirmed.can_overwrite(&KnowledgeState::Confirmed));
        assert!(KnowledgeState::Confirmed.can_overwrite(&KnowledgeState::Inferred));
        assert!(!KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Confirmed));
        assert!(KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Inferred));
        assert!(KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Uncertain));
    }

    #[test]
    fn test_confidence_clamping() {
        assert_eq!(Confidence::new(1.5).value(), 1.0);
        assert_eq!(Confidence::new(-0.5).value(), 0.0);
        assert_eq!(Confidence::new(0.8).value(), 0.8);
    }

    #[test]
    fn test_field_path_validation() {
        assert!(FieldPath::new("identity/company_name").is_ok());
        assert!(FieldPath::new("products/eruka/summary").is_ok());
        assert!(FieldPath::new("").is_err());
        assert!(FieldPath::new("has spaces").is_err());
        assert!(FieldPath::new("/leading-slash").is_err());
        assert!(FieldPath::new("trailing-slash/").is_err());
        let long = "a".repeat(257);
        assert!(FieldPath::new(&long).is_err());
    }

    #[test]
    fn test_knowledge_state_roundtrip() {
        for state in [KnowledgeState::Confirmed, KnowledgeState::Inferred,
                      KnowledgeState::Uncertain, KnowledgeState::Unknown] {
            let s = state.to_string();
            let parsed: KnowledgeState = s.parse().unwrap();
            assert_eq!(parsed, state);
        }
    }

    #[test]
    fn test_knowledge_state_serde() {
        let json = serde_json::to_string(&KnowledgeState::Confirmed).unwrap();
        assert_eq!(json, r#""CONFIRMED""#);
        let parsed: KnowledgeState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, KnowledgeState::Confirmed);
    }
}
