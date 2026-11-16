# Code file index

One document per code file. Each document describes what the file does and
how it fits in the project.

## Interface (`src/`)

| File                                                        | Description                                             |
| ----------------------------------------------------------- | ------------------------------------------------------- |
| [`index.html`](index.html.md)                               | Interface entry page.                                   |
| [`src/main.tsx`](src/main.tsx.md)                           | Mounts the React app into the page.                     |
| [`src/App.tsx`](src/App.tsx.md)                             | Root component: picks the screen from the vault status. |
| [`src/App.test.tsx`](src/App.test.tsx.md)                   | Tests for the root component.                           |
| [`src/styles.css`](src/styles.css.md)                       | Global styles, light and dark themes.                   |
| [`src/passwordRules.ts`](src/passwordRules.ts.md)           | Master password rules mirrored from the core.           |
| [`src/passwordRules.test.ts`](src/passwordRules.test.ts.md) | Tests for the master password rules.                    |
| [`src/entryRules.ts`](src/entryRules.ts.md)                 | Entry field limits and search.                          |
| [`src/entryRules.test.ts`](src/entryRules.test.ts.md)       | Tests for the entry search.                             |
| [`src/format.ts`](src/format.ts.md)                         | Display formatting (file sizes).                        |
| [`src/format.test.ts`](src/format.test.ts.md)               | Tests for the formatting helpers.                       |
| [`src/vite-env.d.ts`](src/vite-env.d.ts.md)                 | Vite client type declarations.                          |

### Command client (`src/api/`)

| File                                              | Description                                         |
| ------------------------------------------------- | --------------------------------------------------- |
| [`api/types.ts`](src/api/types.ts.md)             | Types shared with the core.                         |
| [`api/client.ts`](src/api/client.ts.md)           | `call()`, `ApiError` and error messages.            |
| [`api/client.test.ts`](src/api/client.test.ts.md) | Tests for the command client.                       |
| [`api/vault.ts`](src/api/vault.ts.md)             | Vault commands.                                     |
| [`api/entries.ts`](src/api/entries.ts.md)         | Entry commands.                                     |
| [`api/files.ts`](src/api/files.ts.md)             | File commands.                                      |
| [`api/backup.ts`](src/api/backup.ts.md)           | Backup commands.                                    |
| [`api/security.ts`](src/api/security.ts.md)       | Activity, copy and lock-event bindings.             |
| [`api/api.test.ts`](src/api/api.test.ts.md)       | Tests for every command wrapper and the lock event. |

### Screens (`src/screens/`)

| File                                                                    | Description                                    |
| ----------------------------------------------------------------------- | ---------------------------------------------- |
| [`screens/SetupScreen.tsx`](src/screens/SetupScreen.tsx.md)             | Create the vault or restore a backup.          |
| [`screens/SetupScreen.test.tsx`](src/screens/SetupScreen.test.tsx.md)   | Tests for the setup screen.                    |
| [`screens/UnlockScreen.tsx`](src/screens/UnlockScreen.tsx.md)           | Unlock with the master password.               |
| [`screens/UnlockScreen.test.tsx`](src/screens/UnlockScreen.test.tsx.md) | Tests for the unlock screen.                   |
| [`screens/VaultScreen.tsx`](src/screens/VaultScreen.tsx.md)             | Unlocked vault: tabs and Lock button.          |
| [`screens/VaultScreen.test.tsx`](src/screens/VaultScreen.test.tsx.md)   | Tests for the unlocked vault screen.           |
| [`screens/PasswordsTab.tsx`](src/screens/PasswordsTab.tsx.md)           | Password entries.                              |
| [`screens/PasswordsTab.test.tsx`](src/screens/PasswordsTab.test.tsx.md) | Tests for the password entries section.        |
| [`screens/FilesTab.tsx`](src/screens/FilesTab.tsx.md)                   | Encrypted files.                               |
| [`screens/FilesTab.test.tsx`](src/screens/FilesTab.test.tsx.md)         | Tests for the files section.                   |
| [`screens/SettingsTab.tsx`](src/screens/SettingsTab.tsx.md)             | Settings: master password, backup and restore. |
| [`screens/SettingsTab.test.tsx`](src/screens/SettingsTab.test.tsx.md)   | Tests for the settings section.                |

### Components and hooks (`src/components/`, `src/hooks/`)

