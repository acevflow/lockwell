# src-tauri/src/vault/session.rs

The `Vault`: the vault folder and its locked or unlocked state.

- While locked, it holds no key and no open database.
- `create()` validates the password, writes the database under a temporary
  name with the key header and renames it when complete, then leaves the vault
  unlocked. It also creates the `files` folder.
- `unlock()` opens the database, reads the header and unwraps the vault key.
- `lock()` drops the database connection and the key; the key is wiped from
  memory on drop.
- `change_master_password()` checks the current password, validates the new
  one (which must differ) and replaces the header in one transaction. The
  vault key and data do not change.
- `status()` returns a `VaultStatus` (`exists`, `unlocked`).

The vault folder contains `vault.db` and `files/`. `UnlockedVault` keeps the
vault folder, so file operations can find `files/`.

Tests cover creation, locking and unlocking (a failed unlock always leaves the
vault locked), newer header formats and schemas, missing or swapped headers,
and changing the master password.
