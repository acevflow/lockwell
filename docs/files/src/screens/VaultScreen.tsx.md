# src/screens/VaultScreen.tsx

The unlocked vault: a top bar with the section tabs (Passwords, Files, Settings) and
the Lock button. It passes `onRestored` to the settings. While shown, it reports user activity
(`useActivityReporter`) and listens for the `vault-locked` event
(`useVaultLockedEvent`). Everything decrypted lives in this component tree, so it is
discarded when the vault locks and the screen unmounts.