| File                                                                          | Description                              |
| ----------------------------------------------------------------------------- | ---------------------------------------- |
| [`components/PasswordInput.tsx`](src/components/PasswordInput.tsx.md)         | Password field with Show/Hide.           |
| [`components/ErrorBanner.tsx`](src/components/ErrorBanner.tsx.md)             | Announced error message.                 |
| [`components/ConfirmDialog.tsx`](src/components/ConfirmDialog.tsx.md)         | Confirmation for destructive actions.    |
| [`components/EntryList.tsx`](src/components/EntryList.tsx.md)                 | List of entries.                         |
| [`components/EntryForm.tsx`](src/components/EntryForm.tsx.md)                 | Create or edit an entry.                 |
| [`components/EntryView.tsx`](src/components/EntryView.tsx.md)                 | Read-only entry with masked password.    |
| [`components/entries.test.tsx`](src/components/entries.test.tsx.md)           | Tests for the entry components.          |
| [`components/components.test.tsx`](src/components/components.test.tsx.md)     | Tests for the small components.          |
| [`components/CreateBackupPanel.tsx`](src/components/CreateBackupPanel.tsx.md) | Create an encrypted backup.              |
| [`components/RestoreBackupForm.tsx`](src/components/RestoreBackupForm.tsx.md) | Restore a backup.                        |
| [`components/backup.test.tsx`](src/components/backup.test.tsx.md)             | Tests for the backup components.         |
| [`hooks/useApiError.ts`](src/hooks/useApiError.ts.md)                         | Error state that handles a locked vault. |
| [`hooks/useActivityReporter.ts`](src/hooks/useActivityReporter.ts.md)         | Reports user activity to the core.       |
| [`hooks/useVaultLockedEvent.ts`](src/hooks/useVaultLockedEvent.ts.md)         | Reacts to the lock event.                |
| [`hooks/hooks.test.tsx`](src/hooks/hooks.test.tsx.md)                         | Tests for the hooks.                     |

### Tests (`src/test/`)

| File                                            | Description                                   |
| ----------------------------------------------- | --------------------------------------------- |
| [`test/setup.ts`](src/test/setup.ts.md)         | Vitest setup and Tauri mock.                  |
| [`test/tauriMock.ts`](src/test/tauriMock.ts.md) | Helpers to fake the Tauri commands.           |
| [`test/fakeCore.ts`](src/test/fakeCore.ts.md)   | Stateful in-memory core for end-to-end tests. |

### End-to-end tests (`src/e2e/`)

| File                                              | Description                                  |
| ------------------------------------------------- | -------------------------------------------- |
| [`e2e/flows.test.tsx`](src/e2e/flows.test.tsx.md) | Main user flows through the whole interface. |

## Core (`src-tauri/`)

| File                                                                            | Description                               |
| ------------------------------------------------------------------------------- | ----------------------------------------- |
| [`src-tauri/src/main.rs`](src-tauri/src/main.rs.md)                             | Desktop binary entry point.               |
| [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs.md)                               | Builds and runs the Tauri application.    |
| [`src-tauri/build.rs`](src-tauri/build.rs.md)                                   | Tauri build script.                       |
| [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json.md)                     | Tauri configuration, CSP and window.      |
| [`src-tauri/capabilities/default.json`](src-tauri/capabilities/default.json.md) | Minimal window capability.                |
| [`src-tauri/src/error.rs`](src-tauri/src/error.rs.md)                           | `AppError`, serialized for the interface. |
| [`src-tauri/src/state.rs`](src-tauri/src/state.rs.md)                           | Shared state: the vault behind a mutex.   |

### Commands (`src-tauri/src/commands/`)

| File                                                            | Description                                    |
| --------------------------------------------------------------- | ---------------------------------------------- |
| [`commands/mod.rs`](src-tauri/src/commands/mod.rs.md)           | Module entry point.                            |
| [`commands/vault.rs`](src-tauri/src/commands/vault.rs.md)       | Create, unlock, lock, change master password.  |
| [`commands/entries.rs`](src-tauri/src/commands/entries.rs.md)   | List, read, create, update and delete entries. |
| [`commands/files.rs`](src-tauri/src/commands/files.rs.md)       | File commands with native dialogs.             |
| [`commands/backup.rs`](src-tauri/src/commands/backup.rs.md)     | Backup commands with native dialogs.           |
| [`commands/security.rs`](src-tauri/src/commands/security.rs.md) | Activity and copying to the clipboard.         |

