# src-tauri/src/state.rs

`AppState`, the state managed by Tauri: the vault behind a mutex so commands
use it one at a time.

- `with_vault()` runs an operation with exclusive access.
- `with_vault_blocking()` runs a slow operation (Argon2id, file encryption) on
  a blocking thread so the interface stays responsive.

A poisoned mutex is recovered instead of failing forever, because database
writes are transactional.
