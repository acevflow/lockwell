//! The vault and its locked/unlocked session.
//!
//! A [`Vault`] owns the vault folder. While locked it holds no key and no
//! open database. Unlocking derives the key from the master password and
//! keeps it, with the database connection, in an [`UnlockedVault`]. Locking
//! drops both; the key is wiped from memory when dropped.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;

use super::header::VaultHeader;
use super::password_policy::validate_master_password;
use crate::crypto::kdf::KdfParams;
use crate::crypto::keys::SecretKey;
use crate::error::{AppError, AppResult};
use crate::storage::db;

/// File name of the vault database inside the vault folder.
pub const DATABASE_FILE: &str = "vault.db";
/// Folder for encrypted files inside the vault folder.
pub const FILES_DIR: &str = "files";
const HEADER_META_KEY: &str = "header";

/// Whether a vault exists and whether it is unlocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    /// A vault has been created.
    pub exists: bool,
    /// The vault is unlocked.
    pub unlocked: bool,
}

/// State of an unlocked vault: the vault folder, the open database and the
/// vault key.
pub struct UnlockedVault {
    pub(crate) dir: PathBuf,
    pub(crate) conn: Connection,
    pub(crate) key: SecretKey,
}

impl UnlockedVault {
    /// Folder that holds the encrypted files.
    pub(crate) fn files_dir(&self) -> PathBuf {
        self.dir.join(FILES_DIR)
    }
}

/// The vault stored in one folder.
pub struct Vault {
    dir: PathBuf,
    kdf: KdfParams,
    unlocked: Option<UnlockedVault>,
}

impl Vault {
    /// A vault in `dir` that uses the default Argon2id parameters for new keys.
    pub fn new(dir: impl Into<PathBuf>) -> Vault {
        Vault::with_kdf(dir, KdfParams::DEFAULT)
    }

    /// A vault in `dir` that uses `kdf` for new keys.
    pub fn with_kdf(dir: impl Into<PathBuf>, kdf: KdfParams) -> Vault {
        Vault {
            dir: dir.into(),
            kdf,
            unlocked: None,
        }
    }

    /// The vault folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Path of the vault database.
    pub fn database_path(&self) -> PathBuf {
        self.dir.join(DATABASE_FILE)
    }

    /// Whether a vault has been created in the folder.
    pub fn exists(&self) -> bool {
        self.database_path().is_file()
    }

    /// Whether the vault is unlocked.
    pub fn is_unlocked(&self) -> bool {
        self.unlocked.is_some()
    }

    /// Current status.
    pub fn status(&self) -> VaultStatus {
        VaultStatus {
            exists: self.exists(),
            unlocked: self.is_unlocked(),
        }
    }

