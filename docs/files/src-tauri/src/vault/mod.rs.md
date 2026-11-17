# src-tauri/src/vault/mod.rs

Entry point of the vault module. Declares `backup`, `entries`, `files`, `header`,
`password_policy` and `session`, re-exports `Vault` and `VaultStatus`, and
provides helpers shared by the vault files:

- `now_millis()`, the timestamp used for records.
- `random_suffix()`, 16 random hexadecimal digits to name temporary files and
  folders.
- `decrypt_failure(error, damaged)`, which maps a stream decryption failure:
  file system errors stay as `Io` and anything else becomes `Corrupt` with the
  `damaged` message.

Tests cover both helpers.
