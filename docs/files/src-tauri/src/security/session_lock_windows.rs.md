# src-tauri/src/security/session_lock_windows.rs

Windows only. Detects when the Windows session is locked (Win+L, the lock
screen or switching user).

`watch(on_lock)` starts a thread that creates a hidden message-only window,
registers it with `WTSRegisterSessionNotification` and runs a message loop.
Every `WM_WTSSESSION_CHANGE` message with `WTS_SESSION_LOCK` calls `on_lock`.
It returns an error if the window cannot be created or registered; the app
then refuses to start rather than run without this protection.

The unit test checks the message filter. The lock itself cannot be triggered
in an automated test without locking the machine.