    /// Creates a new vault protected by `password` and leaves it unlocked.
    ///
    /// The database is written under a temporary name and renamed when
    /// complete, so an interrupted creation never leaves a half-made vault.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if the password breaks the policy.
    /// - [`AppError::VaultExists`] if the folder already holds a vault.
    /// - Storage, file system or cryptographic errors.
    pub fn create(&mut self, password: &str) -> AppResult<()> {
        validate_master_password(password)?;
        if self.exists() {
            return Err(AppError::VaultExists);
        }
        fs::create_dir_all(self.dir.join(FILES_DIR))?;

        let temporary = self.dir.join(format!("{DATABASE_FILE}.creating"));
        if temporary.exists() {
            fs::remove_file(&temporary)?;
        }
        let (header, key) = VaultHeader::create(password.as_bytes(), self.kdf)?;
        let written = db::create(&temporary)
            .and_then(|conn| db::set_meta(&conn, HEADER_META_KEY, &header.to_bytes()));
        if let Err(error) = written {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        fs::rename(&temporary, self.database_path())?;

        let conn = db::open(&self.database_path())?;
        self.unlocked = Some(UnlockedVault {
            dir: self.dir.clone(),
            conn,
            key,
        });
        Ok(())
    }

    /// Unlocks the vault with `password`.
    ///
    /// # Errors
    /// - [`AppError::VaultNotFound`] if there is no vault.
    /// - [`AppError::WrongPasswordOrCorrupt`] if the password is wrong.
    /// - [`AppError::Corrupt`] or [`AppError::UnsupportedVersion`] if the
    ///   vault cannot be read.
    pub fn unlock(&mut self, password: &str) -> AppResult<()> {
        self.lock();
        let conn = db::open(&self.database_path())?;
        let header = read_header(&conn)?;
        let key = header.unwrap_key(password.as_bytes())?;
        self.unlocked = Some(UnlockedVault {
            dir: self.dir.clone(),
            conn,
            key,
        });
        Ok(())
    }

    /// Locks the vault: closes the database and wipes the key from memory.
    /// Locking a locked vault does nothing.
    pub fn lock(&mut self) {
        self.unlocked = None;
    }

    /// Changes the master password. The vault key and the data stay the same;
    /// only the key header is replaced, in a single transaction.
    ///
    /// # Errors
    /// - [`AppError::Locked`] if the vault is locked.
    /// - [`AppError::WrongPasswordOrCorrupt`] if `current` is wrong.
    /// - [`AppError::InvalidInput`] if `new` breaks the policy or equals `current`.
    pub fn change_master_password(&mut self, current: &str, new: &str) -> AppResult<()> {
        let kdf = self.kdf;
        let unlocked = self.unlocked_mut()?;
        let header = read_header(&unlocked.conn)?;
        let verified = header.unwrap_key(current.as_bytes())?;
        if verified.as_bytes() != unlocked.key.as_bytes() {
            return Err(AppError::WrongPasswordOrCorrupt);
        }
        validate_master_password(new)?;
        if new == current {
            return Err(AppError::invalid(
                "the new master password must be different from the current one",
            ));
        }
        let new_header = VaultHeader::wrap(&unlocked.key, new.as_bytes(), kdf)?;
        let transaction = unlocked.conn.transaction()?;
        db::set_meta(&transaction, HEADER_META_KEY, &new_header.to_bytes())?;
        transaction.commit()?;
        Ok(())
    }

    /// The unlocked state, or [`AppError::Locked`].
    ///
    /// # Errors
    /// Returns [`AppError::Locked`] if the vault is locked.
    pub fn unlocked(&self) -> AppResult<&UnlockedVault> {
        self.unlocked.as_ref().ok_or(AppError::Locked)
    }

    /// The mutable unlocked state, or [`AppError::Locked`].
    ///
    /// # Errors
    /// Returns [`AppError::Locked`] if the vault is locked.
    pub fn unlocked_mut(&mut self) -> AppResult<&mut UnlockedVault> {
        self.unlocked.as_mut().ok_or(AppError::Locked)
    }
}

/// Reads the key header from the database.
pub(crate) fn read_header(conn: &Connection) -> AppResult<VaultHeader> {
    let bytes = db::get_meta(conn, HEADER_META_KEY)?
        .ok_or_else(|| AppError::Corrupt("missing vault header".into()))?;
    VaultHeader::from_bytes(&bytes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::vault::header::{self, tests::FAST_KDF};

    pub(crate) const PASSWORD: &str = "correct horse battery";

    /// A new, unlocked vault in a temporary folder.
    pub(crate) fn unlocked_vault() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let mut vault = Vault::with_kdf(dir.path().join("vault"), FAST_KDF);
        vault.create(PASSWORD).unwrap();
        (dir, vault)
    }

    #[test]
    fn new_folder_has_no_vault() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::with_kdf(dir.path(), FAST_KDF);
        assert_eq!(
            vault.status(),
            VaultStatus {
                exists: false,
                unlocked: false
            }
        );
    }

    #[test]
    fn create_leaves_the_vault_unlocked() {
        let (_dir, vault) = unlocked_vault();
        assert_eq!(
            vault.status(),
            VaultStatus {
                exists: true,
                unlocked: true
            }
        );
        assert!(vault.dir().join(FILES_DIR).is_dir());
        assert!(!vault.dir().join("vault.db.creating").exists());
    }

    #[test]
    fn create_enforces_the_password_policy() {
        let dir = tempfile::tempdir().unwrap();
        let mut vault = Vault::with_kdf(dir.path(), FAST_KDF);
        assert!(matches!(
            vault.create("short"),
            Err(AppError::InvalidInput(_))
        ));
        assert!(!vault.exists());
    }

    #[test]
    fn create_refuses_an_existing_vault() {
        let (_dir, mut vault) = unlocked_vault();
        assert_eq!(vault.create(PASSWORD).unwrap_err(), AppError::VaultExists);
    }

