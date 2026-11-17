//! Encrypted files: import, list, export and delete.
//!
//! Each file has two parts:
//!
//! - A row in the `files` table with its description (name, size, import
//!   time) as JSON encrypted with the vault key, bound to the file id.
//! - A blob `files/<id>.lwf` in the vault folder: the magic `LWF1` followed
//!   by the content encrypted with STREAM (see [`crate::crypto::stream`]),
//!   bound to the file id.
//!
//! The blob name is the random id, so neither the name nor the type of the
//! file is visible on disk. Writes go to a temporary file that is renamed
//! when complete, so an interrupted operation never leaves a partial file.

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use super::entries::check_id;
use super::now_millis;
use super::session::UnlockedVault;
use crate::crypto::random::{random_array, random_id, to_hex};
use crate::crypto::{aead, stream};
use crate::error::{AppError, AppResult};
use crate::storage::db::{self, Table};

/// Magic bytes at the start of every encrypted file blob.
pub(crate) const BLOB_MAGIC: &[u8; 4] = b"LWF1";
/// Extension of the encrypted file blobs.
pub(crate) const BLOB_EXTENSION: &str = "lwf";
/// Maximum length of a file name, in characters.
pub const MAX_FILE_NAME_CHARS: usize = 255;

/// A stored file as listed in the interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSummary {
    /// Random identifier.
    pub id: String,
    /// Original file name.
    pub name: String,
    /// Size of the original content, in bytes.
    pub size: u64,
    /// Import time, in milliseconds since the Unix epoch.
    pub created_at: i64,
}

/// The encrypted description of a file.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileMeta {
    name: String,
    size: u64,
    created_at: i64,
}

impl UnlockedVault {
    /// Lists the stored files, sorted by name (case-insensitive).
    ///
    /// # Errors
    /// Returns [`AppError::Corrupt`] if a description fails to decrypt.
    pub fn list_files(&self) -> AppResult<Vec<FileSummary>> {
        let mut files = db::list_records(&self.conn, Table::Files)?
            .into_iter()
            .map(|(id, data)| self.decrypt_meta(id, &data))
            .collect::<AppResult<Vec<_>>>()?;
        files.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(files)
    }

    /// Reads the description of one file.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `id` is malformed.
    /// - [`AppError::NotFound`] if there is no such file.
    pub fn get_file(&self, id: &str) -> AppResult<FileSummary> {
        check_id(id)?;
        let data = db::get_record(&self.conn, Table::Files, id)?;
        self.decrypt_meta(id.to_owned(), &data)
    }

    /// Encrypts the file at `source` into the vault.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `source` is not a regular file outside
    ///   the vault folder, or its name is not acceptable.
    /// - File system, storage or cryptographic errors. On error nothing is
    ///   left in the vault.
    pub fn import_file(&self, source: &Path) -> AppResult<FileSummary> {
        let name = file_name_of(source)?;
        if !fs::metadata(source)?.is_file() {
            return Err(AppError::invalid("only regular files can be added"));
        }
        self.check_outside_vault(source)?;

        let id = random_id()?;
        let blob = self.blob_path(&id);
        let size = write_atomically(&blob, |writer| {
            let mut reader = BufReader::new(File::open(source)?);
            writer.write_all(BLOB_MAGIC)?;
            Ok(stream::encrypt(
                &self.key,
                &blob_aad(&id),
                &mut reader,
                writer,
            )?)
        })?;

        let meta = FileMeta {
            name,
            size,
            created_at: now_millis(),
        };
        let stored = self
            .encrypt_meta(&id, &meta)
            .and_then(|sealed| db::insert_record(&self.conn, Table::Files, &id, &sealed));
        if let Err(error) = stored {
            let _ = fs::remove_file(&blob);
            return Err(error);
        }
        Ok(FileSummary {
            id,
            name: meta.name,
            size: meta.size,
            created_at: meta.created_at,
        })
    }

