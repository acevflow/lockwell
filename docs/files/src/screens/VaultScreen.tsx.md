# src/screens/VaultScreen.tsx

The unlocked vault: a top bar with the section tabs (Passwords, Settings) and
the Lock button. While shown, it reports user activity
(`useActivityReporter`) and listens for the `vault-locked` event
(`useVaultLockedEvent`). Everything decrypted lives in this component tree, so it is
discarded when the vault locks and the screen unmounts.
