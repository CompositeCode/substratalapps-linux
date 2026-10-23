//! Platform-independent core of the Substratal Hub for Linux.
//!
//! Everything here is shared by the GTK app (`substratal-hub`) and the CLI
//! (`substratalctl`), and nothing here depends on GTK. The layering follows
//! the specs' CLIENT-ARCHITECTURE §1: screens never call HTTP directly; they
//! go through [`api::ApiClient`], which owns versioning, auth, idempotency,
//! concurrency and error decoding.

pub mod api;
pub mod capabilities;
pub mod config;
pub mod error;
pub mod models;
pub mod nav;
pub mod screens;
pub mod secrets;
pub mod session;
pub mod shortcuts;
pub mod version;

pub use api::ApiClient;
pub use config::Environment;
pub use error::{ApiError, Error, Result};