    #[test]
    fn create_replaces_a_stale_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("vault.db.creating"), b"leftover").unwrap();
        let mut vault = Vault::with_kdf(dir.path(), FAST_KDF);
        vault.create(PASSWORD).unwrap();
        assert!(vault.is_unlocked());
    }

    #[test]
    fn lock_and_unlock() {
        let (_dir, mut vault) = unlocked_vault();
        let key_before = *vault.unlocked().unwrap().key.as_bytes();
        vault.lock();
        assert!(!vault.is_unlocked());
        assert_eq!(vault.unlocked().err(), Some(AppError::Locked));
        vault.lock();
        vault.unlock(PASSWORD).unwrap();
        assert_eq!(vault.unlocked().unwrap().key.as_bytes(), &key_before);
    }

    #[test]
    fn unlock_with_wrong_password_stays_locked() {
        let (_dir, mut vault) = unlocked_vault();
        vault.lock();
        assert_eq!(
            vault.unlock("wrong password!!").unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn unlock_without_vault_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mut vault = Vault::with_kdf(dir.path(), FAST_KDF);
        assert_eq!(vault.unlock(PASSWORD).unwrap_err(), AppError::VaultNotFound);
    }

    #[test]
    fn unlock_without_header_is_corrupt() {
        let (_dir, mut vault) = unlocked_vault();
        vault
            .unlocked()
            .unwrap()
            .conn
            .execute("DELETE FROM meta", [])
            .unwrap();
        vault.lock();
        assert!(matches!(vault.unlock(PASSWORD), Err(AppError::Corrupt(_))));
    }

    #[test]
    fn change_master_password() {
        let (_dir, mut vault) = unlocked_vault();
        vault
            .change_master_password(PASSWORD, "a brand new password")
            .unwrap();
        vault.lock();
        assert_eq!(
            vault.unlock(PASSWORD).unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        vault.unlock("a brand new password").unwrap();
    }

    #[test]
    fn change_master_password_checks_the_current_one() {
        let (_dir, mut vault) = unlocked_vault();
        assert_eq!(
            vault
                .change_master_password("not the password", "a brand new password")
                .unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        vault.lock();
        vault.unlock(PASSWORD).unwrap();
    }

    #[test]
    fn change_master_password_enforces_the_policy() {
        let (_dir, mut vault) = unlocked_vault();
        assert!(matches!(
            vault.change_master_password(PASSWORD, "short"),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            vault.change_master_password(PASSWORD, PASSWORD),
            Err(AppError::InvalidInput(_))
        ));
    }

    #[test]
    fn change_master_password_needs_an_unlocked_vault() {
        let (_dir, mut vault) = unlocked_vault();
        vault.lock();
        assert_eq!(
            vault
                .change_master_password(PASSWORD, "a brand new password")
                .unwrap_err(),
            AppError::Locked
        );
    }

    /// Runs `sql` against the database of a locked vault.
    fn edit_database(vault: &Vault, sql: &str) {
        let conn = Connection::open(vault.database_path()).unwrap();
        conn.execute_batch(sql).unwrap();
    }

    /// Stored header with its format version replaced by `version`.
    fn header_with_version(vault: &Vault, version: u32) -> Vec<u8> {
        let mut bytes = db::get_meta(&vault.unlocked().unwrap().conn, HEADER_META_KEY)
            .unwrap()
            .unwrap();
        bytes[4..8].copy_from_slice(&version.to_le_bytes());
        bytes
    }

    #[test]
    fn paths_are_inside_the_vault_folder() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::with_kdf(dir.path().join("vault"), FAST_KDF);
        assert_eq!(vault.dir(), dir.path().join("vault"));
        assert_eq!(
            vault.database_path(),
            dir.path().join("vault").join("vault.db")
        );
    }

    #[test]
    fn a_failed_unlock_of_an_unlocked_vault_locks_it() {
        let (_dir, mut vault) = unlocked_vault();
        assert_eq!(
            vault.unlock("wrong password!!").unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn unlocking_an_unlocked_vault_keeps_it_unlocked() {
        let (_dir, mut vault) = unlocked_vault();
        vault.unlock(PASSWORD).unwrap();
        assert!(vault.is_unlocked());
    }

    #[test]
    fn unlock_rejects_a_newer_header_format() {
        let (_dir, mut vault) = unlocked_vault();
        let newer = header_with_version(&vault, header::FORMAT_VERSION + 1);
        db::set_meta(&vault.unlocked().unwrap().conn, HEADER_META_KEY, &newer).unwrap();
        vault.lock();
        assert_eq!(
            vault.unlock(PASSWORD).unwrap_err(),
            AppError::UnsupportedVersion
        );
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn unlock_rejects_a_newer_database_schema() {
        let (_dir, mut vault) = unlocked_vault();
        vault.lock();
        edit_database(
            &vault,
            &format!("PRAGMA user_version = {};", db::SCHEMA_VERSION + 1),
        );
        assert_eq!(
            vault.unlock(PASSWORD).unwrap_err(),
            AppError::UnsupportedVersion
        );
    }

    #[test]
    fn change_master_password_detects_a_missing_header() {
        let (_dir, mut vault) = unlocked_vault();
        vault
            .unlocked()
            .unwrap()
            .conn
            .execute("DELETE FROM meta", [])
            .unwrap();
        assert!(matches!(
            vault.change_master_password(PASSWORD, "a brand new password"),
            Err(AppError::Corrupt(_))
        ));
    }

    #[test]
    fn change_master_password_refuses_a_header_of_another_key() {
        // A header swapped in from another vault opens with the same password
        // but wraps a different key: it must not be rewrapped.
        let (_dir, mut vault) = unlocked_vault();
        let (_other_dir, other) = unlocked_vault();
        let foreign = db::get_meta(&other.unlocked().unwrap().conn, HEADER_META_KEY)
            .unwrap()
            .unwrap();
        db::set_meta(&vault.unlocked().unwrap().conn, HEADER_META_KEY, &foreign).unwrap();
        assert_eq!(
            vault
                .change_master_password(PASSWORD, "a brand new password")
                .unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        let stored = db::get_meta(&vault.unlocked().unwrap().conn, HEADER_META_KEY)
            .unwrap()
            .unwrap();
        assert_eq!(stored, foreign);
    }
}
