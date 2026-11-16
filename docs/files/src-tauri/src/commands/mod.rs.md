# src-tauri/src/commands/mod.rs

Entry point of the Tauri commands, the only functions the interface can call.
Commands stay thin: they convert their arguments and delegate to the vault.
