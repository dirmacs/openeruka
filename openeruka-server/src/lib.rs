//! openeruka-server — SQLite-backed knowledge state memory server.
//!
//! Exposes REST API compatible with eruka.dirmacs.com.
//! Connect eruka-mcp to this server for local development,
//! or to eruka.dirmacs.com for production.

pub mod api;
pub mod store;

pub use store::SqliteStore;
