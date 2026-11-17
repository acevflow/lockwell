# src-tauri/src/commands/entries.rs

## Purpose

Commands to list, read, create, update and delete password entries.

## Place in the architecture

Thin wrappers over the `UnlockedVault` methods of
[`vault/entries.rs`](../vault/entries.rs.md), run through `AppState::with_vault`.
All of them need an unlocked vault and fail with `locked` otherwise. Only
`get_entry` returns the password and the notes to the interface. The interface
calls them through `src/api/entries.ts`.

## Public API

| Command                                               | Description                                          | Errors (`code`)                                              |
| ----------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------ |
| `list_entries() -> Vec<EntrySummary>`                 | Entries without passwords or notes, sorted by title. | `locked`, `corrupt`.                                         |
| `get_entry(id: String) -> Entry`                      | One complete entry, including its password.          | `locked`, `invalid_input` (malformed id), `not_found`, `corrupt`. |
| `create_entry(entry: EntryInput) -> String`           | Creates an entry and returns its id.                 | `locked`, `invalid_input`.                                   |
| `update_entry(id: String, entry: EntryInput)`         | Replaces the fields of an entry.                     | `locked`, `invalid_input`, `not_found`.                      |
| `delete_entry(id: String)`                            | Deletes an entry.                                    | `locked`, `invalid_input`, `not_found`.                      |

Argument names are the ones the interface sends: `id` and `entry`. The `entry`
object uses camelCase fields (`title`, `username`, `password`, `url`, `notes`).
Only `title` is required.

Side effects: each call counts as user activity. Create, update and delete write
to the database.

## Internal structure

Each command is one line: `state.with_vault(|vault| vault.unlocked()?.<method>(...))`.
`unlocked()?` turns a locked vault into `AppError::Locked`.

## Dependencies

- Internal: [`state.rs`](../state.rs.md), [`vault/entries.rs`](../vault/entries.rs.md)
  (`Entry`, `EntryInput`, `EntrySummary`), [`error.rs`](../error.rs.md).
- External: `tauri` (`State`).

## Usage example

Verified at the level of the core. Unknown fields in the argument are rejected
when Tauri deserializes it:

```rust
// {"title":"a"}               -> accepted, other fields default to ""
// {"title":"a","extra":1}     -> rejected (unknown field)
state.with_vault(|v| v.unlocked()?.create_entry(&input))?;   // returns the new id
```

From the interface: `await invoke("get_entry", { id })`.

## Edge cases, invariants and error handling

- A locked vault fails every command with `locked`, before any data is touched.
- Malformed identifiers are rejected by the vault layer before they reach SQL or
  a path.
- The list never carries secrets. A secret leaves the core only through
  `get_entry`, or through `copy_entry_field` to the clipboard.

## Security

- `EntryInput` and `Entry` are wiped on drop in the core. Once serialized for
  the interface, the values live in the interface's memory, which this layer
  cannot wipe.
- The ignored finding about `Debug` output on these types is described in
  [`vault/entries.rs`](../vault/entries.rs.md#security). The commands do not
  format them.

## Related files and tests

- Related: [`mod.rs`](mod.rs.md), [`vault/entries.rs`](../vault/entries.rs.md),
  [`security.rs`](security.rs.md) (`copy_entry_field`), [`state.rs`](../state.rs.md).
- Tests: no test inside this file. See
  [`entries_files_flow.rs`](../../../../../src-tauri/tests/entries_files_flow.rs)
  and [`security.rs`](../../../../../src-tauri/tests/security.rs).
