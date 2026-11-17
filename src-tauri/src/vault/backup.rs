//! Encrypted backup of the whole vault, and restoring it.
//!
//! A backup file (`.lwbackup`) is:
//!
//! ```text
//! magic "LWBK" (4) | format version u32 (4) | vault key header (108)
//! | STREAM-encrypted contents
//! ```
//!
//! The key header is the vault's own (Argon2id parameters, salt and wrapped
//! vault key), so a backup opens with the master password the vault had when
//! the backup was made. The contents are encrypted with the vault key and
//! authenticated together with the clear prefix. Decrypted, they are a
//! simple container:
//!
//! ```text
//! "LWBC" | record* | 0x00
//! record = kind u8 (1 database, 2 file blob) | name length u16 | name
//!          | data length u64 | data
//! ```
//!
//! The database is a consistent snapshot (`VACUUM INTO`); file blobs are
//! copied as they are, already encrypted. Restoring decrypts and checks the
//! whole backup in a staging folder next to the vault, and only then swaps
//! it in, so a failed restore never touches the current vault.

use std::collections::{HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};

use super::files::{
    BLOB_EXTENSION, BLOB_MAGIC, blob_file_name, check_outside_folder, write_atomically,
};
use super::header::{HEADER_LEN, VaultHeader};
use super::session::{DATABASE_FILE, FILES_DIR, UnlockedVault, Vault, read_header};
use super::{decrypt_failure, random_suffix};
use crate::crypto::keys::SecretKey;
use crate::crypto::random::is_valid_id;
use crate::crypto::stream;
use crate::error::{AppError, AppResult};
use crate::storage::db::{self, Table};

/// Extension of backup files.
pub const BACKUP_EXTENSION: &str = "lwbackup";
const BACKUP_MAGIC: &[u8; 4] = b"LWBK";
/// Version of the backup format.
pub const BACKUP_FORMAT_VERSION: u32 = 1;
const PREFIX_LEN: usize = 8 + HEADER_LEN;
const CONTAINER_MAGIC: &[u8; 4] = b"LWBC";
const KIND_END: u8 = 0;
const KIND_DATABASE: u8 = 1;
const KIND_BLOB: u8 = 2;

impl UnlockedVault {
    /// Writes an encrypted backup of the whole vault to `destination`.
    ///
    /// # Errors
    /// - [`AppError::InvalidInput`] if `destination` is inside the vault.
    /// - [`AppError::Corrupt`] if a stored file is missing.
    /// - File system, storage or cryptographic errors. On error no backup
    ///   file is left behind.
    pub fn create_backup(&self, destination: &Path) -> AppResult<()> {
        self.check_outside_vault(destination)?;
        let prefix = backup_prefix(&read_header(&self.conn)?);

        let snapshot = self
            .dir
            .join(format!("{DATABASE_FILE}.snapshot-{}", random_suffix()?));
        let result = self.snapshot_database(&snapshot).and_then(|()| {
            let mut contents = self.contents_reader(&snapshot)?;
            write_atomically(destination, |writer| {
                writer.write_all(&prefix)?;
                stream::encrypt(&self.key, &backup_aad(&prefix), &mut contents, writer)?;
                Ok(())
            })
        });
        let _ = fs::remove_file(&snapshot);
        result
    }

    fn snapshot_database(&self, snapshot: &Path) -> AppResult<()> {
        let path = snapshot
            .to_str()
            .ok_or_else(|| AppError::invalid("the vault folder path is not valid Unicode"))?;
        self.conn.execute("VACUUM INTO ?1", [path])?;
        Ok(())
    }

