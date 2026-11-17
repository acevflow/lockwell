//! Application state shared by the Tauri commands and background watchers.
//!
//! Every way of locking the vault (the command, the inactivity timer and the
//! Windows session lock) goes through [`AppState`], so the clipboard is
//! cleared and the vault is wiped in one place.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::error::{AppError, AppResult};
use crate::security::autolock::{ActivityTracker, IDLE_TIMEOUT};
use crate::security::clipboard::{ClipboardBackend, SecureClipboard, SystemClipboard};
use crate::vault::Vault;

/// State managed by Tauri: the vault, the time of the last user activity
/// and the clipboard. Each part sits behind its own mutex; no code holds
/// the clipboard mutex while taking the vault one, so they cannot deadlock.
#[derive(Clone)]
pub struct AppState {
    vault: Arc<Mutex<Vault>>,
    activity: Arc<Mutex<ActivityTracker>>,
    clipboard: Arc<Mutex<SecureClipboard>>,
}

impl AppState {
    /// State over `vault` and the system clipboard.
    pub fn new(vault: Vault) -> AppState {
        AppState::with_clipboard(vault, Box::new(SystemClipboard))
    }

    /// State over `vault` and a given clipboard backend.
    pub fn with_clipboard(vault: Vault, clipboard: Box<dyn ClipboardBackend>) -> AppState {
        AppState {
            vault: Arc::new(Mutex::new(vault)),
            activity: Arc::new(Mutex::new(ActivityTracker::new(
                Instant::now(),
                IDLE_TIMEOUT,
            ))),
            clipboard: Arc::new(Mutex::new(SecureClipboard::new(clipboard))),
        }
    }

    /// Runs `operation` with exclusive access to the vault. Every command
    /// counts as user activity, before and after it runs (so a long
    /// operation does not trigger the automatic lock as soon as it ends).
    ///
    /// # Errors
    /// Returns the error of `operation`.
    pub fn with_vault<T>(
        &self,
        operation: impl FnOnce(&mut Vault) -> AppResult<T>,
    ) -> AppResult<T> {
        self.touch();
        let mut vault = lock(&self.vault);
        let result = operation(&mut vault);
        // Still holding the vault, so an idle check that was waiting for it
        // sees this activity.
        self.touch();
        drop(vault);
        result
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

    /// Records user activity now.
    pub fn touch(&self) {
        lock(&self.activity).touch(Instant::now());
    }

    /// Locks the vault and clears a secret this app left on the clipboard.
    /// Returns whether the vault was unlocked.
    pub fn lock(&self) -> bool {
        self.lock_vault(None)
    }

    /// Locks the vault if it is unlocked and there was no activity for the
    /// idle timeout as of `now`. Returns whether it locked.
    ///
    /// Inactivity is checked once the vault is free: an operation that was
    /// running counts as activity when it ends.
    pub fn lock_if_idle(&self, now: Instant) -> bool {
        self.lock_vault(Some(now))
    }

    /// Locks the vault and clears the clipboard. With `idle_as_of`, it does
    /// nothing unless the user has been idle as of that time.
    fn lock_vault(&self, idle_as_of: Option<Instant>) -> bool {
        let was_unlocked = {
            let mut vault = lock(&self.vault);
            if let Some(now) = idle_as_of
                && !lock(&self.activity).is_idle(now)
            {
                return false;
            }
            let was_unlocked = vault.is_unlocked();
            vault.lock();
            was_unlocked
        };
        lock(&self.clipboard).clear_if_unchanged(None);
        was_unlocked
    }

    /// Clears a secret this app left on the clipboard when the app exits. The
    /// clearing timer would die with the process, so without this a secret
    /// copied just before closing would stay on the clipboard.
    ///
    /// It does not take the vault: exiting must not wait for a long operation,
    /// and the key disappears with the process. Returns whether the clipboard
    /// was cleared.
    pub fn clear_clipboard_on_exit(&self) -> bool {
        lock(&self.clipboard).clear_if_unchanged(None)
    }

    /// Copies a secret to the clipboard and returns its generation, which the
    /// clearing timer later passes to [`AppState::clear_clipboard_if_unchanged`].
    ///
    /// # Errors
    /// Returns [`AppError::Io`] if the clipboard cannot be written.
    pub fn copy_secret(&self, text: &str) -> AppResult<u64> {
        lock(&self.clipboard).copy(text)
    }

    /// Clears the copy `generation` if the clipboard still holds it. Returns
    /// whether the clipboard was cleared.
    pub fn clear_clipboard_if_unchanged(&self, generation: u64) -> bool {
        lock(&self.clipboard).clear_if_unchanged(Some(generation))
    }
}

/// Locks a mutex. A panic while holding it cannot leave the state half
/// updated in a way that matters (database writes are transactional), so a
/// poisoned mutex is recovered instead of failing forever.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::security::clipboard::tests::FakeClipboard;
    use crate::vault::header::tests::FAST_KDF;
    use crate::vault::session::tests::PASSWORD;

    fn unlocked_state() -> (tempfile::TempDir, FakeClipboard, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeClipboard::default();
        let state = AppState::with_clipboard(
            Vault::with_kdf(dir.path().join("vault"), FAST_KDF),
            Box::new(fake.clone()),
        );
        state.with_vault(|vault| vault.create(PASSWORD)).unwrap();
        (dir, fake, state)
    }

    fn is_unlocked(state: &AppState) -> bool {
        state.with_vault(|vault| Ok(vault.is_unlocked())).unwrap()
    }

