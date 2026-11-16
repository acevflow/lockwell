# src-tauri/src/vault/mod.rs

Entry point of the vault module. Declares `entries`, `header`,
`password_policy` and `session`, re-exports `Vault` and `VaultStatus`, and
provides `now_millis()`, the timestamp used for records.
