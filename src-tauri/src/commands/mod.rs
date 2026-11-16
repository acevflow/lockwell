//! Tauri commands: the only entry points the interface can call.
//!
//! Commands stay thin: they validate and convert their arguments and
//! delegate to the vault.

pub mod vault;
