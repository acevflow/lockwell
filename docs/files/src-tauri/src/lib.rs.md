# src-tauri/src/lib.rs

Library crate `lockwell_lib`. It declares the core modules (`commands`,
`crypto`, `error`, `security`, `state`, `storage`, `vault`) and its `run()`
function builds the Tauri application:

- It registers the dialog plugin (`tauri-plugin-dialog`), used by the core to
  show the native open and save dialogs.
- On setup it creates the `AppState` with the vault stored in the `vault`
  folder inside the app data folder
  (`%APPDATA%\com.acevflow.lockwell\vault` on Windows), starts the inactivity
  watcher and, on Windows, the session lock watcher. If the session lock
  watcher cannot start, the app refuses to start.
- It registers the vault commands (`vault_status`, `create_vault`,
  `unlock_vault`, `lock_vault`, `change_master_password`), the entry commands
  (`list_entries`, `get_entry`, `create_entry`, `update_entry`,
  `delete_entry`), the security commands (`touch_activity`,
  `copy_entry_field`), the file commands (`list_files`, `import_files`,
  `export_file`, `delete_file`) and the backup commands (`create_backup`,
  `restore_backup`).