    /// Decrypts a stored file to `destination`.
    ///
    /// The content is written to a temporary file next to `destination` and
    /// renamed only once it has been fully authenticated.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `id` is malformed, `destination` is
    ///   inside the vault folder, or it exists and `overwrite` is false.
    /// - [`AppError::NotFound`] if there is no such file.
    /// - [`AppError::Corrupt`] if the blob is missing or fails to decrypt.
    pub fn export_file(&self, id: &str, destination: &Path, overwrite: bool) -> AppResult<()> {
        let file = self.get_file(id)?;
        if destination.exists() && !overwrite {
            return Err(AppError::invalid("a file with that name already exists"));
        }
        if destination.is_dir() {
            return Err(AppError::invalid("the destination is a folder"));
        }
        self.check_outside_vault(destination)?;

        let blob = self.blob_path(id);
        let mut reader = match File::open(&blob) {
            Ok(file) => BufReader::new(file),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(AppError::Corrupt("the encrypted file is missing".into()));
            }
            Err(error) => return Err(error.into()),
        };
        let written = write_atomically(destination, |writer| {
            let mut magic = [0u8; 4];
            reader
                .read_exact(&mut magic)
                .map_err(|_| AppError::Corrupt("the encrypted file is truncated".into()))?;
            if &magic != BLOB_MAGIC {
                return Err(AppError::Corrupt(
                    "the encrypted file has an unknown format".into(),
                ));
            }
            stream::decrypt(&self.key, &blob_aad(id), &mut reader, writer).map_err(|error| {
                match error {
                    crate::crypto::CryptoError::Io(message) => AppError::Io(message),
                    _ => AppError::Corrupt("the encrypted file failed to decrypt".into()),
                }
            })
        })?;
        if written != file.size {
            // Authenticated content cannot change size; this is defensive.
            let _ = fs::remove_file(destination);
            return Err(AppError::Corrupt(
                "the decrypted file has the wrong size".into(),
            ));
        }
        Ok(())
    }

    /// Deletes a stored file: its encrypted blob and its description.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `id` is malformed.
    /// - [`AppError::NotFound`] if there is no such file.
    pub fn delete_file(&self, id: &str) -> AppResult<()> {
        self.get_file(id)?;
        match fs::remove_file(self.blob_path(id)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        db::delete_record(&self.conn, Table::Files, id)
    }

    fn blob_path(&self, id: &str) -> PathBuf {
        self.files_dir().join(format!("{id}.{BLOB_EXTENSION}"))
    }

    /// Refuses paths inside the vault folder, so the vault's own files are
    /// never imported or overwritten.
    pub(crate) fn check_outside_vault(&self, path: &Path) -> AppResult<()> {
        let vault = fs::canonicalize(&self.dir)?;
        let target = match fs::canonicalize(path) {
            Ok(resolved) => resolved,
            Err(_) => {
                let parent = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .ok_or_else(|| AppError::invalid("the destination folder does not exist"))?;
                fs::canonicalize(parent)
                    .map_err(|_| AppError::invalid("the destination folder does not exist"))?
            }
        };
        if target.starts_with(&vault) {
            return Err(AppError::invalid(
                "files inside the vault folder cannot be used",
            ));
        }
        Ok(())
    }

    fn encrypt_meta(&self, id: &str, meta: &FileMeta) -> AppResult<Vec<u8>> {
        let json = Zeroizing::new(
            serde_json::to_vec(meta).map_err(|_| AppError::invalid("the file cannot be saved"))?,
        );
        Ok(aead::seal(&self.key, &json, &meta_aad(id))?)
    }

    fn decrypt_meta(&self, id: String, data: &[u8]) -> AppResult<FileSummary> {
        let json = aead::open(&self.key, data, &meta_aad(&id))
            .map_err(|_| AppError::Corrupt("a file description failed to decrypt".into()))?;
        let meta: FileMeta = serde_json::from_slice(&json)
            .map_err(|_| AppError::Corrupt("a file description has an invalid format".into()))?;
        Ok(FileSummary {
            id,
            name: meta.name,
            size: meta.size,
            created_at: meta.created_at,
        })
    }
}

/// Associated data of a file description row.
fn meta_aad(id: &str) -> Vec<u8> {
    format!("lockwell:file:v1:{id}").into_bytes()
}

/// Associated data of a file blob.
fn blob_aad(id: &str) -> Vec<u8> {
    format!("lockwell:file-data:v1:{id}").into_bytes()
}

/// The validated name of the file at `path`.
fn file_name_of(path: &Path) -> AppResult<String> {
    let name = path
        .file_name()
        .ok_or_else(|| AppError::invalid("the path has no file name"))?
        .to_str()
        .ok_or_else(|| AppError::invalid("the file name is not valid Unicode"))?;
    validate_file_name(name)?;
    Ok(name.to_owned())
}

/// Checks a file name: not empty, not too long, no control characters.
///
/// # Errors
/// Returns [`AppError::InvalidInput`] explaining the problem.
pub fn validate_file_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::invalid("the file name is empty"));
    }
    if name.chars().count() > MAX_FILE_NAME_CHARS {
        return Err(AppError::invalid(format!(
            "the file name must have at most {MAX_FILE_NAME_CHARS} characters"
        )));
    }
    if name
        .chars()
        .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        return Err(AppError::invalid(
            "the file name contains invalid characters",
        ));
    }
    Ok(())
}

