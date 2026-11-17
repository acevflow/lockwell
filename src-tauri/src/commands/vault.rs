//! Commands to create, unlock and lock the vault and to change the master
//! password.
//!
//! Passwords arrive from the interface as strings; they are moved into
//! wiped-on-drop buffers right away and never logged.

use tauri::State;
use zeroize::Zeroizing;

use crate::error::AppResult;
use crate::state::AppState;
use crate::vault::VaultStatus;

/// Returns whether a vault exists and whether it is unlocked.
#[tauri::command]
pub fn vault_status(state: State<'_, AppState>) -> AppResult<VaultStatus> {
    state.with_vault(|vault| Ok(vault.status()))
}

/// Creates a new vault protected by `password` and unlocks it.
///
/// Errors: `invalid_input` if the password breaks the policy, `vault_exists`
/// if a vault is already there.
#[tauri::command]
pub async fn create_vault(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let password = Zeroizing::new(password);
    state
        .with_vault_blocking(move |vault| vault.create(&password))
        .await
}

/// Unlocks the vault with `password`.
///
/// Errors: `wrong_password` if the password is wrong or the vault key data is
/// damaged, `vault_not_found`, `corrupt` or `unsupported_version` if the vault
/// cannot be read.
#[tauri::command]
pub async fn unlock_vault(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let password = Zeroizing::new(password);
    state
        .with_vault_blocking(move |vault| vault.unlock(&password))
        .await
}

/// Locks the vault, wipes its key from memory and clears a secret this app
/// left on the clipboard.
#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) {
    state.lock();
}

/// Replaces the master password after checking the current one. The vault
/// must be unlocked, and the data is not re-encrypted.
///
/// Errors: `locked`, `wrong_password` or `invalid_input` (policy, or the new
/// password equals the current one).
#[tauri::command]
pub async fn change_master_password(
    state: State<'_, AppState>,
    current: String,
    new: String,
) -> AppResult<()> {
    let current = Zeroizing::new(current);
    let new = Zeroizing::new(new);
    state
        .with_vault_blocking(move |vault| vault.change_master_password(&current, &new))
        .await
}
