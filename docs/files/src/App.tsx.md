# src/App.tsx

Root component of the interface. On start it asks the core for the vault
status (`vault_status`) and shows one of:

- `SetupScreen` when no vault exists,
- `UnlockScreen` when the vault is locked,
- `VaultScreen` when it is unlocked,
- an error message if the status cannot be read.

Locking calls `lock_vault` and always returns to the unlock screen, even if
the call fails, so decrypted data is never left on screen. When the core
reports that the vault is locked, it also returns to the unlock screen.
