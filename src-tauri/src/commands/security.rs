//! Commands for user activity and copying secrets to the clipboard.

use serde::Deserialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::security::schedule_clipboard_clear;
use crate::state::AppState;

/// Field of an entry that can be copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryField {
    /// The username.
    Username,
    /// The password.
    Password,
}

/// Records user activity, postponing the automatic lock.
#[tauri::command]
pub fn touch_activity(state: State<'_, AppState>) {
    state.touch();
}

/// Copies a field of an entry to the clipboard without sending it to the
/// interface. It is cleared after 30 seconds if still on the clipboard.
#[tauri::command]
pub fn copy_entry_field(
    state: State<'_, AppState>,
    id: String,
    field: EntryField,
) -> AppResult<()> {
    copy_field(&state, &id, field)
}

/// Reads the field from the vault, copies it and schedules the clipboard
/// clearing. The value is never returned to the interface. Fails with
/// `invalid_input` if the field is empty.
fn copy_field(state: &AppState, id: &str, field: EntryField) -> AppResult<()> {
    let entry = state.with_vault(|vault| vault.unlocked()?.get_entry(id))?;
    let value = match field {
        EntryField::Username => &entry.username,
        EntryField::Password => &entry.password,
    };
    if value.is_empty() {
        return Err(AppError::invalid("this field is empty"));
    }
    let generation = state.copy_secret(value)?;
    schedule_clipboard_clear(state.clone(), generation)
        .map_err(|_| AppError::Io("cannot schedule clearing the clipboard".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::clipboard::tests::FakeClipboard;
    use crate::vault::Vault;
    use crate::vault::entries::EntryInput;
    use crate::vault::header::tests::FAST_KDF;
    use crate::vault::session::tests::PASSWORD;

    fn state_with_entry(username: &str) -> (tempfile::TempDir, FakeClipboard, AppState, String) {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeClipboard::default();
        let state = AppState::with_clipboard(
            Vault::with_kdf(dir.path().join("vault"), FAST_KDF),
            Box::new(fake.clone()),
        );
        let id = state
            .with_vault(|vault| {
                vault.create(PASSWORD)?;
                vault.unlocked()?.create_entry(&EntryInput {
                    title: "Mail".into(),
                    username: username.into(),
                    password: "example-password".into(),
                    url: String::new(),
                    notes: String::new(),
                })
            })
            .unwrap();
        (dir, fake, state, id)
    }

    #[test]
    fn copies_the_password_and_the_username() {
        let (_dir, fake, state, id) = state_with_entry("me@example.com");
        copy_field(&state, &id, EntryField::Password).unwrap();
        assert_eq!(
            fake.content.lock().unwrap().as_deref(),
            Some("example-password")
        );
        copy_field(&state, &id, EntryField::Username).unwrap();
        assert_eq!(
            fake.content.lock().unwrap().as_deref(),
            Some("me@example.com")
        );
    }

    #[test]
    fn refuses_an_empty_field() {
        let (_dir, fake, state, id) = state_with_entry("");
        assert!(matches!(
            copy_field(&state, &id, EntryField::Username),
            Err(AppError::InvalidInput(_))
        ));
        assert_eq!(*fake.content.lock().unwrap(), None);
    }

    #[test]
    fn needs_an_unlocked_vault() {
        let (_dir, fake, state, id) = state_with_entry("me");
        state.lock();
        assert_eq!(
            copy_field(&state, &id, EntryField::Password).unwrap_err(),
            AppError::Locked
        );
        assert_eq!(*fake.content.lock().unwrap(), None);
    }

    #[test]
    fn parses_field_names() {
        assert_eq!(
            serde_json::from_str::<EntryField>(r#""password""#).unwrap(),
            EntryField::Password
        );
        assert!(serde_json::from_str::<EntryField>(r#""notes""#).is_err());
    }
}
