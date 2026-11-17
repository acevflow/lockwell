//! Integration tests of the security guarantees: no secrets in errors or
//! debug output, input validation at the boundary, and locking that clears
//! the clipboard.

mod common;

use std::time::{Duration, Instant};

use common::{MemoryClipboard, PASSWORD, Workspace, entry};
use lockwell_lib::crypto::keys::SecretKey;
use lockwell_lib::error::AppError;
use lockwell_lib::security::autolock::IDLE_TIMEOUT;
use lockwell_lib::state::AppState;
use lockwell_lib::vault::entries::EntryInput;

/// An app state over a new, unlocked vault and an in-memory clipboard.
fn unlocked_state(workspace: &Workspace) -> (AppState, MemoryClipboard) {
    let clipboard = MemoryClipboard::default();
    let state = AppState::with_clipboard(workspace.vault(), Box::new(clipboard.clone()));
    state.with_vault(|vault| vault.create(PASSWORD)).unwrap();
    (state, clipboard)
}

/// Provokes one error of each kind (validation, file system, wrong password)
/// and checks that the serialized errors do not repeat the secrets, the paths
/// or the passwords that were sent in.
#[test]
fn errors_never_contain_passwords_data_or_paths() {
    let workspace = Workspace::new();
    let mut vault = workspace.created_vault();
    // No title, so the entry is rejected.
    let mut secret_input = EntryInput::default();
    secret_input.password = "synthetic-secret-value".into();
    let missing = workspace
        .outside()
        .join("synthetic-private-folder")
        .join("x.txt");

    let mut errors = vec![
        vault
            .unlocked()
            .unwrap()
            .create_entry(&secret_input)
            .unwrap_err(),
        vault.unlocked().unwrap().import_file(&missing).unwrap_err(),
        vault
            .change_master_password("synthetic-wrong-current", "synthetic new password")
            .unwrap_err(),
    ];
    vault.lock();
    errors.push(vault.unlock("synthetic-wrong-password").unwrap_err());

    let messages: String = errors
        .iter()
        .map(|error| serde_json::to_string(error).unwrap())
        .collect();
    for secret in [
        "synthetic-secret-value",
        "synthetic-private-folder",
        "synthetic-wrong-current",
        "synthetic new password",
        "synthetic-wrong-password",
        PASSWORD,
    ] {
        assert!(!messages.contains(secret), "{secret} leaked: {messages}");
    }
}

#[test]
fn keys_are_redacted_in_debug_output() {
    let key = SecretKey::from_slice(&[0xAB; 32]).unwrap();
    let printed = format!("{key:?}");
    assert!(!printed.contains("171"));
    assert!(!printed.to_lowercase().contains("ab"));
}

/// SQL, path traversal, NUL and oversized identifiers are refused as invalid
/// input by every operation that takes an id, and the stored data is intact.
#[test]
fn malicious_ids_are_rejected_before_reaching_storage() {
    let workspace = Workspace::new();
    let vault = workspace.created_vault();
    let unlocked = vault.unlocked().unwrap();
    unlocked.create_entry(&entry("safe")).unwrap();
    let long = "x".repeat(10_000);
    for id in ["' OR '1'='1", "../../vault", "%00", long.as_str()] {
        assert!(matches!(
            unlocked.get_entry(id),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            unlocked.delete_entry(id),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            unlocked.get_file(id),
            Err(AppError::InvalidInput(_))
        ));
    }
    assert_eq!(unlocked.list_entries().unwrap().len(), 1);
}

/// A caller cannot set fields such as `id` or `createdAt` through the input.
#[test]
fn input_with_unknown_fields_is_refused() {
    let parsed: Result<EntryInput, _> =
        serde_json::from_str(r#"{"title":"x","id":"forged","createdAt":0}"#);
    assert!(parsed.is_err());
}

#[test]
fn weak_master_passwords_are_refused() {
    let workspace = Workspace::new();
    let mut vault = workspace.vault();
    for weak in ["", "short", "elevenchars", "            "] {
        assert!(matches!(vault.create(weak), Err(AppError::InvalidInput(_))));
    }
    assert!(!vault.exists());
}

/// After locking, the copied secret is gone; if the user copied something
/// else in the meantime, that text is left alone.
#[test]
fn locking_clears_a_copied_secret_but_not_user_text() {
    let workspace = Workspace::new();
    let (state, clipboard) = unlocked_state(&workspace);

    state.copy_secret("synthetic-secret-value").unwrap();
    assert!(state.lock());
    assert_eq!(clipboard.text(), None);

    state.with_vault(|vault| vault.unlock(PASSWORD)).unwrap();
    state.copy_secret("synthetic-secret-value").unwrap();
    clipboard.set("text the user copied");
    assert!(state.lock());
    assert_eq!(clipboard.text().as_deref(), Some("text the user copied"));
}

/// Past the idle timeout the vault locks and the clipboard is cleared.
#[test]
fn the_idle_lock_locks_and_clears_the_clipboard() {
    let workspace = Workspace::new();
    let (state, clipboard) = unlocked_state(&workspace);
    state.copy_secret("synthetic-secret-value").unwrap();

    let later = Instant::now() + IDLE_TIMEOUT + Duration::from_secs(1);
    assert!(state.lock_if_idle(later));
    assert_eq!(clipboard.text(), None);
    assert_eq!(
        state.with_vault(|vault| vault.unlocked().map(|_| ())),
        Err(AppError::Locked)
    );
}
