# src-tauri/tests/vault_lifecycle.rs

Integration tests of the vault life cycle through the public API: a vault
reopened from disk by a new instance, the full create, lock, unlock and
password change cycle, refusing a second vault, a missing vault and the
locked state.
