//! Storage backends for openeruka.
//!
//! The [`ContextStore`] trait defines the interface every backend must satisfy.
//! The knowledge state invariant (CONFIRMED blocks INFERRED) is enforced by each
//! backend in its `write_field` implementation.
//!
//! Built-in backends:
//! - [`SqliteStore`] — default, bundled SQLite via rusqlite
//! - [`RedbStore`] — embedded key-value store (opt-in with `--features redb`)

use openeruka::{ErukaField, ErukaFieldWrite, ErukaEntity, ErukaEdge, KnowledgeState};

// ─── Common error ────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(String),
    #[error("conflict: field '{path}' is {existing_state}, cannot overwrite with {incoming_state}")]
    KnowledgeStateConflict {
        path: String,
        existing_state: String,
        incoming_state: String,
    },
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not found")]
    NotFound,
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Db(e.to_string())
    }
}

#[cfg(feature = "redb")]
impl From<redb::Error> for StoreError {
    fn from(e: redb::Error) -> Self {
        StoreError::Db(e.to_string())
    }
}

#[cfg(feature = "redb")]
impl From<redb::DatabaseError> for StoreError {
    fn from(e: redb::DatabaseError) -> Self {
        StoreError::Db(e.to_string())
    }
}

#[cfg(feature = "redb")]
impl From<redb::TransactionError> for StoreError {
    fn from(e: redb::TransactionError) -> Self {
        StoreError::Db(e.to_string())
    }
}

#[cfg(feature = "redb")]
impl From<redb::TableError> for StoreError {
    fn from(e: redb::TableError) -> Self {
        StoreError::Db(e.to_string())
    }
}

#[cfg(feature = "redb")]
impl From<redb::StorageError> for StoreError {
    fn from(e: redb::StorageError) -> Self {
        StoreError::Db(e.to_string())
    }
}

#[cfg(feature = "redb")]
impl From<redb::CommitError> for StoreError {
    fn from(e: redb::CommitError) -> Self {
        StoreError::Db(e.to_string())
    }
}

// ─── ContextStore trait ───────────────────────────────────────────────────────

/// The storage interface every openeruka backend must implement.
///
/// All implementations enforce the knowledge state invariant at write time:
/// a `CONFIRMED` field cannot be overwritten by an `INFERRED` write.
pub trait ContextStore: Send + Sync {
    /// Read a single field by workspace + path.
    fn get_field(&self, workspace_id: &str, path: &str) -> Result<Option<ErukaField>, StoreError>;

    /// Read all fields whose path starts with `prefix` (may end with `*` or `/`).
    fn get_prefix(&self, workspace_id: &str, prefix: &str) -> Result<Vec<ErukaField>, StoreError>;

    /// Write a field, enforcing the knowledge state invariant.
    ///
    /// Returns `StoreError::KnowledgeStateConflict` if an existing CONFIRMED
    /// field would be overwritten by a lower knowledge state.
    fn write_field(&self, workspace_id: &str, req: &ErukaFieldWrite) -> Result<ErukaField, StoreError>;

    /// Read all entities for a workspace.
    fn get_entities(&self, workspace_id: &str) -> Result<Vec<ErukaEntity>, StoreError>;

    /// Read all edges for a workspace.
    fn get_edges(&self, workspace_id: &str) -> Result<Vec<ErukaEdge>, StoreError>;
}

// ─── SQLite backend ───────────────────────────────────────────────────────────

use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

/// SQLite-backed store. Thread-safe via `Arc<Mutex<Connection>>`.
#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    /// Open or create a SQLite database at the given path.
    pub fn open(path: &str) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SQLITE_SCHEMA)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    /// In-memory store — useful for testing and ephemeral sessions.
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SQLITE_SCHEMA)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }
}

