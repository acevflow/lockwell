# src-tauri/src/lib.rs

Library crate `lockwell_lib`. It declares the core modules (`commands`,
`crypto`, `error`, `state`, `storage`, `vault`) and its `run()` function builds
the Tauri application:

- On setup it creates the `AppState` with the vault stored in the `vault`
  folder inside the app data folder
  (`%APPDATA%\com.acevflow.lockwell\vault` on Windows).
- It registers the vault commands (`vault_status`, `create_vault`,
  `unlock_vault`, `lock_vault`, `change_master_password`) and the entry
  commands (`list_entries`, `get_entry`, `create_entry`, `update_entry`,
  `delete_entry`).
