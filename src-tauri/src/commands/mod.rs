//! Tauri commands: the only entry points the interface can call.
//!
//! Commands stay thin: they validate and convert their arguments and
//! delegate to the vault.
//!
//! Most commands return [`crate::error::AppResult`]; a failure reaches the
//! interface as `{ code, message }`. They run through
//! [`crate::state::AppState::with_vault`], which counts as user activity.
//! Native dialogs are shown by the core, so the interface never sees file
//! system paths.

pub mod backup;
mod dialog;
pub mod entries;
pub mod files;
pub mod security;
pub mod vault;
