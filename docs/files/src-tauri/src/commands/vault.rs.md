# src-tauri/src/commands/vault.rs

## Purpose

Commands to read the vault status, create and unlock the vault, lock it, and
change the master password.

## Place in the architecture

Thin wrappers over [`Vault`](../vault/session.rs.md) through
[`AppState`](../state.rs.md). The slow ones (key derivation) run on a blocking
thread with `with_vault_blocking`, so the interface stays responsive. The
interface calls them through `src/api/vault.ts`.

## Public API

| Command                                                     | Description                                                                 | Errors (`code`)                                                              |
| ----------------------------------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `vault_status() -> VaultStatus`                             | Whether a vault exists and whether it is unlocked. Returns `{ exists, unlocked }`. | none in practice.                                                      |
| `create_vault(password: String)` (async)                    | Creates a vault protected by `password` and leaves it unlocked.             | `invalid_input` (policy), `vault_exists`, storage, file system or crypto errors. |
| `unlock_vault(password: String)` (async)                    | Unlocks the vault.                                                          | `wrong_password`, `vault_not_found`, `corrupt`, `unsupported_version`.       |
| `lock_vault()`                                              | Locks the vault, wipes the key and clears a secret this app left on the clipboard. | none.                                                                 |
| `change_master_password(current: String, new: String)` (async) | Replaces the master password after checking the current one. Data is not re-encrypted. | `locked`, `wrong_password`, `invalid_input` (policy, or equal to the current one). |

Side effects: `create_vault` writes the vault folder. `lock_vault` empties the
clipboard if it holds a secret copied by the app. All of them except
`lock_vault` count as user activity.

## Internal structure

The password arguments are moved into `Zeroizing<String>` at the start of each
command and then into the closure given to `with_vault_blocking`. The closure
calls the matching `Vault` method. `lock_vault` calls `AppState::lock`, which
locks the vault and clears the clipboard in one place.

## Dependencies

- Internal: [`state.rs`](../state.rs.md), [`vault/session.rs`](../vault/session.rs.md)
  (`VaultStatus`), [`error.rs`](../error.rs.md).
- External: `tauri` (`State`), `zeroize` (`Zeroizing`).

## Usage example

Verified by running the same calls the commands make on an `AppState`:

```rust
state.with_vault(|v| Ok(v.status()))?;                  // { exists: false, unlocked: false }
state.with_vault(|v| v.create("correct horse battery"))?;
state.lock();
// an operation on the locked vault serializes as:
//   {"code":"locked","message":"the vault is locked"}
// a wrong password serializes as:
//   {"code":"wrong_password","message":"the master password is incorrect or the vault is damaged"}
```

From the interface: `await invoke("unlock_vault", { password })`.

## Edge cases, invariants and error handling

- A wrong password and a damaged key header give the same code, on purpose.
- A failed unlock leaves the vault locked, even if it was unlocked before.
- `change_master_password` requires an unlocked vault and checks the current
  password again.
- The key derivation runs on a blocking thread, so the commands are `async`.

## Security

- Passwords are wiped on drop once they reach the command. The command can only
  wipe the `String` it receives. Copies made by the IPC layer before that point
  are outside this code's control.
- Passwords are never logged and never appear in errors.
- The password policy applies on create and on change, not on unlock.
- `lock_vault` clears the clipboard as well, so locking leaves no copied secret
  behind.

## Related files and tests

- Related: [`mod.rs`](mod.rs.md), [`state.rs`](../state.rs.md),
  [`vault/session.rs`](../vault/session.rs.md),
  [`vault/password_policy.rs`](../vault/password_policy.rs.md).
- Tests: no test inside this file. The behaviour is covered in
  [`vault_lifecycle.rs`](../../../../../src-tauri/tests/vault_lifecycle.rs) and
  [`security.rs`](../../../../../src-tauri/tests/security.rs).