impl ContextStore for SqliteStore {
    fn get_field(&self, workspace_id: &str, path: &str) -> Result<Option<ErukaField>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, workspace_id, field_path, category, value, knowledge_state, confidence, source_type, created_at, updated_at
             FROM eruka_fields WHERE workspace_id = ?1 AND field_path = ?2 LIMIT 1",
            params![workspace_id, path],
            sqlite_row_to_field,
        );
        match result {
            Ok(f) => Ok(Some(f)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_prefix(&self, workspace_id: &str, prefix: &str) -> Result<Vec<ErukaField>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let clean = prefix.trim_end_matches('*').trim_end_matches('/');
        let pattern = format!("{}%", clean);
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, field_path, category, value, knowledge_state, confidence, source_type, created_at, updated_at
             FROM eruka_fields WHERE workspace_id = ?1 AND field_path LIKE ?2
             ORDER BY field_path",
        )?;
        let rows = stmt.query_map(params![workspace_id, pattern], sqlite_row_to_field)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| StoreError::Db(e.to_string()))
    }

    fn write_field(&self, workspace_id: &str, req: &ErukaFieldWrite) -> Result<ErukaField, StoreError> {
        let conn = self.conn.lock().unwrap();

        let existing: Option<(String, String)> = conn.query_row(
            "SELECT id, knowledge_state FROM eruka_fields WHERE workspace_id = ?1 AND field_path = ?2 LIMIT 1",
            params![workspace_id, &req.path],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;

        let incoming_state = format!("{}", req.knowledge_state);

        if let Some((ref existing_id, ref existing_state_str)) = existing {
            let existing_ks: KnowledgeState = existing_state_str.parse().unwrap_or(KnowledgeState::Unknown);
            if !req.knowledge_state.can_overwrite(&existing_ks) {
                return Err(StoreError::KnowledgeStateConflict {
                    path: req.path.clone(),
                    existing_state: existing_state_str.clone(),
                    incoming_state,
                });
            }
            let value_str = serde_json::to_string(&req.value)?;
            conn.execute(
                "UPDATE eruka_fields SET value = ?1, knowledge_state = ?2, confidence = ?3, source_type = ?4, updated_at = datetime('now')
                 WHERE id = ?5",
                params![value_str, format!("{}", req.knowledge_state), req.confidence, format!("{:?}", req.source), existing_id],
            )?;
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let value_str = serde_json::to_string(&req.value)?;
            let category = req.path.split('/').next().unwrap_or("metadata").to_string();
            conn.execute(
                "INSERT INTO eruka_fields (id, workspace_id, field_path, category, value, knowledge_state, confidence, source_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, workspace_id, &req.path, &category, value_str,
                        format!("{}", req.knowledge_state), req.confidence, format!("{:?}", req.source)],
            )?;
        }

        conn.query_row(
            "SELECT id, workspace_id, field_path, category, value, knowledge_state, confidence, source_type, created_at, updated_at
             FROM eruka_fields WHERE workspace_id = ?1 AND field_path = ?2 LIMIT 1",
            params![workspace_id, &req.path],
            sqlite_row_to_field,
        ).map_err(Into::into)
    }

    fn get_entities(&self, workspace_id: &str) -> Result<Vec<ErukaEntity>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, entity_type, name, category, knowledge_state FROM eruka_entities WHERE workspace_id = ?1",
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| {
            Ok(ErukaEntity {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                entity_type: openeruka::EntityType::Other(row.get::<_, String>(2)?),
                name: row.get(3)?,
                category: row.get(4)?,
                knowledge_state: row.get::<_, String>(5)?.parse().unwrap_or(KnowledgeState::Unknown),
                description: None,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| StoreError::Db(e.to_string()))
    }

    fn get_edges(&self, workspace_id: &str) -> Result<Vec<ErukaEdge>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, source_id, target_id, relation_type, knowledge_state, confidence FROM eruka_edges WHERE workspace_id = ?1",
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| {
            Ok(ErukaEdge {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                source_id: row.get(2)?,
                target_id: row.get(3)?,
                relation_type: row.get(4)?,
                knowledge_state: row.get::<_, String>(5)?.parse().unwrap_or(KnowledgeState::Unknown),
                confidence: row.get(6)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| StoreError::Db(e.to_string()))
    }
}

fn sqlite_row_to_field(row: &rusqlite::Row<'_>) -> rusqlite::Result<ErukaField> {
    let value_str: String = row.get(4)?;
    let value = serde_json::from_str(&value_str).unwrap_or(serde_json::Value::String(value_str));
    let ks_str: String = row.get(5)?;
    Ok(ErukaField {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        field_path: row.get(2)?,
        category: row.get(3)?,
        value,
        knowledge_state: ks_str.parse().unwrap_or(KnowledgeState::Unknown),
        confidence: row.get(6)?,
        source_type: row.get(7)?,
        created_at: None,
        updated_at: None,
    })
}

trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}
impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

