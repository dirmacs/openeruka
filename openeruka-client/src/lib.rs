//! # openeruka-client
//!
//! Typed async HTTP client for a hosted [Eruka](https://eruka.dirmacs.com) instance.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use openeruka_client::ErukaClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ErukaClient::new(
//!         "https://eruka.dirmacs.com",
//!         &std::env::var("ERUKA_API_KEY").unwrap(),
//!     );
//!
//!     let field = client
//!         .get_field("my-workspace-id", "identity/company_name")
//!         .await?;
//!
//!     if let Some(f) = field {
//!         println!("Company: {:?}", f.value);
//!     }
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;

pub use client::ErukaClient;
pub use error::ClientError;