/// Writes `target` through a temporary file in the same folder that is
/// renamed over `target` only if `write` succeeds; otherwise it is removed.
/// The temporary name is `.lockwell-<16 hex digits>.tmp`, so it is short
/// enough whatever the length of `target`'s name.
pub(crate) fn write_atomically<T>(
    target: &Path,
    write: impl FnOnce(&mut BufWriter<File>) -> AppResult<T>,
) -> AppResult<T> {
    let suffix = to_hex(&random_array::<8>()?);
    // A fixed-length name: one derived from `target` could exceed the file
    // system's name limit when the target's name is long.
    let temporary = target.with_file_name(format!(".lockwell-{suffix}.tmp"));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let mut writer = BufWriter::new(file);
    let result = write(&mut writer).and_then(|value| {
        let file = writer
            .into_inner()
            .map_err(|error| AppError::from(error.into_error()))?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, target)?;
        Ok(value)
    });
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::stream::CHUNK_LEN;
    use crate::vault::session::tests::{PASSWORD, unlocked_vault};

    /// Writes `content` to a new file named `name` in a fresh folder.
    fn source_file(name: &str, content: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn data(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i * 7 % 253) as u8).collect()
    }

    fn round_trip(content: &[u8]) {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("report.pdf", content);
        let summary = unlocked.import_file(&source).unwrap();
        assert_eq!(summary.name, "report.pdf");
        assert_eq!(summary.size, content.len() as u64);

        let destination = src_dir.path().join("restored.pdf");
        unlocked
            .export_file(&summary.id, &destination, false)
            .unwrap();
        assert_eq!(fs::read(&destination).unwrap(), content);
    }

    #[test]
    fn round_trips_an_empty_file() {
        round_trip(b"");
    }

    #[test]
    fn round_trips_a_small_file() {
        round_trip(b"hello");
    }

    #[test]
    fn round_trips_exact_chunk_multiples() {
        round_trip(&data(CHUNK_LEN));
        round_trip(&data(2 * CHUNK_LEN));
    }

    #[test]
    fn round_trips_a_large_file() {
        round_trip(&data(5 * CHUNK_LEN + 123));
    }

    #[test]
    fn names_and_content_are_not_visible_on_disk() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (_src, source) = source_file("tax return 2026.txt", b"very visible text");
        let summary = unlocked.import_file(&source).unwrap();

        let entries: Vec<_> = fs::read_dir(unlocked.files_dir())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(entries, vec![format!("{}.lwf", summary.id)]);
        let blob = fs::read(unlocked.blob_path(&summary.id)).unwrap();
        assert!(
            !blob
                .windows(17)
                .any(|window| window == b"very visible text")
        );
        let row = db::get_record(&unlocked.conn, Table::Files, &summary.id).unwrap();
        assert!(!row.windows(10).any(|window| window == b"tax return"));
    }

    #[test]
    fn keeps_unicode_names() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (_src, source) = source_file("contraseñas 日本.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        assert_eq!(
            unlocked.get_file(&summary.id).unwrap().name,
            "contraseñas 日本.txt"
        );
    }

    #[test]
    fn lists_sorted_by_name() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        for name in ["b.txt", "A.txt", "c.txt"] {
            let (_src, source) = source_file(name, b"x");
            unlocked.import_file(&source).unwrap();
        }
        let names: Vec<_> = unlocked
            .list_files()
            .unwrap()
            .into_iter()
            .map(|file| file.name)
            .collect();
        assert_eq!(names, ["A.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn export_refuses_to_overwrite_without_permission() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("a.txt", b"new content");
        let summary = unlocked.import_file(&source).unwrap();
        let destination = src_dir.path().join("existing.txt");
        fs::write(&destination, b"keep me").unwrap();

        assert!(matches!(
            unlocked.export_file(&summary.id, &destination, false),
            Err(AppError::InvalidInput(_))
        ));
        assert_eq!(fs::read(&destination).unwrap(), b"keep me");

        unlocked
            .export_file(&summary.id, &destination, true)
            .unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"new content");
    }

    #[test]
    fn a_damaged_blob_is_detected_and_nothing_is_written() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("a.txt", &data(1000));
        let summary = unlocked.import_file(&source).unwrap();
        let blob = unlocked.blob_path(&summary.id);
        let mut bytes = fs::read(&blob).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        fs::write(&blob, bytes).unwrap();

        let destination = src_dir.path().join("out.txt");
        assert!(matches!(
            unlocked.export_file(&summary.id, &destination, false),
            Err(AppError::Corrupt(_))
        ));
        assert!(!destination.exists());
        let leftovers = fs::read_dir(src_dir.path()).unwrap().count();
        assert_eq!(leftovers, 1, "only the source file should remain");
    }

    #[test]
    fn swapped_blobs_are_detected() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, a) = source_file("a.txt", b"first");
        let b = src_dir.path().join("b.txt");
        fs::write(&b, b"second").unwrap();
        let first = unlocked.import_file(&a).unwrap();
        let second = unlocked.import_file(&b).unwrap();
        fs::copy(
            unlocked.blob_path(&first.id),
            unlocked.blob_path(&second.id),
        )
        .unwrap();
        assert!(matches!(
            unlocked.export_file(&second.id, &src_dir.path().join("out"), false),
            Err(AppError::Corrupt(_))
        ));
    }

    #[test]
    fn a_missing_or_foreign_blob_is_corrupt() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        let blob = unlocked.blob_path(&summary.id);

        fs::write(&blob, b"XXXXsomething").unwrap();
        assert!(matches!(
            unlocked.export_file(&summary.id, &src_dir.path().join("o1"), false),
            Err(AppError::Corrupt(_))
        ));
        fs::remove_file(&blob).unwrap();
        assert!(matches!(
            unlocked.export_file(&summary.id, &src_dir.path().join("o2"), false),
            Err(AppError::Corrupt(_))
        ));
    }

    #[test]
    fn delete_removes_blob_and_row() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (_src, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        unlocked.delete_file(&summary.id).unwrap();
        assert!(!unlocked.blob_path(&summary.id).exists());
        assert!(unlocked.list_files().unwrap().is_empty());
        assert_eq!(
            unlocked.delete_file(&summary.id).unwrap_err(),
            AppError::NotFound("file")
        );
    }

    #[test]
    fn rejects_bad_sources() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let folder = tempfile::tempdir().unwrap();
        assert!(matches!(
            unlocked.import_file(folder.path()),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            unlocked.import_file(&folder.path().join("missing.txt")),
            Err(AppError::Io(_))
        ));
        let inside = vault.dir().join("vault.db");
        assert!(matches!(
            unlocked.import_file(&inside),
            Err(AppError::InvalidInput(_))
        ));
        assert!(unlocked.list_files().unwrap().is_empty());
    }

    #[test]
    fn rejects_destinations_inside_the_vault() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (_src, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        let database = vault.dir().join("vault.db");
        assert!(matches!(
            unlocked.export_file(&summary.id, &database, true),
            Err(AppError::InvalidInput(_))
        ));
        assert!(matches!(
            unlocked.export_file(
                &summary.id,
                &vault.dir().join("files").join("new.txt"),
                true
            ),
            Err(AppError::InvalidInput(_))
        ));
    }

    #[test]
    fn rejects_malformed_ids() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let out = tempfile::tempdir().unwrap();
        for id in ["", "../vault", &"z".repeat(32)] {
            assert!(matches!(
                unlocked.get_file(id),
                Err(AppError::InvalidInput(_))
            ));
            assert!(matches!(
                unlocked.delete_file(id),
                Err(AppError::InvalidInput(_))
            ));
            assert!(matches!(
                unlocked.export_file(id, &out.path().join("x"), false),
                Err(AppError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn file_name_rules() {
        assert!(validate_file_name("notes.txt").is_ok());
        assert!(validate_file_name("").is_err());
        assert!(validate_file_name("   ").is_err());
        assert!(validate_file_name("a\u{0}b").is_err());
        assert!(validate_file_name("a/b").is_err());
        assert!(validate_file_name("a\\b").is_err());
        assert!(validate_file_name(&"n".repeat(MAX_FILE_NAME_CHARS)).is_ok());
        assert!(validate_file_name(&"n".repeat(MAX_FILE_NAME_CHARS + 1)).is_err());
    }

    #[test]
    fn files_survive_a_password_change() {
        let (_vault_dir, mut vault) = unlocked_vault();
        let (src_dir, source) = source_file("a.txt", b"kept");
        let id = vault.unlocked().unwrap().import_file(&source).unwrap().id;
        vault
            .change_master_password(PASSWORD, "a brand new password")
            .unwrap();
        vault.lock();
        vault.unlock("a brand new password").unwrap();
        let destination = src_dir.path().join("out.txt");
        vault
            .unlocked()
            .unwrap()
            .export_file(&id, &destination, false)
            .unwrap();
        assert_eq!(fs::read(destination).unwrap(), b"kept");
    }

    #[test]
    fn export_refuses_a_folder_even_when_overwriting() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        assert_eq!(
            unlocked
                .export_file(&summary.id, src_dir.path(), true)
                .unwrap_err(),
            AppError::invalid("the destination is a folder")
        );
    }

    #[test]
    fn export_needs_an_existing_destination_folder() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        let destination = src_dir.path().join("missing").join("out.txt");
        assert_eq!(
            unlocked
                .export_file(&summary.id, &destination, false)
                .unwrap_err(),
            AppError::invalid("the destination folder does not exist")
        );
    }

    #[test]
    fn a_truncated_blob_is_corrupt() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (src_dir, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        fs::write(unlocked.blob_path(&summary.id), b"LW").unwrap();
        let destination = src_dir.path().join("out.txt");
        assert_eq!(
            unlocked
                .export_file(&summary.id, &destination, false)
                .unwrap_err(),
            AppError::Corrupt("the encrypted file is truncated".into())
        );
        assert!(!destination.exists());
    }

    #[test]
    fn delete_succeeds_when_the_blob_is_already_gone() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (_src, source) = source_file("a.txt", b"x");
        let summary = unlocked.import_file(&source).unwrap();
        fs::remove_file(unlocked.blob_path(&summary.id)).unwrap();
        unlocked.delete_file(&summary.id).unwrap();
        assert!(unlocked.list_files().unwrap().is_empty());
    }

    #[test]
    fn rejects_source_names_that_break_the_rules() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let folder = tempfile::tempdir().unwrap();
        // Checked before the file is opened, so these paths need not exist.
        assert_eq!(
            unlocked
                .import_file(&folder.path().join("bell\u{7}.txt"))
                .unwrap_err(),
            AppError::invalid("the file name contains invalid characters")
        );
        assert_eq!(
            unlocked.import_file(&folder.path().join("..")).unwrap_err(),
            AppError::invalid("the path has no file name")
        );
        assert!(unlocked.list_files().unwrap().is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn rejects_source_names_that_are_not_unicode() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let folder = tempfile::tempdir().unwrap();
        // A lone UTF-16 surrogate is a valid Windows name but not Unicode.
        let name = OsString::from_wide(&[0x61, 0xD800, 0x62]);
        assert_eq!(
            unlocked.import_file(&folder.path().join(name)).unwrap_err(),
            AppError::invalid("the file name is not valid Unicode")
        );
    }

    #[test]
    fn tampered_or_malformed_descriptions_are_corrupt() {
        let (_vault_dir, vault) = unlocked_vault();
        let unlocked = vault.unlocked().unwrap();
        let (_src, source) = source_file("a.txt", b"x");
        let id = unlocked.import_file(&source).unwrap().id;

        let mut sealed = db::get_record(&unlocked.conn, Table::Files, &id).unwrap();
        sealed[0] ^= 1;
        db::update_record(&unlocked.conn, Table::Files, &id, &sealed).unwrap();
        let tampered = AppError::Corrupt("a file description failed to decrypt".into());
        assert_eq!(unlocked.get_file(&id).unwrap_err(), tampered);

        let malformed = aead::seal(&unlocked.key, b"[]", &meta_aad(&id)).unwrap();
        db::update_record(&unlocked.conn, Table::Files, &id, &malformed).unwrap();
        let invalid = AppError::Corrupt("a file description has an invalid format".into());
        assert_eq!(unlocked.get_file(&id).unwrap_err(), invalid);
        assert_eq!(unlocked.list_files().unwrap_err(), invalid);
    }

    #[test]
    fn atomic_writes_leave_nothing_behind_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        fs::write(&target, b"original").unwrap();

        let result: AppResult<()> = write_atomically(&target, |writer| {
            writer.write_all(b"partial")?;
            Err(AppError::Corrupt("stop".into()))
        });

        assert_eq!(result.unwrap_err(), AppError::Corrupt("stop".into()));
        assert_eq!(fs::read(&target).unwrap(), b"original");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_writes_work_for_names_at_the_file_name_limit() {
        // The temporary name must fit in the file system's name limit even
        // when the target name is as long as a name can be.
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a".repeat(MAX_FILE_NAME_CHARS));

        write_atomically(&target, |writer| {
            writer.write_all(b"long")?;
            Ok(())
        })
        .unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"long");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn atomic_writes_replace_the_target_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.txt");
        fs::write(&target, b"original").unwrap();

        let value = write_atomically(&target, |writer| {
            writer.write_all(b"replaced")?;
            Ok(8)
        })
        .unwrap();

        assert_eq!(value, 8);
        assert_eq!(fs::read(&target).unwrap(), b"replaced");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
