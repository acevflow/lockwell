# src/api/security.ts

Activity, clipboard and lock-event bindings:

- `touchActivity()` records user activity.
- `copyEntryField(id, field)` copies the username or password through the
  core, without the value passing through the interface.
- `onVaultLocked(handler)` listens for the `vault-locked` event and passes the
  reason (`idle` or `systemLock`).
- `CLIPBOARD_CLEAR_SECONDS` (30) is shown to the user after copying.
