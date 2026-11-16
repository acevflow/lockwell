# src-tauri/src/commands/vault.rs

Vault commands:

| Command                  | Arguments        | Result        |
| ------------------------ | ---------------- | ------------- |
| `vault_status`           | none             | `VaultStatus` |
| `create_vault`           | `password`       | none          |
| `unlock_vault`           | `password`       | none          |
| `lock_vault`             | none             | none          |
| `change_master_password` | `current`, `new` | none          |

`lock_vault` also clears a secret this app left on the clipboard.

Passwords are moved into wiped-on-drop buffers as soon as they arrive and are
never logged. Commands that run Argon2id execute on a blocking thread.
Errors are returned as `AppError`.
