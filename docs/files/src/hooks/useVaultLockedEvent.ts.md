# src/hooks/useVaultLockedEvent.ts

Listens for the `vault-locked` event while the vault screen is shown and calls
`onLocked` with the reason, so the interface returns to the unlock screen and
drops its decrypted state as soon as the core locks the vault. If listening
fails, the error is logged to the console; the next command then fails with
`locked`, which has the same effect.