    /// The decrypted container, read lazily from the snapshot and the blobs.
    fn contents_reader(&self, snapshot: &Path) -> AppResult<Concat> {
        let mut parts: VecDeque<Box<dyn Read + Send>> = VecDeque::new();
        parts.push_back(Box::new(Cursor::new(CONTAINER_MAGIC.to_vec())));
        add_record(&mut parts, KIND_DATABASE, DATABASE_FILE, snapshot)?;
        for (id, _) in db::list_records(&self.conn, Table::Files)? {
            let name = blob_file_name(&id);
            let blob = self.files_dir().join(&name);
            if !blob.is_file() {
                return Err(AppError::Corrupt("a stored file is missing".into()));
            }
            add_record(&mut parts, KIND_BLOB, &name, &blob)?;
        }
        parts.push_back(Box::new(Cursor::new(vec![KIND_END])));
        Ok(Concat { parts })
    }
}

impl Vault {
    /// Replaces the vault with the backup at `backup`, which must open with
    /// `password` (the master password when the backup was made). The vault
    /// is left locked.
    ///
    /// # Errors
    /// - [`AppError::WrongPasswordOrCorrupt`] if the password is wrong.
    /// - [`AppError::Corrupt`] if the file is not a valid, intact backup.
    /// - [`AppError::UnsupportedVersion`] for a backup from a newer version.
    /// - [`AppError::InvalidInput`] if `backup` is inside the vault folder.
    ///
    /// On any error the current vault is left as it was.
    pub fn restore_backup(&mut self, backup: &Path, password: &str) -> AppResult<()> {
        if !fs::metadata(backup)?.is_file() {
            return Err(AppError::invalid("the backup must be a file"));
        }
        if self.dir().exists() {
            check_outside_folder(self.dir(), backup)?;
        }

        let mut reader = BufReader::new(File::open(backup)?);
        let mut prefix = [0u8; PREFIX_LEN];
        reader.read_exact(&mut prefix).map_err(|_| not_a_backup())?;
        let header = parse_prefix(&prefix)?;
        let key = header.unwrap_key(password.as_bytes())?;

        let (parent, name) = self.parent_and_name()?;
        fs::create_dir_all(&parent)?;
        let staging = parent.join(format!("{name}.restoring-{}", random_suffix()?));
        let staged = stage_backup(&staging, &mut reader, &prefix, &header, &key);
        if let Err(error) = staged {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }

        self.lock();
        let swapped = self.swap_in(&staging, &parent, &name);
        if swapped.is_err() {
            let _ = fs::remove_dir_all(&staging);
        }
        swapped
    }

    fn parent_and_name(&self) -> AppResult<(PathBuf, String)> {
        let parent = self
            .dir()
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| AppError::invalid("the vault folder has no parent folder"))?;
        let name = self
            .dir()
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| AppError::invalid("the vault folder name is not valid"))?;
        Ok((parent.to_path_buf(), name.to_owned()))
    }

    /// Moves `staging` into place, keeping the old vault until it succeeds.
    fn swap_in(&self, staging: &Path, parent: &Path, name: &str) -> AppResult<()> {
        if !self.dir().exists() {
            fs::rename(staging, self.dir())?;
            return Ok(());
        }
        let previous = parent.join(format!("{name}.replaced-{}", random_suffix()?));
        fs::rename(self.dir(), &previous)?;
        if let Err(error) = fs::rename(staging, self.dir()) {
            let _ = fs::rename(&previous, self.dir());
            return Err(error.into());
        }
        // The previous vault holds only encrypted data; failing to remove it
        // does not affect the restored vault.
        let _ = fs::remove_dir_all(&previous);
        Ok(())
    }
}

/// Decrypts the backup into `staging` and checks everything in it.
fn stage_backup(
    staging: &Path,
    reader: &mut impl Read,
    prefix: &[u8],
    header: &VaultHeader,
    key: &SecretKey,
) -> AppResult<()> {
    fs::create_dir_all(staging.join(FILES_DIR))?;
    let contents_path = staging.join("contents.tmp");
    {
        let mut contents = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&contents_path)?,
        );
        stream::decrypt(key, &backup_aad(prefix), reader, &mut contents)
            .map_err(|error| decrypt_failure(error, "the backup is damaged or incomplete"))?;
        contents.flush()?;
    }
    unpack_contents(&mut BufReader::new(File::open(&contents_path)?), staging)?;
    fs::remove_file(&contents_path)?;
    check_staged_vault(staging, header, key)
}

