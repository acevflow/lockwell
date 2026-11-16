# src-tauri/tests/common/mod.rs

Fixtures shared by the integration tests: a temporary `Workspace` with a vault
folder (fast Argon2id parameters) and a folder for user files, synthetic
entries, helpers that search every file of the vault for clear text, and an
in-memory clipboard.