const SQLITE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS eruka_fields (
    id             TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL,
    field_path     TEXT NOT NULL,
    category       TEXT NOT NULL DEFAULT 'metadata',
    value          TEXT NOT NULL,
    knowledge_state TEXT NOT NULL DEFAULT 'UNKNOWN',
    confidence     REAL NOT NULL DEFAULT 1.0,
    source_type    TEXT NOT NULL DEFAULT 'user_input',
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (workspace_id, field_path)
);
CREATE TABLE IF NOT EXISTS eruka_entities (
    id             TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL,
    entity_type    TEXT NOT NULL DEFAULT 'concept',
    name           TEXT NOT NULL,
    category       TEXT NOT NULL DEFAULT 'metadata',
    knowledge_state TEXT NOT NULL DEFAULT 'UNKNOWN',
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS eruka_edges (
    id             TEXT PRIMARY KEY,
    workspace_id   TEXT NOT NULL,
    source_id      TEXT NOT NULL,
    target_id      TEXT NOT NULL,
    relation_type  TEXT NOT NULL,
    knowledge_state TEXT NOT NULL DEFAULT 'UNKNOWN',
    confidence     REAL NOT NULL DEFAULT 1.0,
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_fields_ws_path ON eruka_fields (workspace_id, field_path);
CREATE INDEX IF NOT EXISTS idx_entities_ws ON eruka_entities (workspace_id);
CREATE INDEX IF NOT EXISTS idx_edges_ws ON eruka_edges (workspace_id);
"#;

// ─── redb backend ─────────────────────────────────────────────────────────────

#[cfg(feature = "redb")]
mod redb_backend {
    use super::{ContextStore, StoreError};
    use openeruka::{ErukaEdge, ErukaEntity, ErukaField, ErukaFieldWrite, KnowledgeState};
    use redb::{Database, TableDefinition};
    use std::sync::Arc;

    // Table definitions: (workspace_id, path_or_id) -> JSON bytes
    const FIELDS: TableDefinition<(&str, &str), &[u8]> = TableDefinition::new("fields");
    const ENTITIES: TableDefinition<(&str, &str), &[u8]> = TableDefinition::new("entities");
    const EDGES: TableDefinition<(&str, &str), &[u8]> = TableDefinition::new("edges");

    /// redb-backed store. Multiple concurrent readers, single writer (MVCC).
    /// Thread-safe via `Arc<Database>`.
    pub struct RedbStore {
        db: Arc<Database>,
    }

    impl RedbStore {
        pub fn open(path: &str) -> Result<Self, StoreError> {
            let db = Database::create(path)?;
            // Ensure tables exist
            let txn = db.begin_write()?;
            txn.open_table(FIELDS)?;
            txn.open_table(ENTITIES)?;
            txn.open_table(EDGES)?;
            txn.commit()?;
            Ok(Self { db: Arc::new(db) })
        }
    }

    impl ContextStore for RedbStore {
        fn get_field(&self, workspace_id: &str, path: &str) -> Result<Option<ErukaField>, StoreError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(FIELDS)?;
            match table.get((workspace_id, path))? {
                Some(bytes) => {
                    let field: ErukaField = serde_json::from_slice(bytes.value())?;
                    Ok(Some(field))
                }
                None => Ok(None),
            }
        }

        fn get_prefix(&self, workspace_id: &str, prefix: &str) -> Result<Vec<ErukaField>, StoreError> {
            let clean = prefix.trim_end_matches('*').trim_end_matches('/');
            let txn = self.db.begin_read()?;
            let table = txn.open_table(FIELDS)?;

            // Range: (workspace_id, "prefix") ..= (workspace_id, "prefix\u{FFFF}...")
            let start = (workspace_id, clean);
            let end_suffix = format!("{}\u{FFFF}", clean);
            let end = (workspace_id, end_suffix.as_str());

            let mut results = Vec::new();
            for entry in table.range(start..=end)? {
                let (key, bytes) = entry?;
                // Confirm same workspace_id (the range bound ensures it, but double-check)
                if key.value().0 != workspace_id {
                    break;
                }
                let field: ErukaField = serde_json::from_slice(bytes.value())?;
                results.push(field);
            }
            Ok(results)
        }

        fn write_field(&self, workspace_id: &str, req: &ErukaFieldWrite) -> Result<ErukaField, StoreError> {
            let incoming_state = format!("{}", req.knowledge_state);

            // Check existing (read txn)
            let existing_ks_opt: Option<KnowledgeState> = {
                let rtxn = self.db.begin_read()?;
                let table = rtxn.open_table(FIELDS)?;
                match table.get((workspace_id, req.path.as_str()))? {
                    Some(bytes) => {
                        let f: ErukaField = serde_json::from_slice(bytes.value())?;
                        Some(f.knowledge_state)
                    }
                    None => None,
                }
            };

            if let Some(existing_ks) = existing_ks_opt {
                if !req.knowledge_state.can_overwrite(&existing_ks) {
                    return Err(StoreError::KnowledgeStateConflict {
                        path: req.path.clone(),
                        existing_state: format!("{}", existing_ks),
                        incoming_state,
                    });
                }
            }

            // Build the field to store
            let category = req.path.split('/').next().unwrap_or("metadata").to_string();
            let field = ErukaField {
                id: uuid::Uuid::new_v4().to_string(),
                workspace_id: workspace_id.to_string(),
                field_path: req.path.clone(),
                category,
                value: req.value.clone(),
                knowledge_state: req.knowledge_state.clone(),
                confidence: req.confidence,
                source_type: format!("{:?}", req.source),
                created_at: None,
                updated_at: None,
            };

            let bytes = serde_json::to_vec(&field)?;
            let txn = self.db.begin_write()?;
            {
                let mut table = txn.open_table(FIELDS)?;
                table.insert((workspace_id, req.path.as_str()), bytes.as_slice())?;
            }
            txn.commit()?;

            Ok(field)
        }

        fn get_entities(&self, workspace_id: &str) -> Result<Vec<ErukaEntity>, StoreError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENTITIES)?;
            let start = (workspace_id, "");
            let end_suffix = format!("{}\u{FFFF}", workspace_id);
            let end = (workspace_id, end_suffix.as_str());

            let mut results = Vec::new();
            for entry in table.range(start..=end)? {
                let (key, bytes) = entry?;
                if key.value().0 != workspace_id { break; }
                let entity: ErukaEntity = serde_json::from_slice(bytes.value())?;
                results.push(entity);
            }
            Ok(results)
        }

        fn get_edges(&self, workspace_id: &str) -> Result<Vec<ErukaEdge>, StoreError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(EDGES)?;
            let start = (workspace_id, "");
            let end_suffix = format!("{}\u{FFFF}", workspace_id);
            let end = (workspace_id, end_suffix.as_str());

            let mut results = Vec::new();
            for entry in table.range(start..=end)? {
                let (key, bytes) = entry?;
                if key.value().0 != workspace_id { break; }
                let edge: ErukaEdge = serde_json::from_slice(bytes.value())?;
                results.push(edge);
            }
            Ok(results)
        }
    }
}

