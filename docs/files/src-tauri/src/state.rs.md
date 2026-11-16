# src-tauri/src/state.rs

`AppState`, the state managed by Tauri and shared with the background
watchers: the vault, the time of the last user activity and the clipboard,
each behind its own mutex.

- `with_vault()` runs an operation with exclusive access to the vault and
  records activity before and after it.
- `with_vault_blocking()` runs a slow operation (Argon2id, file encryption) on
  a blocking thread so the interface stays responsive.
- `touch()` records activity; `lock_if_idle()` locks after the inactivity
  timeout.
- `lock()` locks the vault and clears a secret this app left on the clipboard.
- `copy_secret()` and `clear_clipboard_if_unchanged()` drive the clipboard.

A poisoned mutex is recovered instead of failing forever, because database
writes are transactional.
