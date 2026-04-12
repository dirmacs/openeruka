//! openeruka — standalone knowledge state memory server.
//!
//! Subcommands:
//!   serve  — start the REST API server (eruka.dirmacs.com-compatible surface)
//!   get    — read a field from the local database
//!   set    — write a field to the local database
//!
//! Backends (select with --backend):
//!   sqlite — default, bundled SQLite (./eruka.db)
//!   redb   — embedded key-value store (./eruka.redb) [requires --features redb]
//!
//! Connect eruka-mcp to this server:
//!   ERUKA_API_URL=http://localhost:8080 eruka-mcp

use clap::{Parser, Subcommand, ValueEnum};
use std::sync::Arc;

use openeruka::{
    api::create_router,
    store::{ContextStore, SqliteStore},
};

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    /// Bundled SQLite (default)
    Sqlite,
    /// Embedded redb key-value store
    #[cfg(feature = "redb")]
    Redb,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Sqlite => write!(f, "sqlite"),
            #[cfg(feature = "redb")]
            Backend::Redb => write!(f, "redb"),
        }
    }
}

#[derive(Parser)]
#[command(name = "openeruka")]
#[command(version, about = "Knowledge state memory server — REST + MCP + CLI")]
struct Cli {
    /// Storage backend to use
    #[arg(long, env = "OPENERUKA_BACKEND", default_value = "sqlite")]
    backend: Backend,

    /// Database path (SQLite: ./eruka.db, redb: ./eruka.redb)
    #[arg(long, env = "OPENERUKA_DB")]
    db: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the REST API server
    Serve {
        /// Port to listen on
        #[arg(long, short = 'p', default_value = "8080")]
        port: u16,
    },
    /// Read a field value
    Get {
        workspace_id: String,
        path: String,
    },
    /// Write a field value
    Set {
        workspace_id: String,
        path: String,
        value: String,
        /// Knowledge state: confirmed, inferred, uncertain, unknown
        #[arg(long, default_value = "inferred")]
        state: String,
    },
}

fn open_store(backend: &Backend, db_path: Option<&str>) -> anyhow::Result<Arc<dyn ContextStore>> {
    match backend {
        Backend::Sqlite => {
            let path = db_path.unwrap_or("./eruka.db");
            tracing::info!("backend: sqlite  db: {}", path);
            Ok(Arc::new(SqliteStore::open(path)?))
        }
        #[cfg(feature = "redb")]
        Backend::Redb => {
            use openeruka_server::store::RedbStore;
            let path = db_path.unwrap_or("./eruka.redb");
            tracing::info!("backend: redb  db: {}", path);
            Ok(Arc::new(RedbStore::open(path)?))
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "openeruka=info".into()))
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let store = open_store(&cli.backend, cli.db.as_deref())?;

    match cli.command {
        Commands::Serve { port } => {
            let addr = format!("0.0.0.0:{}", port);
            tracing::info!("openeruka {} listening on http://{}", env!("CARGO_PKG_VERSION"), addr);
            tracing::info!("Connect eruka-mcp: ERUKA_API_URL=http://localhost:{}", port);

            let app = create_router(store);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }
        Commands::Get { workspace_id, path } => {
            match store.get_field(&workspace_id, &path)? {
                Some(field) => {
                    println!("{}", serde_json::to_string_pretty(&field.value)?);
                    eprintln!("  state: {}  confidence: {:.0}%",
                        field.knowledge_state, field.confidence * 100.0);
                }
                None => eprintln!("(not found: {}/{})", workspace_id, path),
            }
        }
        Commands::Set { workspace_id, path, value, state } => {
            use openeruka::{ErukaFieldWrite, KnowledgeState, SourceType};
            let ks = state.parse::<KnowledgeState>().unwrap_or(KnowledgeState::Inferred);
            let val: serde_json::Value = serde_json::from_str(&value)
                .unwrap_or(serde_json::Value::String(value));
            let req = ErukaFieldWrite {
                workspace_id: workspace_id.clone(),
                path: path.clone(),
                value: val,
                knowledge_state: ks,
                confidence: 1.0,
                source: SourceType::UserInput,
            };
            match store.write_field(&workspace_id, &req) {
                Ok(field) => eprintln!("✓ {}/{} = {} ({})",
                    workspace_id, path, field.value, field.knowledge_state),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
