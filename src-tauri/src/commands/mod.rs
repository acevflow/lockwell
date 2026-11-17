//! Tauri commands: the only entry points the interface can call.
//!
//! Commands stay thin: they validate and convert their arguments and
//! delegate to the vault.

pub mod backup;
mod dialog;
pub mod entries;
pub mod files;
pub mod security;
pub mod vault;
