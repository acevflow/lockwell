# src-tauri/src/storage/db.rs

## Purpose

Opens and creates the vault's SQLite database and gives raw access to its
tables. Every value it stores is an opaque blob that the caller has already
encrypted.

## Place in the architecture

- Lowest layer of persistence. It does no encryption and no validation of
  record contents: [`vault/`](../vault/mod.rs.md) encrypts before calling in
  and decrypts after reading.
- SQLite therefore sees only random ids and encrypted blobs. The database
  reveals how many items exist and their approximate sizes, nothing else.
- Errors are expressed as [`AppError`](../error.rs.md) values so they reach the
  interface with a stable code.

Schema version 1, stored in SQLite's `user_version`:

| Table     | Columns        | Content                                |
| --------- | -------------- | -------------------------------------- |
| `meta`    | `key`, `value` | Vault metadata, such as the key header |
| `entries` | `id`, `data`   | One encrypted password entry per row   |
| `files`   | `id`, `data`   | One encrypted file description per row |

All tables are `WITHOUT ROWID`, so rows are keyed only by their random id with
no sequential row number.

## Public API

| Item                                                              | Description                                             |
| ----------------------------------------------------------------- | ------------------------------------------------------- |
| `SCHEMA_VERSION: i32 = 1`                                         | Current schema version.                                 |
| `enum Table { Entries, Files }`                                   | Tables that hold encrypted records.                     |
| `create(path: &Path) -> AppResult<Connection>`                    | Creates a new database with the current schema.         |
| `open(path: &Path) -> AppResult<Connection>`                      | Opens an existing database and checks its version.      |
| `get_meta(conn, key: &str) -> AppResult<Option<Vec<u8>>>`         | Reads a metadata value. `None` if the key is absent.    |
| `set_meta(conn, key: &str, value: &[u8]) -> AppResult<()>`        | Inserts or replaces a metadata value.                   |
| `insert_record(conn, table, id: &str, data: &[u8]) -> AppResult<()>` | Inserts a new record.                                |
| `update_record(conn, table, id, data) -> AppResult<()>`           | Replaces the data of an existing record.                |
| `get_record(conn, table, id) -> AppResult<Vec<u8>>`               | Reads one record.                                       |
| `list_records(conn, table) -> AppResult<Vec<(String, Vec<u8>)>>`  | Reads all records as `(id, data)`, ordered by `id`.     |
| `delete_record(conn, table, id) -> AppResult<()>`                 | Deletes one record.                                     |

Errors:

| Function                                  | Errors                                                                                                   |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `create`                                  | `VaultExists` if a file is already at `path`; `Storage` if SQLite fails.                                 |
| `open`                                    | `VaultNotFound` if there is no file; `Corrupt` if it is not a lockwell database; `UnsupportedVersion` if a newer version created it. |
| `update_record`, `get_record`, `delete_record` | `NotFound("entry")` or `NotFound("file")` when the id does not exist; `Storage` if SQLite fails.   |
| `insert_record`                           | `Storage` on any SQLite failure, including a duplicate id.                                               |
| `get_meta`, `set_meta`, `list_records`    | `Storage` if SQLite fails.                                                                               |

Side effects: `create` makes a file on disk. Every write goes to the database
file. Connections are opened with `SQLITE_OPEN_NO_MUTEX`, so a `Connection`
must not be shared between threads without external locking.

## Internal structure

- `create` refuses an existing path, opens the file, applies `configure`, runs
  the version 1 schema and sets `user_version`.
- `open` requires a regular file, reads `user_version`, then:
  - maps SQLite's "not a database" error to `Corrupt`
    (`map_not_a_database`);
  - rejects a version above `SCHEMA_VERSION` as `UnsupportedVersion`;
  - rejects any other version as `Corrupt("unknown database schema")`.
- `configure` turns on `secure_delete` for the connection.
- `Table::name` returns a fixed table name. The record functions format that
  name into the SQL text. Ids and data always travel as bound parameters.
- `record_name` gives the singular name (`entry` or `file`) used in
  `NotFound`.

## Dependencies

- Internal: [`error.rs`](../error.rs.md) (`AppError`, `AppResult`).
- External: `rusqlite` with the bundled SQLite, so the app does not depend on
  a system SQLite.

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::storage::db::{self, Table};

let conn = db::create(&path)?;                       // a second create gives VaultExists
db::set_meta(&conn, "header", b"one")?;
db::insert_record(&conn, Table::Entries, "b", b"2")?;
db::insert_record(&conn, Table::Entries, "a", b"1")?;
let rows = db::list_records(&conn, Table::Entries)?; // [("a", [49]), ("b", [50])]
db::delete_record(&conn, Table::Entries, "a")?;
// db::get_record(&conn, Table::Entries, "a") -> Err(NotFound("entry"))
```

Observed results: a duplicate insert returns
`Storage("UNIQUE constraint failed: entries.id")`; opening a missing path
returns `VaultNotFound`; opening 4 KiB of garbage returns
`Corrupt("the vault database is not readable")`.

## Edge cases, invariants and error handling

- `create` never overwrites. If anything exists at `path`, even an empty
  file, the result is `VaultExists`.
- A valid SQLite file that is not a lockwell vault has `user_version` 0 and is
  reported as `Corrupt("unknown database schema")`.
- `set_meta` is an upsert. `insert_record` is not, so an existing id fails.
- `list_records` loads every row into memory at once, ordered by id.
- `Table` separates the namespaces: the same id can exist in `entries` and in
  `files`.
- `open` does not check that the key header exists. That belongs to the vault
  layer.

## Security

- Only encrypted values may be passed in. This layer cannot tell ciphertext
  from plaintext and does not protect plaintext.
- `secure_delete` makes SQLite overwrite deleted rows, so removed (still
  encrypted) records do not linger in the file.
- SQL text is built only from the fixed names in `Table`, so no
  caller-provided text reaches the query string.
- Ids are used as keys and are not validated here. Callers that take ids from
  outside must validate them first
  ([`random::is_valid_id`](../crypto/random.rs.md)).
- The database file holds no key. Opening it without the master password
  yields only ciphertext.

## Related files and tests

- Related: [`mod.rs`](mod.rs.md), [`error.rs`](../error.rs.md),
  [`vault/entries.rs`](../vault/entries.rs.md),
  [`vault/files.rs`](../vault/files.rs.md),
  [`vault/header.rs`](../vault/header.rs.md),
  [`vault/backup.rs`](../vault/backup.rs.md).
- Tests: inline `tests` module in
  [`db.rs`](../../../../../src-tauri/src/storage/db.rs): create and reopen,
  `secure_delete` on, refusal of an existing file, missing file, garbage file,
  newer schema, foreign database, metadata upsert, record CRUD, separate
  tables, missing records and duplicate insert.