/// Writes the records of the container into `staging`.
fn unpack_contents(reader: &mut impl Read, staging: &Path) -> AppResult<()> {
    let mut magic = [0u8; 4];
    read_exact(reader, &mut magic)?;
    if &magic != CONTAINER_MAGIC {
        return Err(corrupt("unknown backup contents"));
    }
    let mut has_database = false;
    let mut blobs = HashSet::new();
    loop {
        let kind = read_u8(reader)?;
        if kind == KIND_END {
            break;
        }
        let name_len = usize::from(read_u16(reader)?);
        let mut name_bytes = vec![0u8; name_len];
        read_exact(reader, &mut name_bytes)?;
        let name = String::from_utf8(name_bytes).map_err(|_| corrupt("invalid name"))?;
        let target = match kind {
            KIND_DATABASE if name == DATABASE_FILE && !has_database => {
                has_database = true;
                staging.join(DATABASE_FILE)
            }
            KIND_BLOB if is_blob_name(&name) && blobs.insert(name.clone()) => {
                staging.join(FILES_DIR).join(&name)
            }
            _ => return Err(corrupt("unexpected record")),
        };
        let len = read_u64(reader)?;
        let mut file = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)?,
        );
        if io::copy(&mut reader.take(len), &mut file)? != len {
            return Err(corrupt("the backup is truncated"));
        }
        file.flush()?;
    }
    let mut rest = [0u8; 1];
    if reader.read(&mut rest)? != 0 {
        return Err(corrupt("unexpected data after the end"));
    }
    if !has_database {
        return Err(corrupt("the database is missing"));
    }
    Ok(())
}

/// Opens the staged vault with `key` and checks every record and file.
fn check_staged_vault(staging: &Path, header: &VaultHeader, key: &SecretKey) -> AppResult<()> {
    let conn = db::open(&staging.join(DATABASE_FILE))?;
    if read_header(&conn)?.to_bytes() != header.to_bytes() {
        return Err(corrupt("the backup header does not match its database"));
    }
    let staged = UnlockedVault {
        dir: staging.to_path_buf(),
        conn,
        key: key.clone(),
    };
    staged.list_entries()?;
    for file in staged.list_files()? {
        let blob = staged.files_dir().join(blob_file_name(&file.id));
        let mut magic = [0u8; 4];
        File::open(&blob)
            .and_then(|mut opened| opened.read_exact(&mut magic))
            .map_err(|_| corrupt("a file is missing from the backup"))?;
        if &magic != BLOB_MAGIC {
            return Err(corrupt("a file in the backup has an unknown format"));
        }
    }
    Ok(())
}

fn backup_prefix(header: &VaultHeader) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(PREFIX_LEN);
    prefix.extend_from_slice(BACKUP_MAGIC);
    prefix.extend_from_slice(&BACKUP_FORMAT_VERSION.to_le_bytes());
    prefix.extend_from_slice(&header.to_bytes());
    prefix
}

fn parse_prefix(prefix: &[u8; PREFIX_LEN]) -> AppResult<VaultHeader> {
    if &prefix[..4] != BACKUP_MAGIC {
        return Err(not_a_backup());
    }
    let mut version = [0u8; 4];
    version.copy_from_slice(&prefix[4..8]);
    let version = u32::from_le_bytes(version);
    if version > BACKUP_FORMAT_VERSION {
        return Err(AppError::UnsupportedVersion);
    }
    if version != BACKUP_FORMAT_VERSION {
        return Err(not_a_backup());
    }
    VaultHeader::from_bytes(&prefix[8..])
}

/// Associated data of the encrypted contents: the whole clear prefix.
fn backup_aad(prefix: &[u8]) -> Vec<u8> {
    let mut aad = b"lockwell:backup:v1:".to_vec();
    aad.extend_from_slice(prefix);
    aad
}