    #[test]
    fn with_vault_returns_the_operation_result() {
        let dir = tempfile::tempdir().unwrap();
        let state =
            AppState::with_clipboard(Vault::new(dir.path()), Box::new(FakeClipboard::default()));
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
        let state =
            AppState::with_clipboard(Vault::new(dir.path()), Box::new(FakeClipboard::default()));
        let status =
            tauri::async_runtime::block_on(state.with_vault_blocking(|vault| Ok(vault.status())))
                .unwrap();
        assert!(!status.exists);
    }

    /// Closing the app within the clipboard time must not leave the secret
    /// behind: the clearing timer dies with the process.
    #[test]
    fn exiting_clears_the_copied_secret_without_waiting_for_the_vault() {
        let (_dir, fake, state) = unlocked_state();
        state.copy_secret("example-secret").unwrap();
        // A long operation holds the vault; exiting must not wait for it.
        let held = lock(&state.vault);
        assert!(state.clear_clipboard_on_exit());
        drop(held);
        assert_eq!(*fake.content.lock().unwrap(), None);
    }

    /// Text the user copied by themselves is never cleared on exit.
    #[test]
    fn exiting_leaves_text_the_user_copied() {
        let (_dir, fake, state) = unlocked_state();
        state.copy_secret("example-secret").unwrap();
        *fake.content.lock().unwrap() = Some("user text".into());
        assert!(!state.clear_clipboard_on_exit());
        assert_eq!(fake.content.lock().unwrap().as_deref(), Some("user text"));
    }

    #[test]
    fn lock_locks_and_clears_the_copied_secret() {
        let (_dir, fake, state) = unlocked_state();
        state.copy_secret("example-secret").unwrap();
        assert!(state.lock());
        assert!(!is_unlocked(&state));
        assert_eq!(*fake.content.lock().unwrap(), None);
        assert!(!state.lock());
    }

    #[test]
    fn lock_keeps_what_the_user_copied() {
        let (_dir, fake, state) = unlocked_state();
        state.copy_secret("example-secret").unwrap();
        *fake.content.lock().unwrap() = Some("user text".into());
        state.lock();
        assert_eq!(fake.content.lock().unwrap().as_deref(), Some("user text"));
    }

    #[test]
    fn lock_if_idle_waits_for_the_timeout() {
        let (_dir, _fake, state) = unlocked_state();
        let now = Instant::now();
        assert!(!state.lock_if_idle(now));
        assert!(is_unlocked(&state));
        assert!(state.lock_if_idle(now + IDLE_TIMEOUT + Duration::from_secs(1)));
        assert!(!is_unlocked(&state));
    }

    #[test]
    fn lock_if_idle_does_nothing_when_already_locked() {
        let (_dir, _fake, state) = unlocked_state();
        state.lock();
        assert!(!state.lock_if_idle(Instant::now() + IDLE_TIMEOUT * 2));
    }

    #[test]
    fn activity_postpones_the_idle_lock() {
        let (_dir, _fake, state) = unlocked_state();
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(20));
        state.touch();
        assert!(!state.lock_if_idle(start + IDLE_TIMEOUT));
    }

    #[test]
    fn a_long_operation_postpones_the_idle_lock_that_was_waiting_for_it() {
        let (_dir, _fake, state) = unlocked_state();
        let (started_sender, started) = std::sync::mpsc::channel();
        let worker_state = state.clone();
        let worker = std::thread::spawn(move || {
            worker_state.with_vault(|_| {
                started_sender.send(()).unwrap();
                std::thread::sleep(Duration::from_millis(500));
                Ok(())
            })
        });
        started.recv().unwrap();

        // The watcher sees no activity for the whole timeout at `later`, but
        // has to wait for the operation, which counts as activity when it ends.
        let later = Instant::now() + IDLE_TIMEOUT + Duration::from_millis(100);
        let locked = state.lock_if_idle(later);
        worker.join().unwrap().unwrap();

        assert!(!locked);
        assert!(is_unlocked(&state));
    }

    #[test]
    fn timed_clear_only_clears_its_own_copy() {
        let (_dir, fake, state) = unlocked_state();
        let first = state.copy_secret("first").unwrap();
        state.copy_secret("second").unwrap();
        assert!(!state.clear_clipboard_if_unchanged(first));
        assert_eq!(fake.content.lock().unwrap().as_deref(), Some("second"));
    }

    #[test]
    fn copy_errors_reach_the_caller() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeClipboard {
            fail: true,
            ..FakeClipboard::default()
        };
        let state = AppState::with_clipboard(
            Vault::with_kdf(dir.path().join("vault"), FAST_KDF),
            Box::new(fake),
        );
        assert_eq!(
            state.copy_secret("example-secret").unwrap_err(),
            AppError::Io("the clipboard is not available".into())
        );
    }

    #[test]
    fn a_panic_inside_an_operation_does_not_block_the_vault() {
        let (_dir, _fake, state) = unlocked_state();
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: AppResult<()> = state.with_vault(|_| panic!("operation failed"));
        }));
        assert!(panicked.is_err());
        assert!(is_unlocked(&state));
        assert!(state.lock());
        assert!(!is_unlocked(&state));
    }

    #[test]
    fn locking_reports_whether_the_vault_was_unlocked() {
        let (_dir, _fake, state) = unlocked_state();
        assert!(state.lock());
        assert!(!state.lock());
    }
}
