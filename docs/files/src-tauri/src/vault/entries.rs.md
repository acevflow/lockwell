# src-tauri/src/vault/entries.rs

Password entries stored in the `entries` table.

Each row holds a random id and the whole entry (title, username, password,
URL, notes, creation and update times) as JSON encrypted with the vault key.
The associated data `lockwell:entry:v1:<id>` binds each row to its id, so
swapping or copying rows is detected as corruption.

Types:

- `EntryInput`: the fields the user enters. Unknown fields are rejected.
- `EntrySummary`: what the list shows (id, title, username, URL, update time).
  It has no password or notes, so listing never sends secrets to the
  interface.
- `Entry`: the complete entry, sent only when the user opens one.

`UnlockedVault` methods: `list_entries()` (sorted by title,
case-insensitive), `get_entry()`, `create_entry()`, `update_entry()` (keeps the
creation time) and `delete_entry()`.

Validation (`validate_entry()`): the title is required; maximum lengths are
200 (title), 500 (username), 4096 (password), 2048 (URL) and 20 000 (notes)
characters. Title, username and URL are trimmed; password and notes are kept
exactly. Ids must have the shape of a generated id (`check_id()`).

Entry values are wiped from memory when dropped.

Tests cover the CRUD operations, every field at and over its limit, invalid
ids and updates, encryption at rest, and swapped, tampered or malformed rows.
An ignored test records that the `Debug` output of `Entry` and `EntryInput`
includes the password and notes.