fn add_record(
    parts: &mut VecDeque<Box<dyn Read + Send>>,
    kind: u8,
    name: &str,
    path: &Path,
) -> AppResult<()> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let name_len =
        u16::try_from(name.len()).map_err(|_| AppError::invalid("a name is too long"))?;
    let mut record = vec![kind];
    record.extend_from_slice(&name_len.to_le_bytes());
    record.extend_from_slice(name.as_bytes());
    record.extend_from_slice(&len.to_le_bytes());
    parts.push_back(Box::new(Cursor::new(record)));
    // `take` guards against the file growing while it is read.
    parts.push_back(Box::new(BufReader::new(file).take(len)));
    Ok(())
}

/// Reads several readers one after the other.
struct Concat {
    parts: VecDeque<Box<dyn Read + Send>>,
}

impl Read for Concat {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        while let Some(part) = self.parts.front_mut() {
            let read = part.read(buffer)?;
            if read > 0 || buffer.is_empty() {
                return Ok(read);
            }
            self.parts.pop_front();
        }
        Ok(0)
    }
}

fn is_blob_name(name: &str) -> bool {
    name.strip_suffix(&format!(".{BLOB_EXTENSION}"))
        .is_some_and(is_valid_id)
}

fn read_exact(reader: &mut impl Read, buffer: &mut [u8]) -> AppResult<()> {
    reader
        .read_exact(buffer)
        .map_err(|_| corrupt("the backup is truncated"))
}

fn read_u8(reader: &mut impl Read) -> AppResult<u8> {
    let mut bytes = [0u8; 1];
    read_exact(reader, &mut bytes)?;
    Ok(bytes[0])
}

