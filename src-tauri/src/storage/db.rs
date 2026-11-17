//! SQLite database of the vault: opening, schema and raw record access.
//!
//! Schema (version 1):
//!
//! - `meta(key, value)`: vault metadata, such as the key header.
//! - `entries(id, data)`: one encrypted password entry per row.
//! - `files(id, data)`: one encrypted file description per row.
//!
//! `id` is a random identifier and `data` an encrypted blob, so the database
//! reveals only how many items there are and their approximate sizes.
//! `secure_delete` is on, so deleted rows are overwritten on disk.

use std::path::Path;

use rusqlite::{Connection, ErrorCode, OpenFlags, OptionalExtension, params};

use crate::error::{AppError, AppResult};

/// Current schema version, stored in SQLite's `user_version`.
pub const SCHEMA_VERSION: i32 = 1;

/// SQL that creates the version 1 schema. `WITHOUT ROWID` keeps rows keyed by
/// their random id only, with no extra sequential row number.
const SCHEMA_V1: &str = "
    CREATE TABLE meta (key TEXT PRIMARY KEY, value BLOB NOT NULL) WITHOUT ROWID;
    CREATE TABLE entries (id TEXT PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;
    CREATE TABLE files (id TEXT PRIMARY KEY, data BLOB NOT NULL) WITHOUT ROWID;
";

/// Tables that hold encrypted records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Table {
    /// Password entries.
    Entries,
    /// Descriptions of encrypted files.
    Files,
}

impl Table {
    /// SQL table name. Only these fixed names are ever formatted into the
    /// queries below, so no caller-provided text reaches the SQL string.
    fn name(self) -> &'static str {
        match self {
            Table::Entries => "entries",
            Table::Files => "files",
        }
    }
}

/// Creates a new database at `path` with the current schema.
///
/// # Errors
/// - [`AppError::VaultExists`] if a file already exists at `path`.
/// - [`AppError::Storage`] if SQLite fails.
pub fn create(path: &Path) -> AppResult<Connection> {
    if path.exists() {
        return Err(AppError::VaultExists);
    }
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    configure(&conn)?;
    conn.execute_batch(SCHEMA_V1)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(conn)
}

/// Opens an existing database and checks its schema version.
///
/// # Errors
/// - [`AppError::VaultNotFound`] if there is no file at `path`.
/// - [`AppError::Corrupt`] if the file is not a lockwell database.
/// - [`AppError::UnsupportedVersion`] if a newer lockwell created it.
pub fn open(path: &Path) -> AppResult<Connection> {
    if !path.is_file() {
        return Err(AppError::VaultNotFound);
    }
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(map_not_a_database)?;
    if version > SCHEMA_VERSION {
        return Err(AppError::UnsupportedVersion);
    }
    if version != SCHEMA_VERSION {
        return Err(AppError::Corrupt("unknown database schema".into()));
    }
    configure(&conn)?;
    Ok(conn)
}

/// Reads a metadata value.
///
/// # Errors
/// Returns [`AppError::Storage`] if SQLite fails.
pub fn get_meta(conn: &Connection, key: &str) -> AppResult<Option<Vec<u8>>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

