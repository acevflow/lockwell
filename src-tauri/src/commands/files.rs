//! Commands to list, add, save a copy of and delete encrypted files.
//!
//! The native open and save dialogs are shown from the core, so the
//! interface never handles file system paths.

use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;

use super::dialog::{run_blocking, to_path};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::vault::files::FileSummary;

/// Lists the stored files.
#[tauri::command]
pub fn list_files(state: State<'_, AppState>) -> AppResult<Vec<FileSummary>> {
    state.with_vault(|vault| vault.unlocked()?.list_files())
}

/// Asks the user for files and encrypts them into the vault. Returns the
/// added files; an empty list means the user cancelled.
#[tauri::command]
pub async fn import_files<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> AppResult<Vec<FileSummary>> {
    state.with_vault(|vault| vault.unlocked().map(|_| ()))?;
    let chosen = run_blocking(move || {
        app.dialog()
            .file()
            .set_title("Add files to the vault")
            .blocking_pick_files()
    })
    .await?;
    let Some(paths) = chosen else {
        return Ok(Vec::new());
    };
    let paths = paths
        .into_iter()
        .map(to_path)
        .collect::<AppResult<Vec<_>>>()?;
    state
        .with_vault_blocking(move |vault| {
            let unlocked = vault.unlocked()?;
            paths
                .iter()
                .map(|path| {
                    unlocked.import_file(path).map_err(|error| {
                        let name = path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        AppError::invalid(format!("could not add “{name}”: {error}"))
                    })
                })
                .collect()
        })
        .await
}

/// Asks the user where to save a decrypted copy of a file and writes it.
/// Returns `false` if the user cancelled.
#[tauri::command]
pub async fn export_file<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<bool> {
    let file = state.with_vault(|vault| vault.unlocked()?.get_file(&id))?;
    let chosen = run_blocking(move || {
        app.dialog()
            .file()
            .set_title("Save a decrypted copy")
            .set_file_name(file.name)
            .blocking_save_file()
    })
    .await?;
    let Some(destination) = chosen else {
        return Ok(false);
    };
    let destination = to_path(destination)?;
    // The native save dialog has already asked before replacing a file.
    state
        .with_vault_blocking(move |vault| {
            vault
                .unlocked()?
                .export_file(&id, &destination, true)
                .map(|()| true)
        })
        .await
}

/// Deletes a stored file.
#[tauri::command]
pub fn delete_file(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_vault(|vault| vault.unlocked()?.delete_file(&id))
}
