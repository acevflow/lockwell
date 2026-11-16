# src-tauri/src/security/mod.rs

Security controls around the vault.

- `spawn_idle_watcher()` starts a thread that checks every 5 seconds and locks
  the vault after 5 minutes without activity.
- `watch_system_lock()` (Windows) locks the vault when the Windows session is
  locked.
- `schedule_clipboard_clear()` clears a copied secret after 30 seconds if the
  clipboard still holds it.
- `notify_locked()` emits the `vault-locked` event, with the reason (`idle` or
  `systemLock`), so the interface drops its decrypted state at once.

Declares the `autolock`, `clipboard` and (on Windows) `session_lock_windows`
modules.
