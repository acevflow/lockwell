//! Security controls around the vault: automatic lock after inactivity or
//! when Windows locks, and clearing copied secrets from the clipboard.
//!
//! When the core locks the vault on its own, it emits the
//! [`VAULT_LOCKED_EVENT`] event so the interface drops its decrypted state.

pub mod autolock;
pub mod clipboard;
#[cfg(windows)]
pub mod session_lock_windows;

use std::thread;
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::state::AppState;

/// Event emitted when the core locks the vault without a user request.
pub const VAULT_LOCKED_EVENT: &str = "vault-locked";

/// Why the core locked the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LockReason {
    /// No activity for [`autolock::IDLE_TIMEOUT`].
    Idle,
    /// The Windows session was locked.
    SystemLock,
}

#[derive(Clone, Serialize)]
struct VaultLockedPayload {
    reason: LockReason,
}

/// Tells the interface that the vault was locked.
pub fn notify_locked<R: Runtime>(app: &AppHandle<R>, reason: LockReason) {
    if let Err(error) = app.emit(VAULT_LOCKED_EVENT, VaultLockedPayload { reason }) {
        // The vault is already locked; the interface will find out on its
        // next command, which fails with `locked`.
        eprintln!("lockwell: could not notify the interface of the lock: {error}");
    }
}

/// Starts the thread that locks the vault after the inactivity timeout.
///
/// # Errors
/// Returns an error if the thread cannot be started.
pub fn spawn_idle_watcher<R: Runtime>(app: AppHandle<R>) -> std::io::Result<()> {
    thread::Builder::new()
        .name("idle-watcher".into())
        .spawn(move || {
            loop {
                thread::sleep(autolock::CHECK_INTERVAL);
                if app.state::<AppState>().lock_if_idle(Instant::now()) {
                    notify_locked(&app, LockReason::Idle);
                }
            }
        })
        .map(|_| ())
}

/// Locks the vault whenever Windows locks the session.
///
/// # Errors
/// Returns a description of the failure if the watcher cannot start.
#[cfg(windows)]
pub fn watch_system_lock<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    session_lock_windows::watch(move || {
        if app.state::<AppState>().lock() {
            notify_locked(&app, LockReason::SystemLock);
        }
    })
}

/// Clears copy `generation` from the clipboard after
/// [`clipboard::CLEAR_AFTER`], if the clipboard still holds it.
///
/// # Errors
/// Returns an error if the timer thread cannot be started.
pub fn schedule_clipboard_clear(state: AppState, generation: u64) -> std::io::Result<()> {
    thread::Builder::new()
        .name("clipboard-clear".into())
        .spawn(move || {
            thread::sleep(clipboard::CLEAR_AFTER);
            state.clear_clipboard_if_unchanged(generation);
        })
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_reasons_serialize_for_the_interface() {
        let payload = VaultLockedPayload {
            reason: LockReason::SystemLock,
        };
        assert_eq!(
            serde_json::to_string(&payload).unwrap(),
            r#"{"reason":"systemLock"}"#
        );
        assert_eq!(
            serde_json::to_string(&LockReason::Idle).unwrap(),
            r#""idle""#
        );
    }
}