### Vault (`src-tauri/src/vault/`)

| File                                                                    | Description                                         |
| ----------------------------------------------------------------------- | --------------------------------------------------- |
| [`vault/mod.rs`](src-tauri/src/vault/mod.rs.md)                         | Module entry point.                                 |
| [`vault/header.rs`](src-tauri/src/vault/header.rs.md)                   | Key header: Argon2id parameters, salt, wrapped key. |
| [`vault/password_policy.rs`](src-tauri/src/vault/password_policy.rs.md) | Master password rules.                              |
| [`vault/session.rs`](src-tauri/src/vault/session.rs.md)                 | `Vault`: create, unlock, lock, change password.     |
| [`vault/entries.rs`](src-tauri/src/vault/entries.rs.md)                 | Encrypted password entries and their validation.    |
| [`vault/files.rs`](src-tauri/src/vault/files.rs.md)                     | Encrypted files: import, list, export, delete.      |
| [`vault/backup.rs`](src-tauri/src/vault/backup.rs.md)                   | Encrypted backup and restore.                       |

### Security (`src-tauri/src/security/`)

| File                                                                                    | Description                                                  |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| [`security/mod.rs`](src-tauri/src/security/mod.rs.md)                                   | Idle and Windows lock watchers, clipboard timer, lock event. |
| [`security/autolock.rs`](src-tauri/src/security/autolock.rs.md)                         | Inactivity tracking (5 minutes).                             |
| [`security/clipboard.rs`](src-tauri/src/security/clipboard.rs.md)                       | Copy secrets and clear them after 30 seconds.                |
| [`security/session_lock_windows.rs`](src-tauri/src/security/session_lock_windows.rs.md) | Detect the Windows session lock.                             |

### Storage (`src-tauri/src/storage/`)

| File                                                | Description                          |
| --------------------------------------------------- | ------------------------------------ |
| [`storage/mod.rs`](src-tauri/src/storage/mod.rs.md) | Module entry point.                  |
| [`storage/db.rs`](src-tauri/src/storage/db.rs.md)   | SQLite schema and raw record access. |

### Cryptographic core (`src-tauri/src/crypto/`)

| File                                                    | Description                                      |
| ------------------------------------------------------- | ------------------------------------------------ |
| [`crypto/mod.rs`](src-tauri/src/crypto/mod.rs.md)       | Module entry point and `CryptoError`.            |
| [`crypto/keys.rs`](src-tauri/src/crypto/keys.rs.md)     | `SecretKey`, wiped from memory on drop.          |
| [`crypto/random.rs`](src-tauri/src/crypto/random.rs.md) | Random bytes and identifiers.                    |
| [`crypto/kdf.rs`](src-tauri/src/crypto/kdf.rs.md)       | Argon2id key derivation.                         |
| [`crypto/aead.rs`](src-tauri/src/crypto/aead.rs.md)     | XChaCha20-Poly1305 for single records.           |
| [`crypto/stream.rs`](src-tauri/src/crypto/stream.rs.md) | Chunked STREAM encryption for files and backups. |

### Integration tests (`src-tauri/tests/`)

| File                                                                      | Description                                          |
| ------------------------------------------------------------------------- | ---------------------------------------------------- |
| [`tests/common/mod.rs`](src-tauri/tests/common/mod.rs.md)                 | Shared fixtures: workspace, entries, clipboard.      |
| [`tests/vault_lifecycle.rs`](src-tauri/tests/vault_lifecycle.rs.md)       | Create, lock, unlock, change password, reopen.       |
| [`tests/entries_files_flow.rs`](src-tauri/tests/entries_files_flow.rs.md) | Entries and files across instances, at rest.         |
| [`tests/backup_flow.rs`](src-tauri/tests/backup_flow.rs.md)               | Restore elsewhere; damaged backups refused.          |
| [`tests/security.rs`](src-tauri/tests/security.rs.md)                     | No leaks in errors, input checks, clipboard on lock. |

## Tooling

| File                                      | Description                    |
| ----------------------------------------- | ------------------------------ |
| [`vite.config.ts`](vite.config.ts.md)     | Vite and Vitest configuration. |
| [`eslint.config.js`](eslint.config.js.md) | ESLint configuration.          |
