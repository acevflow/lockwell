//! Password entries: validation, encryption and CRUD.
//!
//! Every entry is one row of the `entries` table: a random id and the whole
//! entry (title, username, password, URL, notes and timestamps) serialized as
//! JSON and encrypted with the vault key. The associated data binds the row
//! to its id, so rows cannot be swapped or copied under another id.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use super::now_millis;
use super::session::UnlockedVault;
use crate::crypto::aead;
use crate::crypto::random::{is_valid_id, random_id};
use crate::error::{AppError, AppResult};
use crate::storage::db::{self, Table};

/// Maximum length of an entry title, in characters.
pub const MAX_TITLE_CHARS: usize = 200;
/// Maximum length of a username, in characters.
pub const MAX_USERNAME_CHARS: usize = 500;
/// Maximum length of a stored password, in characters.
pub const MAX_PASSWORD_CHARS: usize = 4096;
/// Maximum length of a URL, in characters.
pub const MAX_URL_CHARS: usize = 2048;
/// Maximum length of the notes, in characters.
pub const MAX_NOTES_CHARS: usize = 20_000;

/// Fields of an entry as entered by the user.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryInput {
    /// Name of the entry; required.
    pub title: String,
    /// Username or email.
    #[serde(default)]
    pub username: String,
    /// The stored password.
    #[serde(default)]
    pub password: String,
    /// Website or app address.
    #[serde(default)]
    pub url: String,
    /// Free-form notes.
    #[serde(default)]
    pub notes: String,
}

/// An entry as shown in the list. It has no password or notes, so listing
/// entries never sends secrets to the interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySummary {
    /// Random identifier.
    pub id: String,
    /// Name of the entry.
    pub title: String,
    /// Username or email.
    pub username: String,
    /// Website or app address.
    pub url: String,
    /// Last change, in milliseconds since the Unix epoch.
    pub updated_at: i64,
}

/// A complete entry, sent only when the user opens it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Random identifier (not part of the encrypted payload).
    #[serde(skip_deserializing)]
    pub id: String,
    /// Name of the entry.
    pub title: String,
    /// Username or email.
    pub username: String,
    /// The stored password.
    pub password: String,
    /// Website or app address.
    pub url: String,
    /// Free-form notes.
    pub notes: String,
    /// Creation time, in milliseconds since the Unix epoch.
    pub created_at: i64,
    /// Last change, in milliseconds since the Unix epoch.
    pub updated_at: i64,
}

impl Entry {
    fn summary(&self) -> EntrySummary {
        EntrySummary {
            id: self.id.clone(),
            title: self.title.clone(),
            username: self.username.clone(),
            url: self.url.clone(),
            updated_at: self.updated_at,
        }
    }
}