/// Inserts or replaces a metadata value.
///
/// # Errors
/// Returns [`AppError::Storage`] if SQLite fails.
pub fn set_meta(conn: &Connection, key: &str, value: &[u8]) -> AppResult<()> {
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Inserts a new record.
///
/// # Errors
/// Returns [`AppError::Storage`] if SQLite fails, including on a duplicate id.
pub fn insert_record(conn: &Connection, table: Table, id: &str, data: &[u8]) -> AppResult<()> {
    let sql = format!("INSERT INTO {} (id, data) VALUES (?1, ?2)", table.name());
    conn.execute(&sql, params![id, data])?;
    Ok(())
}

/// Replaces the data of an existing record.
///
/// # Errors
/// - [`AppError::NotFound`] if no record has this id.
/// - [`AppError::Storage`] if SQLite fails.
pub fn update_record(conn: &Connection, table: Table, id: &str, data: &[u8]) -> AppResult<()> {
    let sql = format!("UPDATE {} SET data = ?2 WHERE id = ?1", table.name());
    if conn.execute(&sql, params![id, data])? == 0 {
        return Err(AppError::NotFound(record_name(table)));
    }
    Ok(())
}

/// Reads one record.
///
/// # Errors
/// - [`AppError::NotFound`] if no record has this id.
/// - [`AppError::Storage`] if SQLite fails.
pub fn get_record(conn: &Connection, table: Table, id: &str) -> AppResult<Vec<u8>> {
    let sql = format!("SELECT data FROM {} WHERE id = ?1", table.name());
    conn.query_row(&sql, [id], |row| row.get(0))
        .optional()?
        .ok_or(AppError::NotFound(record_name(table)))
}

/// Reads every record of a table as `(id, data)` pairs, ordered by id.
///
/// # Errors
/// Returns [`AppError::Storage`] if SQLite fails.
pub fn list_records(conn: &Connection, table: Table) -> AppResult<Vec<(String, Vec<u8>)>> {
    let sql = format!("SELECT id, data FROM {} ORDER BY id", table.name());
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Deletes one record.
///
/// # Errors
/// - [`AppError::NotFound`] if no record has this id.
/// - [`AppError::Storage`] if SQLite fails.
pub fn delete_record(conn: &Connection, table: Table, id: &str) -> AppResult<()> {
    let sql = format!("DELETE FROM {} WHERE id = ?1", table.name());
    if conn.execute(&sql, [id])? == 0 {
        return Err(AppError::NotFound(record_name(table)));
    }
    Ok(())
}

/// Singular name of the records of `table`, used in `NotFound` errors.
fn record_name(table: Table) -> &'static str {
    match table {
        Table::Entries => "entry",
        Table::Files => "file",
    }
}

/// Applies per-connection settings. `secure_delete` makes SQLite overwrite
/// deleted rows, so removed (still encrypted) records do not linger on disk.
fn configure(conn: &Connection) -> AppResult<()> {
    conn.pragma_update(None, "secure_delete", true)?;
    Ok(())
}

/// Turns SQLite's "not a database" error, raised when the file is not SQLite
/// at all, into [`AppError::Corrupt`]; any other error is passed through.
fn map_not_a_database(error: rusqlite::Error) -> AppError {
    match error.sqlite_error_code() {
        Some(ErrorCode::NotADatabase) => {
            AppError::Corrupt("the vault database is not readable".into())
        }
        _ => AppError::from(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_db() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let conn = create(&dir.path().join("vault.db")).unwrap();
        (dir, conn)
    }

    #[test]
    fn create_then_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        drop(create(&path).unwrap());
        let conn = open(&path).unwrap();
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        let secure: i64 = conn
            .pragma_query_value(None, "secure_delete", |row| row.get(0))
            .unwrap();
        assert_eq!(secure, 1);
    }

    #[test]
    fn create_refuses_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        std::fs::write(&path, b"x").unwrap();
        assert_eq!(create(&path).unwrap_err(), AppError::VaultExists);
    }

    #[test]
    fn open_missing_file_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            open(&dir.path().join("vault.db")).unwrap_err(),
            AppError::VaultNotFound
        );
    }

    #[test]
    fn open_garbage_is_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        std::fs::write(&path, vec![0x5a; 4096]).unwrap();
        assert!(matches!(open(&path).unwrap_err(), AppError::Corrupt(_)));
    }

    #[test]
    fn open_newer_schema_is_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let conn = create(&path).unwrap();
        conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();
        drop(conn);
        assert_eq!(open(&path).unwrap_err(), AppError::UnsupportedVersion);
    }

    #[test]
    fn open_foreign_database_is_corrupt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.db");
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE other (x INTEGER);")
            .unwrap();
        assert!(matches!(open(&path).unwrap_err(), AppError::Corrupt(_)));
    }

    #[test]
    fn meta_round_trip_and_overwrite() {
        let (_dir, conn) = new_db();
        assert_eq!(get_meta(&conn, "header").unwrap(), None);
        set_meta(&conn, "header", b"one").unwrap();
        set_meta(&conn, "header", b"two").unwrap();
        assert_eq!(get_meta(&conn, "header").unwrap(), Some(b"two".to_vec()));
    }

    #[test]
    fn record_crud() {
        let (_dir, conn) = new_db();
        insert_record(&conn, Table::Entries, "b", b"2").unwrap();
        insert_record(&conn, Table::Entries, "a", b"1").unwrap();
        assert_eq!(get_record(&conn, Table::Entries, "a").unwrap(), b"1");
        assert_eq!(
            list_records(&conn, Table::Entries).unwrap(),
            vec![("a".into(), b"1".to_vec()), ("b".into(), b"2".to_vec())]
        );
        update_record(&conn, Table::Entries, "a", b"3").unwrap();
        assert_eq!(get_record(&conn, Table::Entries, "a").unwrap(), b"3");
        delete_record(&conn, Table::Entries, "a").unwrap();
        assert_eq!(
            get_record(&conn, Table::Entries, "a").unwrap_err(),
            AppError::NotFound("entry")
        );
    }

    #[test]
    fn tables_are_separate() {
        let (_dir, conn) = new_db();
        insert_record(&conn, Table::Files, "a", b"file").unwrap();
        assert!(list_records(&conn, Table::Entries).unwrap().is_empty());
        assert_eq!(get_record(&conn, Table::Files, "a").unwrap(), b"file");
    }

    #[test]
    fn missing_records_are_not_found() {
        let (_dir, conn) = new_db();
        assert_eq!(
            update_record(&conn, Table::Files, "x", b"").unwrap_err(),
            AppError::NotFound("file")
        );
        assert_eq!(
            delete_record(&conn, Table::Entries, "x").unwrap_err(),
            AppError::NotFound("entry")
        );
    }

    #[test]
    fn duplicate_insert_fails() {
        let (_dir, conn) = new_db();
        insert_record(&conn, Table::Entries, "a", b"1").unwrap();
        assert!(matches!(
            insert_record(&conn, Table::Entries, "a", b"2").unwrap_err(),
            AppError::Storage(_)
        ));
    }
}
