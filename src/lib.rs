//! gho - GitHub operator CLI for multi-account workflows.

pub mod commands;
pub mod config;
pub mod error;
pub mod github;
pub mod keychain;
pub mod models;
pub mod storage;

pub use commands::{account, pr, repo};
pub use config::Config;
pub use error::AppError;
pub use models::{Account, AccountKind, AccountsFile, Protocol, Repository};
pub use storage::{FilesystemStorage, Storage};