impl UnlockedVault {
    /// Lists every entry, sorted by title (case-insensitive).
    ///
    /// # Errors
    /// Returns [`AppError::Corrupt`] if any row fails to decrypt.
    pub fn list_entries(&self) -> AppResult<Vec<EntrySummary>> {
        let mut summaries = db::list_records(&self.conn, Table::Entries)?
            .into_iter()
            .map(|(id, data)| self.decrypt_entry(id, &data).map(|entry| entry.summary()))
            .collect::<AppResult<Vec<_>>>()?;
        summaries.sort_by(|a, b| {
            a.title
                .to_lowercase()
                .cmp(&b.title.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(summaries)
    }

    /// Reads one complete entry, including its password.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `id` is malformed.
    /// - [`AppError::NotFound`] if there is no such entry.
    /// - [`AppError::Corrupt`] if the row fails to decrypt.
    pub fn get_entry(&self, id: &str) -> AppResult<Entry> {
        check_id(id)?;
        let data = db::get_record(&self.conn, Table::Entries, id)?;
        self.decrypt_entry(id.to_owned(), &data)
    }

    /// Creates an entry and returns its id.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if a field breaks the rules.
    /// - Storage or cryptographic errors.
    pub fn create_entry(&self, input: &EntryInput) -> AppResult<String> {
        validate_entry(input)?;
        let now = now_millis();
        let entry = entry_from_input(random_id()?, input, now, now);
        let sealed = self.encrypt_entry(&entry)?;
        db::insert_record(&self.conn, Table::Entries, &entry.id, &sealed)?;
        Ok(entry.id.clone())
    }

    /// Replaces the fields of an entry, keeping its creation time.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `id` or a field is invalid.
    /// - [`AppError::NotFound`] if there is no such entry.
    pub fn update_entry(&self, id: &str, input: &EntryInput) -> AppResult<()> {
        validate_entry(input)?;
        let existing = self.get_entry(id)?;
        let updated_at = now_millis().max(existing.updated_at);
        let entry = entry_from_input(existing.id.clone(), input, existing.created_at, updated_at);
        let sealed = self.encrypt_entry(&entry)?;
        db::update_record(&self.conn, Table::Entries, id, &sealed)
    }

    /// Deletes an entry.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `id` is malformed.
    /// - [`AppError::NotFound`] if there is no such entry.
    pub fn delete_entry(&self, id: &str) -> AppResult<()> {
        check_id(id)?;
        db::delete_record(&self.conn, Table::Entries, id)
    }

    fn encrypt_entry(&self, entry: &Entry) -> AppResult<Vec<u8>> {
        let json = Zeroizing::new(
            serde_json::to_vec(entry)
                .map_err(|_| AppError::invalid("the entry cannot be saved"))?,
        );
        Ok(aead::seal(&self.key, &json, &entry_aad(&entry.id))?)
    }

    fn decrypt_entry(&self, id: String, data: &[u8]) -> AppResult<Entry> {
        let json = aead::open(&self.key, data, &entry_aad(&id))
            .map_err(|_| AppError::Corrupt("an entry failed to decrypt".into()))?;
        let mut entry: Entry = serde_json::from_slice(&json)
            .map_err(|_| AppError::Corrupt("an entry has an invalid format".into()))?;
        entry.id = id;
        Ok(entry)
    }
}

/// Associated data of an entry row: type, format version and id.
fn entry_aad(id: &str) -> Vec<u8> {
    format!("lockwell:entry:v1:{id}").into_bytes()
}

fn entry_from_input(id: String, input: &EntryInput, created_at: i64, updated_at: i64) -> Entry {
    Entry {
        id,
        title: input.title.trim().to_owned(),
        username: input.username.trim().to_owned(),
        password: input.password.clone(),
        url: input.url.trim().to_owned(),
        notes: input.notes.clone(),
        created_at,
        updated_at,
    }
}

/// Checks that `id` has the shape of a generated id.
///
/// # Errors
/// Returns [`AppError::InvalidInput`] otherwise.
pub(crate) fn check_id(id: &str) -> AppResult<()> {
    if is_valid_id(id) {
        Ok(())
    } else {
        Err(AppError::invalid("invalid identifier"))
    }
}

/// Checks the fields of an entry.
///
/// # Errors
/// Returns [`AppError::InvalidInput`] naming the field that failed.
pub fn validate_entry(input: &EntryInput) -> AppResult<()> {
    if input.title.trim().is_empty() {
        return Err(AppError::invalid("the title is required"));
    }
    check_length("title", &input.title, MAX_TITLE_CHARS)?;
    check_length("username", &input.username, MAX_USERNAME_CHARS)?;
    check_length("password", &input.password, MAX_PASSWORD_CHARS)?;
    check_length("URL", &input.url, MAX_URL_CHARS)?;
    check_length("notes", &input.notes, MAX_NOTES_CHARS)?;
    Ok(())
}

fn check_length(field: &str, value: &str, max_chars: usize) -> AppResult<()> {
    if value.chars().count() > max_chars {
        return Err(AppError::invalid(format!(
            "the {field} must have at most {max_chars} characters"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::session::tests::{PASSWORD, unlocked_vault};

    fn input(title: &str) -> EntryInput {
        EntryInput {
            title: title.into(),
            username: "user@example.com".into(),
            password: "example-password".into(),
            url: "https://example.com".into(),
            notes: "some notes".into(),
        }
    }

    #[test]
    fn create_and_get() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("Mail")).unwrap();
        assert!(is_valid_id(&id));
        let entry = unlocked.get_entry(&id).unwrap();
        assert_eq!(entry.id, id);
        assert_eq!(entry.title, "Mail");
        assert_eq!(entry.username, "user@example.com");
        assert_eq!(entry.password, "example-password");
        assert_eq!(entry.url, "https://example.com");
        assert_eq!(entry.notes, "some notes");
        assert_eq!(entry.created_at, entry.updated_at);
        assert!(entry.created_at > 0);
    }

    #[test]
    fn trims_single_line_fields_but_keeps_secrets_verbatim() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked
            .create_entry(&EntryInput {
                title: "  Bank  ".into(),
                username: " me ".into(),
                password: " spaced ".into(),
                url: " https://bank.example ".into(),
                notes: " line\n".into(),
            })
            .unwrap();
        let entry = unlocked.get_entry(&id).unwrap();
        assert_eq!(entry.title, "Bank");
        assert_eq!(entry.username, "me");
        assert_eq!(entry.url, "https://bank.example");
        assert_eq!(entry.password, " spaced ");
        assert_eq!(entry.notes, " line\n");
    }

    #[test]
    fn list_is_sorted_and_has_no_secrets() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        unlocked.create_entry(&input("beta")).unwrap();
        unlocked.create_entry(&input("Alpha")).unwrap();
        unlocked.create_entry(&input("gamma")).unwrap();
        let list = unlocked.list_entries().unwrap();
        let titles: Vec<_> = list.iter().map(|e| e.title.as_str()).collect();
        assert_eq!(titles, ["Alpha", "beta", "gamma"]);
        let json = serde_json::to_string(&list).unwrap();
        assert!(!json.contains("example-password"));
        assert!(!json.contains("some notes"));
    }

    #[test]
    fn empty_vault_lists_nothing() {
        let (_dir, vault) = unlocked_vault();
        assert!(vault.unlocked().unwrap().list_entries().unwrap().is_empty());
    }

    #[test]
    fn update_keeps_creation_time() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("Old")).unwrap();
        let before = unlocked.get_entry(&id).unwrap();
        let mut changed = input("New");
        changed.password = "another-password".into();
        unlocked.update_entry(&id, &changed).unwrap();
        let after = unlocked.get_entry(&id).unwrap();
        assert_eq!(after.title, "New");
        assert_eq!(after.password, "another-password");
        assert_eq!(after.created_at, before.created_at);
        assert!(after.updated_at >= before.updated_at);
    }

    #[test]
    fn delete_removes_the_entry() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("Gone")).unwrap();
        unlocked.delete_entry(&id).unwrap();
        assert_eq!(
            unlocked.get_entry(&id).unwrap_err(),
            AppError::NotFound("entry")
        );
        assert_eq!(
            unlocked.delete_entry(&id).unwrap_err(),
            AppError::NotFound("entry")
        );
    }

    #[test]
    fn unknown_ids_are_not_found() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = random_id().unwrap();
        assert_eq!(
            unlocked.get_entry(&id).unwrap_err(),
            AppError::NotFound("entry")
        );
        assert_eq!(
            unlocked.update_entry(&id, &input("x")).unwrap_err(),
            AppError::NotFound("entry")
        );
    }

    #[test]
    fn malformed_ids_are_rejected() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        for id in ["", "1", "' OR 1=1 --", &"Z".repeat(32)] {
            assert!(matches!(
                unlocked.get_entry(id),
                Err(AppError::InvalidInput(_))
            ));
            assert!(matches!(
                unlocked.delete_entry(id),
                Err(AppError::InvalidInput(_))
            ));
            assert!(matches!(
                unlocked.update_entry(id, &input("x")),
                Err(AppError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn validation_rules() {
        assert!(validate_entry(&input("ok")).is_ok());
        assert!(validate_entry(&input("")).is_err());
        assert!(validate_entry(&input("   ")).is_err());
        assert!(validate_entry(&input(&"t".repeat(MAX_TITLE_CHARS))).is_ok());
        assert!(validate_entry(&input(&"t".repeat(MAX_TITLE_CHARS + 1))).is_err());

        let mut long = input("ok");
        long.username = "u".repeat(MAX_USERNAME_CHARS + 1);
        assert!(validate_entry(&long).is_err());
        let mut long = input("ok");
        long.password = "p".repeat(MAX_PASSWORD_CHARS + 1);
        assert!(validate_entry(&long).is_err());
        let mut long = input("ok");
        long.url = "u".repeat(MAX_URL_CHARS + 1);
        assert!(validate_entry(&long).is_err());
        let mut long = input("ok");
        long.notes = "n".repeat(MAX_NOTES_CHARS + 1);
        let message = validate_entry(&long).unwrap_err().to_string();
        assert!(message.contains("notes"));
    }

    #[test]
    fn invalid_input_is_not_stored() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        assert!(unlocked.create_entry(&input("")).is_err());
        assert!(unlocked.list_entries().unwrap().is_empty());
    }

    #[test]
    fn values_are_encrypted_at_rest() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        unlocked.create_entry(&input("Visible title")).unwrap();
        let rows = db::list_records(&unlocked.conn, Table::Entries).unwrap();
        let raw = &rows[0].1;
        for needle in [
            &b"Visible title"[..],
            b"example-password",
            b"user@example.com",
        ] {
            assert!(!raw.windows(needle.len()).any(|window| window == needle));
        }
    }

    #[test]
    fn swapped_rows_are_detected() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let a = unlocked.create_entry(&input("A")).unwrap();
        let b = unlocked.create_entry(&input("B")).unwrap();
        let data_a = db::get_record(&unlocked.conn, Table::Entries, &a).unwrap();
        db::update_record(&unlocked.conn, Table::Entries, &b, &data_a).unwrap();
        assert!(matches!(unlocked.get_entry(&b), Err(AppError::Corrupt(_))));
        assert!(matches!(unlocked.list_entries(), Err(AppError::Corrupt(_))));
        assert_eq!(unlocked.get_entry(&a).unwrap().title, "A");
    }

    #[test]
    fn tampered_rows_are_detected() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("A")).unwrap();
        let mut data = db::get_record(&unlocked.conn, Table::Entries, &id).unwrap();
        let last = data.len() - 1;
        data[last] ^= 1;
        db::update_record(&unlocked.conn, Table::Entries, &id, &data).unwrap();
        assert!(matches!(unlocked.get_entry(&id), Err(AppError::Corrupt(_))));
    }

    #[test]
    fn entries_survive_lock_and_password_change() {
        let (_dir, mut vault) = unlocked_vault();
        let id = vault
            .unlocked()
            .unwrap()
            .create_entry(&input("Kept"))
            .unwrap();
        vault
            .change_master_password(PASSWORD, "a brand new password")
            .unwrap();
        vault.lock();
        assert_eq!(vault.unlocked().err(), Some(AppError::Locked));
        vault.unlock("a brand new password").unwrap();
        assert_eq!(
            vault.unlocked().unwrap().get_entry(&id).unwrap().title,
            "Kept"
        );
    }

    #[test]
    fn input_rejects_unknown_fields() {
        let parsed: Result<EntryInput, _> = serde_json::from_str(r#"{"title":"x","createdAt":1}"#);
        assert!(parsed.is_err());
        let parsed: EntryInput = serde_json::from_str(r#"{"title":"x"}"#).unwrap();
        assert_eq!(parsed.password, "");
    }

    /// An input where every field is exactly at its limit, counted in
    /// characters that take several bytes each.
    fn input_at_the_limits() -> EntryInput {
        EntryInput {
            title: "é".repeat(MAX_TITLE_CHARS),
            username: "é".repeat(MAX_USERNAME_CHARS),
            password: "é".repeat(MAX_PASSWORD_CHARS),
            url: "é".repeat(MAX_URL_CHARS),
            notes: "é".repeat(MAX_NOTES_CHARS),
        }
    }

    #[test]
    fn accepts_every_field_at_its_limit() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input_at_the_limits()).unwrap();
        assert_eq!(
            unlocked.get_entry(&id).unwrap().notes.chars().count(),
            MAX_NOTES_CHARS
        );
    }

    /// Makes one field of an entry one character longer.
    type Lengthen = fn(&mut EntryInput);

    #[test]
    fn names_the_field_that_is_too_long() {
        let cases: [(Lengthen, &str); 5] = [
            (
                |e| e.title.push('x'),
                "the title must have at most 200 characters",
            ),
            (
                |e| e.username.push('x'),
                "the username must have at most 500 characters",
            ),
            (
                |e| e.password.push('x'),
                "the password must have at most 4096 characters",
            ),
            (
                |e| e.url.push('x'),
                "the URL must have at most 2048 characters",
            ),
            (
                |e| e.notes.push('x'),
                "the notes must have at most 20000 characters",
            ),
        ];
        for (lengthen, message) in cases {
            let mut entry = input_at_the_limits();
            lengthen(&mut entry);
            assert_eq!(validate_entry(&entry), Err(AppError::invalid(message)));
        }
    }

    #[test]
    fn invalid_updates_leave_the_entry_unchanged() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("Original")).unwrap();
        assert_eq!(
            unlocked.update_entry(&id, &input(" ")).unwrap_err(),
            AppError::invalid("the title is required")
        );
        assert_eq!(unlocked.get_entry(&id).unwrap().title, "Original");
    }

    #[test]
    fn deleting_twice_is_not_found() {
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("Once")).unwrap();
        unlocked.delete_entry(&id).unwrap();
        assert_eq!(
            unlocked.delete_entry(&id).unwrap_err(),
            AppError::NotFound("entry")
        );
    }

    #[test]
    fn authentic_rows_with_an_invalid_format_are_corrupt() {
        // The row decrypts with the right key and context, but is not an entry.
        let (_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let id = unlocked.create_entry(&input("Replaced")).unwrap();
        let sealed = aead::seal(&unlocked.key, b"not json", &entry_aad(&id)).unwrap();
        db::update_record(&unlocked.conn, Table::Entries, &id, &sealed).unwrap();

        let expected = AppError::Corrupt("an entry has an invalid format".into());
        assert_eq!(unlocked.get_entry(&id).unwrap_err(), expected);
        assert_eq!(unlocked.list_entries().unwrap_err(), expected);
    }

    #[test]
    #[ignore = "FINDING: Entry and EntryInput derive Debug, which prints the password \
                and notes in plain text (SecretKey redacts its Debug output)"]
    fn debug_output_hides_secrets() {
        let mut entry = input("Mail");
        entry.password = "synthetic-secret-value".into();
        entry.notes = "synthetic-secret-notes".into();
        let printed = format!("{entry:?}");
        assert!(!printed.contains("synthetic-secret-value"));
        assert!(!printed.contains("synthetic-secret-notes"));
    }
}
