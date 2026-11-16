//! Application entry point shared by the desktop binary.

pub mod commands;
pub mod crypto;
pub mod error;
pub mod security;
pub mod state;
pub mod storage;
pub mod vault;

use tauri::Manager;

use crate::state::AppState;
use crate::vault::Vault;

/// Name of the vault folder inside the app data folder.
const VAULT_DIR: &str = "vault";

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let vault_dir = app.path().app_data_dir()?.join(VAULT_DIR);
            app.manage(AppState::new(Vault::new(vault_dir)));
            security::spawn_idle_watcher(app.handle().clone())?;
            // The lock on Windows session lock is a security requirement:
            // refuse to start without it rather than run unprotected.
            #[cfg(windows)]
            security::watch_system_lock(app.handle().clone())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::vault::vault_status,
            commands::vault::create_vault,
            commands::vault::unlock_vault,
            commands::vault::lock_vault,
            commands::vault::change_master_password,
            commands::entries::list_entries,
            commands::entries::get_entry,
            commands::entries::create_entry,
            commands::entries::update_entry,
            commands::entries::delete_entry,
            commands::security::touch_activity,
            commands::security::copy_entry_field,
            commands::files::list_files,
            commands::files::import_files,
            commands::files::export_file,
            commands::files::delete_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
