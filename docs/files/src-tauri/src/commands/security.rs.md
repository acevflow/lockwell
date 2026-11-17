# src-tauri/src/commands/security.rs

## Purpose

Commands for user activity and for copying a field of an entry to the clipboard
without sending it to the interface.

## Place in the architecture

Wrappers over [`AppState`](../state.rs.md) and the clipboard timer of
[`security/mod.rs`](../security/mod.rs.md). The interface reports activity and
asks for copies through `src/api/security.ts`.

## Public API

| Item                                                | Description                                                                            | Errors (`code`)                                           |
| --------------------------------------------------- | -------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| `enum EntryField { Username, Password }`            | Field that can be copied. Deserialized from `"username"` or `"password"` (lowercase).  | An unknown name is rejected when the argument is parsed.  |
| `touch_activity()`                                  | Records user activity, postponing the automatic lock.                                  | none.                                                     |
| `copy_entry_field(id: String, field: EntryField)`   | Copies the field to the clipboard. It is cleared after 30 seconds if still there.      | `locked`, `invalid_input` (malformed id, or empty field), `not_found`, `corrupt`, `io`. |

Side effects: `copy_entry_field` writes to the system clipboard and starts a timer
thread. The value is never returned to the interface.

## Internal structure

`copy_field` reads the entry through `with_vault`, picks the field, refuses an
empty one (`this field is empty`), copies with `AppState::copy_secret` and starts
`schedule_clipboard_clear` with the generation of that copy. If the timer thread
cannot start, the result is `io` (`cannot schedule clearing the clipboard`). The
entry is wiped on drop when the function ends.

## Dependencies

- Internal: [`state.rs`](../state.rs.md), [`security/mod.rs`](../security/mod.rs.md)
  (`schedule_clipboard_clear`), [`security/clipboard.rs`](../security/clipboard.rs.md),
  [`vault/entries.rs`](../vault/entries.rs.md), [`error.rs`](../error.rs.md).
- External: `tauri` (`State`), `serde` (`Deserialize`).

## Usage example

Verified by running the same calls on an `AppState` with an in-memory clipboard:

```rust
let entry = state.with_vault(|v| v.unlocked()?.get_entry(&id))?;
let generation = state.copy_secret(&entry.password)?;   // clipboard now holds the secret
state.lock();                                           // lock_vault: clipboard is cleared
```

From the interface: `await invoke("copy_entry_field", { id, field: "password" })`.

## Edge cases, invariants and error handling

- An empty username or password is refused, so the clipboard is never emptied by
  a copy.
- Copying again before the timer fires replaces the secret. The older timer then
  does nothing, because its generation is no longer current.
- A locked vault fails with `locked`. Locking also clears the clipboard.
- `touch_activity` cannot fail.

## Security

- The secret goes from the vault to the clipboard inside the core. It does not
  cross to the interface.
- Clearing only happens if the clipboard still holds the copied text, so
  something the user copied afterwards is not erased.
- Limits from [`clipboard.rs`](../security/clipboard.rs.md#security) apply,
  including that nothing clears the clipboard if the app exits before the timer.
- `touch_activity` resets the inactivity clock. It should be called from real
  user actions only.

## Related files and tests

- Related: [`mod.rs`](mod.rs.md), [`state.rs`](../state.rs.md),
  [`security/autolock.rs`](../security/autolock.rs.md),
  [`security/clipboard.rs`](../security/clipboard.rs.md),
  [`vault/entries.rs`](../vault/entries.rs.md).
- Tests: inline `tests` module in
  [`security.rs`](../../../../../src-tauri/src/commands/security.rs): field name
  parsing, copying password and username, refusing an empty field and requiring
  an unlocked vault. Integration tests:
  [`security.rs`](../../../../../src-tauri/tests/security.rs).
