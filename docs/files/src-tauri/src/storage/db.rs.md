# src-tauri/src/storage/db.rs

The SQLite database of the vault (`rusqlite` with bundled SQLite).

Schema version 1 (stored in `user_version`):

| Table     | Columns        | Content                                |
| --------- | -------------- | -------------------------------------- |
| `meta`    | `key`, `value` | Vault metadata, such as the key header |
| `entries` | `id`, `data`   | One encrypted password entry per row   |
| `files`   | `id`, `data`   | One encrypted file description per row |

- `create()` makes a new database and refuses an existing file.
- `open()` opens an existing one and rejects files that are not lockwell
  databases (`corrupt`) or come from a newer version (`unsupported_version`).
- `get_meta()`/`set_meta()` and `insert_record()`, `update_record()`,
  `get_record()`, `list_records()`, `delete_record()` give raw access to the
  tables. Table names come from the `Table` enum, never from input.

`secure_delete` is enabled so deleted rows are overwritten on disk.
