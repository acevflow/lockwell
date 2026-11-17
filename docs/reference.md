# Reference

**Purpose:** exact description of the core's command interface, error codes,
data types, limits and on-disk formats.
**Audience:** contributors and reviewers. For explanations, see the
[architecture](architecture.md); for per-file detail, see
[`files/`](files/README.md).

lockwell has no public library API and no command line. Its programmatic
interface is the set of Tauri commands that the interface calls with
`invoke`, plus one event.

## Table of contents

- [Calling conventions](#calling-conventions)
- [Commands](#commands)
- [Event](#event)
- [Data types](#data-types)
- [Error codes](#error-codes)
- [Limits](#limits)
- [Cryptographic parameters](#cryptographic-parameters)
- [On-disk formats](#on-disk-formats)
- [Associated data](#associated-data)

## Calling conventions

- Command names are `snake_case`. Argument keys are `camelCase`
  (for example `{ id }`, `{ current, new }`).
- Every command returns a value or fails with an error serialized as
  `{ "code": "<code>", "message": "<text>" }`. Messages never contain
  passwords, keys or decrypted content.
- In the interface, `src/api/client.ts` turns these errors into `ApiError`
  with a `code`. Any other failure becomes the client-side code `unknown`.
- Unless noted, a command that needs the vault fails with `locked` when the
  vault is locked. Every command that goes through the vault also counts as
  user activity for the inactivity lock.
- Commands marked *dialog* open a native file dialog from the core.

## Commands

| Command | Arguments | Returns | Possible error codes |
| --- | --- | --- | --- |
| `vault_status` | none | `VaultStatus` | none |
| `create_vault` | `password` | nothing | `invalid_input` (password policy), `vault_exists`, `storage`, `io`, `crypto` |
| `unlock_vault` | `password` | nothing | `vault_not_found`, `wrong_password`, `corrupt`, `unsupported_version` |
| `lock_vault` | none | nothing | none. Also clears the clipboard. |
| `change_master_password` | `current`, `new` | nothing | `locked`, `wrong_password`, `invalid_input` (policy, or the new password equals the current one) |
| `list_entries` | none | `EntrySummary[]`, sorted by title ignoring case | `locked`, `corrupt` |
| `get_entry` | `id` | `Entry` | `locked`, `invalid_input` (malformed id), `not_found`, `corrupt` |
| `create_entry` | `entry: EntryInput` | the new id (string) | `locked`, `invalid_input` |
| `update_entry` | `id`, `entry: EntryInput` | nothing | `locked`, `invalid_input`, `not_found` |
| `delete_entry` | `id` | nothing | `locked`, `invalid_input`, `not_found` |
| `touch_activity` | none | nothing | none |
| `copy_entry_field` | `id`, `field` (`"username"` or `"password"`) | nothing. The value goes to the clipboard and is cleared after 30 seconds. | `locked`, `invalid_input` (empty field or malformed id), `not_found`, `io` (clipboard unavailable) |
| `list_files` | none | `FileSummary[]`, sorted by name ignoring case | `locked`, `corrupt` |
| `import_files` (*dialog*) | none | `FileSummary[]` of the added files; empty if the dialog was cancelled | `locked`, `invalid_input` with the message `could not add “name”: …` (files added before the failure stay) |
| `export_file` (*dialog*) | `id` | `boolean`: `false` if the save dialog was cancelled | `locked`, `invalid_input`, `not_found`, `corrupt`, `io` |
| `delete_file` | `id` | nothing | `locked`, `invalid_input`, `not_found` |
| `create_backup` (*dialog*) | none | `boolean`: `false` if cancelled | `locked`, `invalid_input`, `corrupt`, `io` |
| `restore_backup` (*dialog*) | `password` | `boolean`: `false` if cancelled | `invalid_input` (empty or longer than 1024 characters), `wrong_password`, `corrupt`, `unsupported_version`, `io` |

Notes:

- `create_vault`, `unlock_vault`, `change_master_password`, `import_files`,
  `export_file`, `create_backup` and `restore_backup` are asynchronous and run
  their slow work on a blocking thread.
- `unlock_vault` locks first, so a failed attempt leaves the vault locked.
- `restore_backup` does not need an unlocked vault. It replaces the vault
  folder and leaves the session locked.
- `export_file` replaces the destination chosen in the save dialog; the
  dialog's own confirmation is the only overwrite check.
- The password policy is checked on `create_vault` and `change_master_password`
  only, not on `unlock_vault`.

## Event

| Event | Payload | Emitted when |
| --- | --- | --- |
| `vault-locked` | `{ "reason": "idle" }` or `{ "reason": "systemLock" }` | The vault was locked by 5 minutes of inactivity, or by the Windows session lock while the vault was unlocked. |

The window may listen to events (`core:event:allow-listen` and
`core:event:allow-unlisten` in `src-tauri/capabilities/default.json`), and has
no other permission.

## Data types

JSON field names are `camelCase`. Times are milliseconds since the Unix
epoch.

| Type | Fields |
| --- | --- |
| `VaultStatus` | `exists: boolean`, `unlocked: boolean` |
| `EntryInput` | `title`, `username`, `password`, `url`, `notes` (all strings). Unknown fields are rejected. |
| `EntrySummary` | `id`, `title`, `username`, `url`, `updatedAt` |
| `Entry` | `id`, `title`, `username`, `password`, `url`, `notes`, `createdAt`, `updatedAt` |
| `FileSummary` | `id`, `name`, `size` (bytes of the original content), `createdAt` |

Ids are 16 random bytes written as 32 lowercase hexadecimal characters.

## Error codes

| Code | Rust variant | Meaning |
| --- | --- | --- |
| `wrong_password` | `WrongPasswordOrCorrupt` | The password is wrong or the key data is damaged. Both are reported the same way on purpose. |
| `locked` | `Locked` | The operation needs an unlocked vault. |
| `vault_exists` | `VaultExists` | A vault already exists where a new one would be created. |
| `vault_not_found` | `VaultNotFound` | There is no vault to open. |
| `invalid_input` | `InvalidInput` | Validation failed. The message says what and never includes the value. |
| `not_found` | `NotFound` | The entry or file does not exist. |
| `corrupt` | `Corrupt` | Stored data failed authentication or has an invalid format. |
| `unsupported_version` | `UnsupportedVersion` | The vault or backup was created by a newer version of lockwell. |
| `crypto` | `Crypto` | A cryptographic operation failed for a reason other than authentication. |
| `storage` | `Storage` | SQLite failed. |
| `io` | `Io` | A file or clipboard operation failed. Only the error kind is kept, so paths do not leak. |
| `unknown` | none (client side) | The failure did not come from the core. |

## Limits

Lengths are counted in characters, not bytes.

| Item | Limit |
| --- | --- |
| Master password | 12 to 1024; not made only of whitespace |
| Entry title | required (not empty after trimming); at most 200 |
| Entry username, password, URL, notes | 500, 4096, 2048, 20000 |
| File name | 255; no control characters, `/` or `\` |
| Argon2id parallelism, iterations | 1 to 16 each |
| Argon2id memory | at least 8 KiB times the parallelism, at most 1 GiB |

## Cryptographic parameters

| Item | Value |
| --- | --- |
| Password hashing | Argon2id, version 0x13, 32-byte output, 16-byte random salt |
| Default cost for new vaults | 64 MiB (65536 KiB), 3 iterations, 4 lanes |
| Record encryption | XChaCha20-Poly1305. Layout: nonce (24 random bytes), ciphertext, tag (16 bytes) |
| Stream encryption | STREAM (BE32) over XChaCha20-Poly1305: 19-byte nonce prefix, then chunks of 64 KiB plaintext plus a 16-byte tag; the last chunk holds 0 to 64 KiB |
| Vault key | 32 random bytes from the operating system's random source |
| Identifiers | 16 random bytes, hex encoded |

The Argon2id parameters are validated before every derivation and when a
header is decoded. The cryptographic crates are listed in
`src-tauri/Cargo.toml`: `argon2`, `chacha20poly1305`, `aead-stream`,
`getrandom`, `sha2` and `zeroize`.

## On-disk formats

### Vault folder

```text
<app data dir>\vault\
├── vault.db            # SQLite database
└── files\<id>.lwf      # encrypted file contents
```

Temporary names that may appear while an operation runs (or remain after a
crash) and hold only encrypted data: `vault.db.creating`,
`vault.db.snapshot-<hex>`, `.lockwell-<hex>.tmp`, and
`<vault folder>.restoring-<hex>` and `<vault folder>.replaced-<hex>` next to
the vault folder. They are removed when an operation fails cleanly, but a
forced shutdown can leave them behind.

### SQLite schema (version 1)

`PRAGMA user_version` is 1 and `secure_delete` is on.

```sql
CREATE TABLE meta    (key TEXT PRIMARY KEY, value BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE entries (id  TEXT PRIMARY KEY, data  BLOB NOT NULL) WITHOUT ROWID;
CREATE TABLE files   (id  TEXT PRIMARY KEY, data  BLOB NOT NULL) WITHOUT ROWID;
```

- `meta` holds one row, `header`, with the 108-byte key header.
- `entries.data` is one sealed record: the JSON of the entry (`id`, `title`,
  `username`, `password`, `url`, `notes`, `createdAt`, `updatedAt`).
- `files.data` is one sealed record: the JSON `{ name, size, createdAt }`.

A database with a newer `user_version` fails with `unsupported_version`, and
any other unexpected version with `corrupt`.

### Key header (108 bytes, little-endian)

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | magic `LWVH` |
| 4 | 4 | format version (`1`) |
| 8 | 4 | Argon2id memory in KiB |
| 12 | 4 | Argon2id iterations |
| 16 | 4 | Argon2id parallelism |
| 20 | 16 | salt |
| 36 | 24 | nonce of the wrapped key |
| 60 | 32 | encrypted vault key |
| 92 | 16 | authentication tag |

Bytes 0 to 35 are the associated data of the wrapped key.

### File content (`.lwf`)

```text
"LWF1" (4) | STREAM-encrypted content
```

The file name on disk is the id; the original name exists only inside the
encrypted description.

### Backup (`.lwbackup`)

```text
"LWBK" (4) | format version u32 (4) | key header (108) | STREAM-encrypted container
```

The container, once decrypted:

```text
"LWBC" | record* | 0x00
record = kind u8 (1 database, 2 file blob) | name length u16 | name | data length u64 | data
```

The database record is a `VACUUM INTO` snapshot named `vault.db`. Blob records
are named `<id>.lwf` and are copied already encrypted. Restoring accepts only
those names, each at most once, and creates files with `create_new`.

## Associated data

Each sealed object is bound to its identity by associated data (AAD), so it
cannot be moved to another row, file or format.

| Object | AAD |
| --- | --- |
| Entry record | `lockwell:entry:v1:<id>` |
| File description | `lockwell:file:v1:<id>` |
| File content | `lockwell:file-data:v1:<id>` |
| Backup contents | `lockwell:backup:v1:` followed by the clear prefix (magic, version, header) |
| Wrapped vault key | the first 36 bytes of the header |
