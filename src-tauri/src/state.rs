//! Application state shared by the Tauri commands.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::error::{AppError, AppResult};
use crate::vault::Vault;

/// State managed by Tauri: the vault behind a mutex, so only one command
/// uses it at a time.
#[derive(Clone)]
pub struct AppState {
    vault: Arc<Mutex<Vault>>,
}

impl AppState {
    /// Wraps a vault.
    pub fn new(vault: Vault) -> AppState {
        AppState {
            vault: Arc::new(Mutex::new(vault)),
        }
    }

    /// Runs `operation` with exclusive access to the vault.
    ///
    /// # Errors
    /// Returns the error of `operation`.
    pub fn with_vault<T>(
        &self,
        operation: impl FnOnce(&mut Vault) -> AppResult<T>,
    ) -> AppResult<T> {
        operation(&mut self.lock_vault())
    }

    /// Runs a slow `operation` (key derivation, file encryption) on a
    /// blocking thread, so the interface stays responsive.
    ///
    /// # Errors
    /// Returns the error of `operation`, or [`AppError::Io`] if the thread
    /// could not run it.
    pub async fn with_vault_blocking<T, F>(&self, operation: F) -> AppResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Vault) -> AppResult<T> + Send + 'static,
    {
        let state = self.clone();
        tauri::async_runtime::spawn_blocking(move || state.with_vault(operation))
            .await
            .map_err(|_| AppError::Io("background task failed".into()))?
    }

    /// Locks the mutex. A panic while holding it cannot leave the vault in a
    /// half-updated state that matters (database writes are transactional),
    /// so a poisoned mutex is recovered instead of failing forever.
    fn lock_vault(&self) -> MutexGuard<'_, Vault> {
        self.vault
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_vault_returns_the_operation_result() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(Vault::new(dir.path()));
        assert!(!state.with_vault(|vault| Ok(vault.exists())).unwrap());
        assert_eq!(
            state
                .with_vault(|vault| vault.unlocked().map(|_| ()))
                .unwrap_err(),
            AppError::Locked
        );
    }

    #[test]
    fn with_vault_blocking_runs_on_another_thread() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(Vault::new(dir.path()));
        let status =
            tauri::async_runtime::block_on(state.with_vault_blocking(|vault| Ok(vault.status())))
                .unwrap();
        assert!(!status.exists);
    }
}
