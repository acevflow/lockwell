# src-tauri/tests/backup_flow.rs

Integration tests of encrypted backups: restoring on another computer,
replacing newer changes, damaged backups refused without touching the vault
or leaving files behind, and a wrong password.
