//! # openeruka
//!
//! Open core types and traits for the [Eruka](https://eruka.dirmacs.com) context memory system.
//!
//! Eruka is a structured, knowledge-state-aware memory layer for AI agents.
//! This crate provides the vocabulary every consumer needs: types for fields,
//! entities, edges, and the epistemological states that make Eruka different.
//!
//! ## Quick start
//!
//! ```rust
//! use openeruka::{KnowledgeState, FieldPath, Confidence};
//!
//! let path = FieldPath::new("identity/company_name").unwrap();
//! assert_eq!(path.as_str(), "identity/company_name");
//!
//! // The core invariant: Confirmed facts resist Inferred overwrites
//! assert!( KnowledgeState::Confirmed.can_overwrite(&KnowledgeState::Inferred));
//! assert!(!KnowledgeState::Inferred.can_overwrite(&KnowledgeState::Confirmed));
//!
//! let c = Confidence::new(0.9);
//! assert_eq!(c.value(), 0.9);
//! ```
//!
//! ## Related
//!
//! - [`openeruka-client`](https://crates.io/crates/openeruka-client) — typed HTTP client for a hosted Eruka instance
//! - [eruka-mcp](https://github.com/dirmacs/eruka-mcp) — MCP server for Claude Desktop / Claude Code
//! - [Eruka hosted API](https://eruka.dirmacs.com)
//! - [DIRMACS](https://dirmacs.com)

pub mod field;
pub mod entity;
pub mod edge;
pub mod error;

#[cfg(feature = "test-helpers")]
pub mod test_helpers;

pub use field::{ErukaField, ErukaFieldWrite, FieldPath, KnowledgeState, Confidence, SourceType, ErukaCategory};
pub use entity::{ErukaEntity, EntityType};
pub use edge::{ErukaEdge, RelationType};
pub use error::ErukaError;
