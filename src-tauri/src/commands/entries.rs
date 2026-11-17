//! Commands to list, read, create, update and delete password entries.
//!
//! All of them need an unlocked vault and fail with `locked` otherwise. Only
//! `get_entry` returns the password and the notes to the interface.

use tauri::State;

use crate::error::AppResult;
use crate::state::AppState;
use crate::vault::entries::{Entry, EntryInput, EntrySummary};

/// Lists the entries without passwords or notes.
#[tauri::command]
pub fn list_entries(state: State<'_, AppState>) -> AppResult<Vec<EntrySummary>> {
    state.with_vault(|vault| vault.unlocked()?.list_entries())
}

/// Returns one complete entry, including its password.
#[tauri::command]
pub fn get_entry(state: State<'_, AppState>, id: String) -> AppResult<Entry> {
    state.with_vault(|vault| vault.unlocked()?.get_entry(&id))
}

/// Creates an entry and returns its id.
#[tauri::command]
pub fn create_entry(state: State<'_, AppState>, entry: EntryInput) -> AppResult<String> {
    state.with_vault(|vault| vault.unlocked()?.create_entry(&entry))
}

/// Replaces the fields of an entry.
#[tauri::command]
pub fn update_entry(state: State<'_, AppState>, id: String, entry: EntryInput) -> AppResult<()> {
    state.with_vault(|vault| vault.unlocked()?.update_entry(&id, &entry))
}

/// Deletes an entry.
#[tauri::command]
pub fn delete_entry(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_vault(|vault| vault.unlocked()?.delete_entry(&id))
}