fn read_u16(reader: &mut impl Read) -> AppResult<u16> {
    let mut bytes = [0u8; 2];
    read_exact(reader, &mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> AppResult<u64> {
    let mut bytes = [0u8; 8];
    read_exact(reader, &mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn corrupt(detail: &str) -> AppError {
    AppError::Corrupt(format!("invalid backup: {detail}"))
}

fn not_a_backup() -> AppError {
    AppError::Corrupt("the file is not a lockwell backup".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::entries::EntryInput;
    use crate::vault::header::tests::FAST_KDF;
    use crate::vault::session::tests::{PASSWORD, unlocked_vault};

    fn entry(title: &str) -> EntryInput {
        EntryInput {
            title: title.into(),
            username: "user".into(),
            password: "example-password".into(),
            url: String::new(),
            notes: String::new(),
        }
    }

    /// A vault with one entry and one file, and a backup of it.
    fn vault_with_backup() -> (tempfile::TempDir, Vault, tempfile::TempDir, PathBuf) {
        let (vault_dir, vault) = unlocked_vault();
        let outside = tempfile::tempdir().unwrap();
        let source = outside.path().join("doc.txt");
        fs::write(&source, b"file content").unwrap();
        {
            let unlocked = vault.unlocked().unwrap();
            unlocked.create_entry(&entry("Mail")).unwrap();
            unlocked.import_file(&source).unwrap();
        }
        let backup = outside.path().join("vault.lwbackup");
        vault.unlocked().unwrap().create_backup(&backup).unwrap();
        (vault_dir, vault, outside, backup)
    }

    fn titles(vault: &Vault) -> Vec<String> {
        vault
            .unlocked()
            .unwrap()
            .list_entries()
            .unwrap()
            .into_iter()
            .map(|entry| entry.title)
            .collect()
    }

    /// Names in the folder that holds the vault folder.
    fn siblings(vault: &Vault) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(vault.dir().parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn restore_brings_back_entries_and_files() {
        let (_vault_dir, mut vault, outside, backup) = vault_with_backup();
        {
            let unlocked = vault.unlocked().unwrap();
            for summary in unlocked.list_entries().unwrap() {
                unlocked.delete_entry(&summary.id).unwrap();
            }
            for file in unlocked.list_files().unwrap() {
                unlocked.delete_file(&file.id).unwrap();
            }
            unlocked.create_entry(&entry("Added later")).unwrap();
        }

        vault.restore_backup(&backup, PASSWORD).unwrap();
        assert!(!vault.is_unlocked());
        vault.unlock(PASSWORD).unwrap();
        assert_eq!(titles(&vault), ["Mail"]);
        let unlocked = vault.unlocked().unwrap();
        let files = unlocked.list_files().unwrap();
        assert_eq!(files.len(), 1);
        let out = outside.path().join("restored.txt");
        unlocked.export_file(&files[0].id, &out, false).unwrap();
        assert_eq!(fs::read(out).unwrap(), b"file content");
        assert_eq!(siblings(&vault), ["vault"]);
    }

    #[test]
    fn restore_into_a_new_computer() {
        let (_vault_dir, _vault, _outside, backup) = vault_with_backup();
        let fresh = tempfile::tempdir().unwrap();
        let mut vault = Vault::with_kdf(fresh.path().join("vault"), FAST_KDF);
        assert!(!vault.exists());
        vault.restore_backup(&backup, PASSWORD).unwrap();
        assert!(vault.exists());
        vault.unlock(PASSWORD).unwrap();
        assert_eq!(titles(&vault), ["Mail"]);
    }

    #[test]
    fn backup_uses_the_password_at_backup_time() {
        let (_vault_dir, mut vault, _outside, backup) = vault_with_backup();
        vault
            .change_master_password(PASSWORD, "a brand new password")
            .unwrap();
        assert_eq!(
            vault
                .restore_backup(&backup, "a brand new password")
                .unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        vault.restore_backup(&backup, PASSWORD).unwrap();
        vault.unlock(PASSWORD).unwrap();
    }

    #[test]
    fn wrong_password_leaves_the_vault_untouched() {
        let (_vault_dir, mut vault, _outside, backup) = vault_with_backup();
        vault
            .unlocked()
            .unwrap()
            .create_entry(&entry("Newer"))
            .unwrap();
        assert_eq!(
            vault
                .restore_backup(&backup, "wrong password!!")
                .unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
        assert!(vault.is_unlocked());
        assert_eq!(titles(&vault), ["Mail", "Newer"]);
        assert_eq!(siblings(&vault), ["vault"]);
    }

    #[test]
    fn damaged_backups_are_rejected_without_side_effects() {
        let (_vault_dir, mut vault, outside, backup) = vault_with_backup();
        let bytes = fs::read(&backup).unwrap();
        let damaged = outside.path().join("damaged.lwbackup");
        let cases: Vec<Vec<u8>> = vec![
            bytes[..bytes.len() - 1].to_vec(),
            bytes[..PREFIX_LEN + 10].to_vec(),
            {
                let mut flipped = bytes.clone();
                flipped[PREFIX_LEN + 40] ^= 1;
                flipped
            },
            {
                let mut longer = bytes.clone();
                longer.extend_from_slice(b"extra");
                longer
            },
        ];
        for case in cases {
            fs::write(&damaged, &case).unwrap();
            assert!(matches!(
                vault.restore_backup(&damaged, PASSWORD),
                Err(AppError::Corrupt(_))
            ));
            assert!(vault.is_unlocked());
            assert_eq!(siblings(&vault), ["vault"]);
        }
    }

    #[test]
    fn a_tampered_clear_prefix_is_rejected() {
        let (_vault_dir, mut vault, outside, backup) = vault_with_backup();
        let mut bytes = fs::read(&backup).unwrap();
        // A salt byte: the derived key changes, so the wrapped key fails.
        bytes[8 + 20] ^= 1;
        let damaged = outside.path().join("damaged.lwbackup");
        fs::write(&damaged, bytes).unwrap();
        assert_eq!(
            vault.restore_backup(&damaged, PASSWORD).unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
    }

    #[test]
    fn other_files_are_not_backups() {
        let (_vault_dir, mut vault, outside, _backup) = vault_with_backup();
        let other = outside.path().join("other.lwbackup");
        fs::write(
            &other,
            b"just some text that is not a backup at all, really",
        )
        .unwrap();
        assert!(matches!(
            vault.restore_backup(&other, PASSWORD),
            Err(AppError::Corrupt(_))
        ));
        fs::write(&other, vec![0u8; PREFIX_LEN + 100]).unwrap();
        assert!(matches!(
            vault.restore_backup(&other, PASSWORD),
            Err(AppError::Corrupt(_))
        ));
    }

    #[test]
    fn newer_backups_are_unsupported() {
        let (_vault_dir, mut vault, outside, backup) = vault_with_backup();
        let mut bytes = fs::read(&backup).unwrap();
        bytes[4..8].copy_from_slice(&(BACKUP_FORMAT_VERSION + 1).to_le_bytes());
        let newer = outside.path().join("newer.lwbackup");
        fs::write(&newer, bytes).unwrap();
        assert_eq!(
            vault.restore_backup(&newer, PASSWORD).unwrap_err(),
            AppError::UnsupportedVersion
        );
    }

    #[test]
    fn backups_never_go_inside_the_vault() {
        let (_vault_dir, mut vault, _outside, _backup) = vault_with_backup();
        let inside = vault.dir().join("copy.lwbackup");
        assert!(matches!(
            vault.unlocked().unwrap().create_backup(&inside),
            Err(AppError::InvalidInput(_))
        ));
        let database = vault.dir().join(DATABASE_FILE);
        assert!(matches!(
            vault.restore_backup(&database, PASSWORD),
            Err(AppError::InvalidInput(_))
        ));
    }

    #[test]
    fn backup_content_is_encrypted() {
        let (_vault_dir, _vault, _outside, backup) = vault_with_backup();
        let bytes = fs::read(&backup).unwrap();
        assert_eq!(&bytes[..4], BACKUP_MAGIC);
        for needle in [&b"SQLite format"[..], b"LWF1", b"LWBC", b"file content"] {
            assert!(!bytes.windows(needle.len()).any(|window| window == needle));
        }
    }

    #[test]
    fn a_backup_with_a_missing_file_is_refused() {
        let (_vault_dir, vault, outside, _backup) = vault_with_backup();
        let unlocked = vault.unlocked().unwrap();
        for entry in fs::read_dir(unlocked.files_dir()).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        let target = outside.path().join("second.lwbackup");
        assert!(matches!(
            unlocked.create_backup(&target),
            Err(AppError::Corrupt(_))
        ));
        assert!(!target.exists());
    }

    #[test]
    fn container_parser_rejects_bad_records() {
        let staging = tempfile::tempdir().unwrap();
        fs::create_dir_all(staging.path().join(FILES_DIR)).unwrap();
        let record = |kind: u8, name: &str, data: &[u8]| {
            let mut bytes = vec![kind];
            bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&(data.len() as u64).to_le_bytes());
            bytes.extend_from_slice(data);
            bytes
        };
        let parse = |body: Vec<u8>| {
            let dir = tempfile::tempdir().unwrap();
            fs::create_dir_all(dir.path().join(FILES_DIR)).unwrap();
            let mut bytes = CONTAINER_MAGIC.to_vec();
            bytes.extend(body);
            unpack_contents(&mut Cursor::new(bytes), dir.path())
        };

        // No database.
        assert!(parse(vec![KIND_END]).is_err());
        // Path traversal in a blob name.
        let mut body = record(KIND_DATABASE, DATABASE_FILE, b"db");
        body.extend(record(KIND_BLOB, "../evil.lwf", b"x"));
        body.push(KIND_END);
        assert!(parse(body).is_err());
        // Two databases.
        let mut body = record(KIND_DATABASE, DATABASE_FILE, b"db");
        body.extend(record(KIND_DATABASE, DATABASE_FILE, b"db"));
        body.push(KIND_END);
        assert!(parse(body).is_err());
        // Unknown kind.
        let mut body = record(KIND_DATABASE, DATABASE_FILE, b"db");
        body.extend(record(9, "x", b""));
        body.push(KIND_END);
        assert!(parse(body).is_err());
        // Declared length longer than the data.
        let mut body = record(KIND_DATABASE, DATABASE_FILE, b"db");
        body.truncate(body.len() - 1);
        assert!(parse(body).is_err());
        // A valid container.
        let mut body = record(KIND_DATABASE, DATABASE_FILE, b"db");
        body.extend(record(KIND_BLOB, &format!("{}.lwf", "a".repeat(32)), b"x"));
        body.push(KIND_END);
        assert!(parse(body).is_ok());
    }

    /// One container record, as written by `add_record`.
    fn record_bytes(kind: u8, name: &[u8], data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![kind];
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name);
        bytes.extend_from_slice(&(data.len() as u64).to_le_bytes());
        bytes.extend_from_slice(data);
        bytes
    }

    /// Unpacks `contents` into a fresh staging folder.
    fn unpack(contents: Vec<u8>) -> AppResult<()> {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(FILES_DIR)).unwrap();
        unpack_contents(&mut Cursor::new(contents), dir.path())
    }

    fn container(records: &[Vec<u8>]) -> Vec<u8> {
        let mut bytes = CONTAINER_MAGIC.to_vec();
        for record in records {
            bytes.extend_from_slice(record);
        }
        bytes
    }

    #[test]
    fn container_parser_names_each_problem() {
        let database = record_bytes(KIND_DATABASE, DATABASE_FILE.as_bytes(), b"db");
        let blob_name = format!("{}.{BLOB_EXTENSION}", "b".repeat(32));
        let blob = record_bytes(KIND_BLOB, blob_name.as_bytes(), b"x");
        let end = vec![KIND_END];

        let cases = [
            (b"NOPE".to_vec(), "unknown backup contents"),
            (CONTAINER_MAGIC[..2].to_vec(), "the backup is truncated"),
            (
                container(std::slice::from_ref(&database)),
                "the backup is truncated",
            ),
            (
                container(&[database.clone(), end.clone(), b"x".to_vec()]),
                "unexpected data after the end",
            ),
            (
                container(&[record_bytes(KIND_DATABASE, &[0xFF, 0xFE], b""), end.clone()]),
                "invalid name",
            ),
            (
                container(&[database.clone(), blob.clone(), blob, end.clone()]),
                "unexpected record",
            ),
            (container(&[end]), "the database is missing"),
        ];
        for (contents, detail) in cases {
            assert_eq!(unpack(contents), Err(corrupt(detail)), "{detail}");
        }
    }

    /// A copy of the vault folder, made while the vault is locked, together
    /// with the vault key and header.
    fn staged_copy(vault: &mut Vault) -> (tempfile::TempDir, SecretKey, VaultHeader) {
        vault.lock();
        let staging = tempfile::tempdir().unwrap();
        fs::create_dir_all(staging.path().join(FILES_DIR)).unwrap();
        fs::copy(vault.database_path(), staging.path().join(DATABASE_FILE)).unwrap();
        for blob in fs::read_dir(vault.dir().join(FILES_DIR)).unwrap() {
            let blob = blob.unwrap();
            fs::copy(
                blob.path(),
                staging.path().join(FILES_DIR).join(blob.file_name()),
            )
            .unwrap();
        }
        vault.unlock(PASSWORD).unwrap();
        let unlocked = vault.unlocked().unwrap();
        (
            staging,
            unlocked.key.clone(),
            read_header(&unlocked.conn).unwrap(),
        )
    }

    fn only_blob(staging: &Path) -> PathBuf {
        let mut blobs = fs::read_dir(staging.join(FILES_DIR)).unwrap();
        let blob = blobs.next().unwrap().unwrap().path();
        assert!(blobs.next().is_none());
        blob
    }

    #[test]
    fn staged_vault_check_accepts_an_intact_copy() {
        let (_vault_dir, mut vault, _outside, _backup) = vault_with_backup();
        let (staging, key, header) = staged_copy(&mut vault);
        assert_eq!(check_staged_vault(staging.path(), &header, &key), Ok(()));
    }

    #[test]
    fn staged_vault_check_refuses_a_header_that_does_not_match() {
        let (_vault_dir, mut vault, _outside, _backup) = vault_with_backup();
        let (staging, key, _header) = staged_copy(&mut vault);
        let other = VaultHeader::wrap(&key, b"another password", FAST_KDF).unwrap();
        assert_eq!(
            check_staged_vault(staging.path(), &other, &key),
            Err(corrupt("the backup header does not match its database"))
        );
    }

    #[test]
    fn staged_vault_check_refuses_missing_or_foreign_blobs() {
        let (_vault_dir, mut vault, _outside, _backup) = vault_with_backup();
        let (staging, key, header) = staged_copy(&mut vault);
        let blob = only_blob(staging.path());

        fs::write(&blob, b"XXXXnot a blob").unwrap();
        assert_eq!(
            check_staged_vault(staging.path(), &header, &key),
            Err(corrupt("a file in the backup has an unknown format"))
        );
        fs::remove_file(&blob).unwrap();
        assert_eq!(
            check_staged_vault(staging.path(), &header, &key),
            Err(corrupt("a file is missing from the backup"))
        );
    }

    #[test]
    fn staged_vault_check_refuses_rows_of_another_key() {
        let (_vault_dir, mut vault, _outside, _backup) = vault_with_backup();
        let (staging, _key, header) = staged_copy(&mut vault);
        let other_key = SecretKey::from_slice(&[7u8; 32]).unwrap();
        assert!(matches!(
            check_staged_vault(staging.path(), &header, &other_key),
            Err(AppError::Corrupt(_))
        ));
    }

    #[test]
    fn restore_needs_an_existing_file() {
        let (_vault_dir, mut vault, outside, _backup) = vault_with_backup();
        assert_eq!(
            vault.restore_backup(outside.path(), PASSWORD).unwrap_err(),
            AppError::invalid("the backup must be a file")
        );
        assert_eq!(
            vault
                .restore_backup(&outside.path().join("missing.lwbackup"), PASSWORD)
                .unwrap_err()
                .code(),
            "io"
        );
        assert!(vault.is_unlocked());
        assert_eq!(siblings(&vault), ["vault"]);
    }

    #[test]
    fn restore_needs_a_vault_folder_with_a_parent() {
        let (_vault_dir, _vault, _outside, backup) = vault_with_backup();
        // A bare relative name has no parent folder to stage the restore in.
        let mut vault = Vault::with_kdf("lockwell-test-vault-without-parent", FAST_KDF);
        assert_eq!(
            vault.restore_backup(&backup, PASSWORD).unwrap_err(),
            AppError::invalid("the vault folder has no parent folder")
        );
        assert!(!vault.exists());
    }

    #[test]
    fn concat_reads_its_parts_in_order() {
        let parts: Vec<Box<dyn Read + Send>> = vec![
            Box::new(Cursor::new(b"ab".to_vec())),
            Box::new(Cursor::new(Vec::new())),
            Box::new(Cursor::new(b"cde".to_vec())),
        ];
        let mut concat = Concat {
            parts: parts.into(),
        };
        assert_eq!(concat.read(&mut []).unwrap(), 0);
        let mut joined = Vec::new();
        concat.read_to_end(&mut joined).unwrap();
        assert_eq!(joined, b"abcde");
        assert_eq!(concat.read(&mut [0u8; 4]).unwrap(), 0);
    }

    #[test]
    fn blob_names_must_be_generated_ids() {
        let id = "c".repeat(32);
        assert!(is_blob_name(&format!("{id}.{BLOB_EXTENSION}")));
        assert!(!is_blob_name(&id));
        assert!(!is_blob_name(&format!("{id}.txt")));
        assert!(!is_blob_name(&format!("../{id}.{BLOB_EXTENSION}")));
    }
}
