+++
title = "Types Reference"
description = "Core types in the openeruka crate — KnowledgeState, ErukaField, FieldPath"
+++

The `openeruka` crate provides the shared type definitions used by the server, client, and any Rust application working with knowledge state memory.

Add it to your project:

```toml
[dependencies]
openeruka = "0.1"
```

## KnowledgeState

The core invariant: **Confirmed facts cannot be overwritten by Inferred guesses.**

```rust
pub enum KnowledgeState {
    Confirmed,   // Verified by authoritative source — highest trust
    Inferred,    // Derived from other facts — medium trust
    Uncertain,   // Known to be questionable
    Unknown,     // No information available
}
```

The hierarchy: `Confirmed > Inferred > Uncertain > Unknown`

### `can_overwrite`

```rust
impl KnowledgeState {
    pub fn can_overwrite(&self, existing: &KnowledgeState) -> bool
}
```

Returns `true` if the incoming state is allowed to replace the existing one. A `Confirmed` write can always overwrite. An `Inferred` write cannot overwrite a `Confirmed` field — the write is rejected with `StoreError::KnowledgeStateConflict`.

| Incoming \ Existing | Confirmed | Inferred | Uncertain | Unknown |
|---------------------|-----------|----------|-----------|---------|
| **Confirmed**       | ✓         | ✓        | ✓         | ✓       |
| **Inferred**        | ✗         | ✓        | ✓         | ✓       |
| **Uncertain**       | ✗         | ✗        | ✓         | ✓       |
| **Unknown**         | ✗         | ✗        | ✗         | ✓       |

## ErukaField

A stored context field — the fundamental unit of knowledge:

```rust
pub struct ErukaField {
    pub id: Uuid,
    pub workspace_id: String,
    pub path: String,           // e.g., "identity/company_name"
    pub value: serde_json::Value,
    pub knowledge_state: KnowledgeState,
    pub confidence: f64,        // 0.0 – 1.0
    pub source: SourceType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## ErukaFieldWrite

The request type for writing a field:

```rust
pub struct ErukaFieldWrite {
    pub workspace_id: String,
    pub path: String,
    pub value: serde_json::Value,
    pub knowledge_state: KnowledgeState,
    pub confidence: f64,
    pub source: SourceType,
}
```

## SourceType

Where a piece of knowledge came from:

```rust
pub enum SourceType {
    UserInput,       // Directly provided by a user
    AgentInference,  // Derived by an AI agent
    ApiIngestion,    // Imported from an external system
    SystemDefault,   // Built-in default value
}
```

## FieldPath

Helper for working with dot-separated or slash-separated paths:

```rust
pub struct FieldPath(pub String);

impl FieldPath {
    pub fn category(&self) -> &str   // "identity" from "identity/company_name"
    pub fn key(&self) -> &str        // "company_name" from "identity/company_name"
    pub fn is_wildcard(&self) -> bool // true for "*"
}
```

## Graph types

```rust
pub struct ErukaEntity {
    pub id: Uuid,
    pub workspace_id: String,
    pub entity_type: EntityType,  // Person, Company, Product, Concept, Event
    pub name: String,
    pub properties: serde_json::Value,
}

pub struct ErukaEdge {
    pub id: Uuid,
    pub workspace_id: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub relation: String,
    pub weight: f64,
}
```
