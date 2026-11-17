//! Helpers shared by the commands that show native dialogs.

use std::path::PathBuf;

use tauri_plugin_dialog::FilePath;

use crate::error::{AppError, AppResult};

/// Runs a native dialog off the async runtime threads.
pub(super) async fn run_blocking<T: Send + 'static>(
    dialog: impl FnOnce() -> T + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(dialog)
        .await
        .map_err(|_| AppError::Io("the file dialog failed".into()))
}

/// Converts a location chosen in a dialog to a local path.
pub(super) fn to_path(path: FilePath) -> AppResult<PathBuf> {
    path.into_path()
        .map_err(|_| AppError::invalid("the selected location is not a local file"))
}