#[cfg(feature = "redb")]
pub use redb_backend::RedbStore;

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use openeruka::{SourceType, KnowledgeState};

    fn test_write(path: &str, state: KnowledgeState, val: &str) -> ErukaFieldWrite {
        ErukaFieldWrite {
            workspace_id: "test-ws".into(),
            path: path.into(),
            value: serde_json::json!(val),
            knowledge_state: state,
            confidence: 1.0,
            source: SourceType::UserInput,
        }
    }

    /// Run the standard ContextStore test suite against any backend.
    fn run_store_tests(store: &dyn ContextStore) {
        // write + read
        let req = test_write("identity/company_name", KnowledgeState::Confirmed, "DIRMACS");
        store.write_field("ws", &req).unwrap();
        let field = store.get_field("ws", "identity/company_name").unwrap().unwrap();
        assert_eq!(field.value, serde_json::json!("DIRMACS"));
        assert_eq!(field.knowledge_state, KnowledgeState::Confirmed);

        // confirmed blocks inferred
        let result = store.write_field("ws", &ErukaFieldWrite {
            workspace_id: "ws".into(),
            path: "identity/company_name".into(),
            value: serde_json::json!("OVERWRITE"),
            knowledge_state: KnowledgeState::Inferred,
            confidence: 0.8,
            source: SourceType::AgentInference,
        });
        assert!(matches!(result, Err(StoreError::KnowledgeStateConflict { .. })));

        // confirmed can overwrite confirmed
        store.write_field("ws", &test_write("identity/company_name", KnowledgeState::Confirmed, "DIRMACS v2")).unwrap();
        let field2 = store.get_field("ws", "identity/company_name").unwrap().unwrap();
        assert_eq!(field2.value, serde_json::json!("DIRMACS v2"));

        // inferred can overwrite inferred
        store.write_field("ws2", &ErukaFieldWrite {
            workspace_id: "ws2".into(), path: "key".into(),
            value: serde_json::json!("v1"), knowledge_state: KnowledgeState::Inferred,
            confidence: 0.7, source: SourceType::AgentInference,
        }).unwrap();
        store.write_field("ws2", &ErukaFieldWrite {
            workspace_id: "ws2".into(), path: "key".into(),
            value: serde_json::json!("v2"), knowledge_state: KnowledgeState::Inferred,
            confidence: 0.8, source: SourceType::AgentInference,
        }).unwrap();
        let f3 = store.get_field("ws2", "key").unwrap().unwrap();
        assert_eq!(f3.value, serde_json::json!("v2"));

        // prefix query
        store.write_field("ws3", &test_write("products/eruka/name", KnowledgeState::Confirmed, "Eruka")).unwrap();
        store.write_field("ws3", &test_write("products/ares/name", KnowledgeState::Confirmed, "ARES")).unwrap();
        store.write_field("ws3", &test_write("identity/name", KnowledgeState::Confirmed, "DIRMACS")).unwrap();
        let results = store.get_prefix("ws3", "products/*").unwrap();
        assert_eq!(results.len(), 2, "products/* should match 2 fields");
    }

    #[test]
    fn test_sqlite_store() {
        let store = SqliteStore::in_memory().unwrap();
        run_store_tests(&store);
    }
}
