# src-tauri/src/vault/entries.rs

## Purpose

Password entries: validation, encryption and the create, read, update, delete
and list operations.

## Place in the architecture

- Adds methods to `UnlockedVault` ([`session.rs`](session.rs.md)). The
  [`commands/entries.rs`](../commands/entries.rs.md) commands call them.
- Every entry is one row of the `entries` table: a random id and the whole
  entry (title, username, password, URL, notes and timestamps) serialized as
  JSON and encrypted with the vault key.
- The associated data binds the row to its id, so rows cannot be swapped or
  copied under another id.
- Lists carry only `EntrySummary`, which has no password or notes. The full
  entry is returned only by `get_entry`.

## Public API

Limits (in characters): `MAX_TITLE_CHARS = 200`, `MAX_USERNAME_CHARS = 500`,
`MAX_PASSWORD_CHARS = 4096`, `MAX_URL_CHARS = 2048`, `MAX_NOTES_CHARS = 20000`.

Types:

| Type           | Fields                                                              | Notes                                                                   |
| -------------- | ------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| `EntryInput`   | `title`, `username`, `password`, `url`, `notes`                     | `Deserialize` (camelCase, unknown fields rejected), wiped on drop. Only `title` is required. |
| `EntrySummary` | `id`, `title`, `username`, `url`, `updated_at`                      | `Serialize`. No secrets.                                                |
| `Entry`        | `id`, `title`, `username`, `password`, `url`, `notes`, `created_at`, `updated_at` | `Serialize` and `Deserialize`, wiped on drop. `id` is not part of the encrypted payload. |

Methods of `UnlockedVault`:

| Method                                               | Description                                        | Errors                                                          |
| ---------------------------------------------------- | -------------------------------------------------- | --------------------------------------------------------------- |
| `list_entries() -> AppResult<Vec<EntrySummary>>`     | All entries sorted by title, case-insensitive, ties by id. | `Corrupt` if any row fails to decrypt.                  |
| `get_entry(id) -> AppResult<Entry>`                  | One full entry.                                    | `InvalidInput` (malformed id), `NotFound`, `Corrupt`.           |
| `create_entry(&EntryInput) -> AppResult<String>`     | Creates an entry and returns its id.               | `InvalidInput` (a field breaks the rules), storage or crypto errors. |
| `update_entry(id, &EntryInput) -> AppResult<()>`     | Replaces the fields, keeps `created_at`.           | `InvalidInput`, `NotFound`, `Corrupt`.                          |
| `delete_entry(id) -> AppResult<()>`                  | Deletes an entry.                                  | `InvalidInput`, `NotFound`.                                     |

Free function: `validate_entry(&EntryInput) -> AppResult<()>`. Fails with
`InvalidInput` naming the field and the limit, never the value. Crate-visible:
`check_id(id)`, which [`files.rs`](files.rs.md) also uses.

## Internal structure

- `encrypt_entry` serializes the entry to JSON in a `Zeroizing` buffer and seals
  it with [`aead::seal`](../crypto/aead.rs.md).
- `decrypt_entry` opens the row and parses the JSON. Any failure (wrong key, row
  moved under another id, tampering, bad JSON) becomes
  `Corrupt("an entry failed to decrypt")` or
  `Corrupt("an entry has an invalid format")`. The content never appears in the
  error.
- Associated data of a row: `lockwell:entry:v1:<id>`.
- `entry_from_input` trims `title`, `username` and `url`. It keeps `password`
  and `notes` exactly as typed, because whitespace can be part of a secret.
- `update_entry` sets `updated_at` to `max(now, previous updated_at)`, so the
  value never goes back if the clock does.

## Dependencies

- Internal: [`session.rs`](session.rs.md), [`mod.rs`](mod.rs.md) (`now_millis`),
  [`crypto/aead.rs`](../crypto/aead.rs.md), [`crypto/random.rs`](../crypto/random.rs.md),
  [`storage/db.rs`](../storage/db.rs.md), [`error.rs`](../error.rs.md).
- External: `serde` and `serde_json` (serialization), `zeroize` (wiping secrets).

## Usage example

Checked by running it against the library. `EntryInput` implements `Drop`, so
Rust does not accept `..Default::default()` on it. Set the fields instead:

```rust
use lockwell_lib::vault::entries::EntryInput;

let unlocked = vault.unlocked()?;
let mut input = EntryInput::default();
input.title = "  Mail ".into();
input.username = " me ".into();
input.password = " pw ".into();
let id = unlocked.create_entry(&input)?;
let entry = unlocked.get_entry(&id)?;
// entry.title == "Mail", entry.username == "me", entry.password == " pw "
// unlocked.get_entry("../x")  == Err(InvalidInput("invalid identifier"))
// unlocked.create_entry(&EntryInput::default()) == Err(InvalidInput("the title is required"))
```

## Edge cases, invariants and error handling

- The title must not be blank after trimming. Each limit counts characters and
  is checked on the value as typed, before trimming.
- Identifiers are checked with
  [`is_valid_id`](../crypto/random.rs.md#public-api) before they reach the
  database, so malformed ids never get to SQL or a path.
- A single row that fails to decrypt makes `list_entries` fail as a whole.
- Deleting an entry twice gives `NotFound`. A failed update leaves the entry
  unchanged.
- An authentic row with invalid JSON is `Corrupt`.
- Entries survive a lock and unlock and a master password change, because the
  vault key does not change.

## Security

- Values are encrypted at rest and bound to the entry id by the associated data.
- `EntryInput`, `Entry` and the JSON buffer are wiped on drop. `EntrySummary` is
  not, but it carries no secrets.
- **Observation, not fixed:** `EntryInput` and `Entry` derive `Debug`, which
  prints the password and the notes in plain text if a value is formatted with
  `{:?}`. `SecretKey` redacts its `Debug` output, these types do not. Nothing in
  the code formats them this way. An ignored test, `debug_output_hides_secrets`,
  records the finding. Avoid logging these values.
- Error messages name the field and the limit, never the value.

## Related files and tests

- Related: [`session.rs`](session.rs.md), [`files.rs`](files.rs.md),
  [`commands/entries.rs`](../commands/entries.rs.md),
  [`commands/security.rs`](../commands/security.rs.md) (reads an entry to copy
  a field), [`storage/db.rs`](../storage/db.rs.md).
- Tests: inline `tests` module in
  [`entries.rs`](../../../../../src-tauri/src/vault/entries.rs): creation,
  trimming, sorting, validation limits, encryption at rest, swapped and
  tampered rows, persistence across lock and password change, unknown fields
  and invalid formats. Integration tests:
  [`entries_files_flow.rs`](../../../../../src-tauri/tests/entries_files_flow.rs).
