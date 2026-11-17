# Code file index

One document per source code file: 64 files in total. Each document follows the same
template: purpose, place in the architecture, public API, internal structure,
dependencies, a usage example, edge cases, security notes, and related files and tests.

The path of a document mirrors the path of the file and adds `.md`. For example,
`src-tauri/src/crypto/aead.rs` is described in
[`src-tauri/src/crypto/aead.rs.md`](src-tauri/src/crypto/aead.rs.md).

**Scope.** The index covers source code, the build script and the maintenance scripts. Tests, test support code,
configuration files (`package.json`, `tsconfig*.json`, `vite.config.ts`,
`eslint.config.js`, `tauri.conf.json`, the capability files), generated files, lock files,
`index.html`, the stylesheet and the CI workflows have no document of their own. Tests are
linked from the document of the file they test.

For the overall design, see the [architecture](../architecture.md) and the
[reference](../reference.md).

## Core (`src-tauri/`)

| File                                         | Description                                                           |
| -------------------------------------------- | --------------------------------------------------------------------- |
| [`build.rs`](src-tauri/build.rs.md)          | Build script that prepares the Tauri configuration, capabilities and icons. |
| [`src/main.rs`](src-tauri/src/main.rs.md)    | Binary entry point. Calls `lockwell_lib::run()`.                      |
| [`src/lib.rs`](src-tauri/src/lib.rs.md)      | Builds and runs the app: plugin, state, watchers and the 18 commands. |
| [`src/error.rs`](src-tauri/src/error.rs.md)  | `AppError`, its stable codes and how it is serialized for the interface. |
| [`src/state.rs`](src-tauri/src/state.rs.md)  | Shared state: the vault, the activity clock and the clipboard, with the lock logic. |

### Cryptography (`src-tauri/src/crypto/`)

| File                                                  | Description                                                       |
| ----------------------------------------------------- | ----------------------------------------------------------------- |
| [`mod.rs`](src-tauri/src/crypto/mod.rs.md)            | Declares the layer and defines `CryptoError`.                     |
| [`aead.rs`](src-tauri/src/crypto/aead.rs.md)          | XChaCha20-Poly1305 encryption of single records with associated data. |
| [`kdf.rs`](src-tauri/src/crypto/kdf.rs.md)            | Argon2id key derivation and validation of its parameters.         |
| [`keys.rs`](src-tauri/src/crypto/keys.rs.md)          | `SecretKey`, a 256-bit key wiped on drop and never printed.       |
| [`random.rs`](src-tauri/src/crypto/random.rs.md)      | Random bytes and identifiers from the operating system, and id validation. |
| [`stream.rs`](src-tauri/src/crypto/stream.rs.md)      | Chunked STREAM encryption of files and backups of any size.       |

### Storage (`src-tauri/src/storage/`)

| File                                          | Description                                                    |
| --------------------------------------------- | -------------------------------------------------------------- |
| [`mod.rs`](src-tauri/src/storage/mod.rs.md)   | Declares the storage layer.                                    |
| [`db.rs`](src-tauri/src/storage/db.rs.md)     | The SQLite database: schema, opening, and raw record access.   |

### Vault (`src-tauri/src/vault/`)

| File                                                          | Description                                                     |
| ------------------------------------------------------------- | --------------------------------------------------------------- |
| [`mod.rs`](src-tauri/src/vault/mod.rs.md)                     | Declares the layer and the helpers shared by its modules.       |
| [`header.rs`](src-tauri/src/vault/header.rs.md)               | Key header: Argon2id parameters, salt and the wrapped vault key. |
| [`password_policy.rs`](src-tauri/src/vault/password_policy.rs.md) | Rules for the master password.                              |
| [`session.rs`](src-tauri/src/vault/session.rs.md)             | `Vault`: create, unlock, lock and change the master password.   |
| [`entries.rs`](src-tauri/src/vault/entries.rs.md)             | Password entries: validation, encryption and CRUD.              |
| [`files.rs`](src-tauri/src/vault/files.rs.md)                 | Encrypted files: import, list, export and delete, with path checks. |
| [`backup.rs`](src-tauri/src/vault/backup.rs.md)               | Encrypted backup and restore of the whole vault.                |

### Security controls (`src-tauri/src/security/`)

| File                                                                  | Description                                                    |
| --------------------------------------------------------------------- | -------------------------------------------------------------- |
| [`mod.rs`](src-tauri/src/security/mod.rs.md)                          | Starts the watchers and timers, and emits the lock event.      |
| [`autolock.rs`](src-tauri/src/security/autolock.rs.md)                | Inactivity clock for the automatic lock.                       |
| [`clipboard.rs`](src-tauri/src/security/clipboard.rs.md)              | Copying secrets and clearing them from the clipboard.          |
| [`session_lock_windows.rs`](src-tauri/src/security/session_lock_windows.rs.md) | Windows only: detects the session lock (Win+L).       |

### Commands (`src-tauri/src/commands/`)

| File                                                | Description                                                       |
| --------------------------------------------------- | ----------------------------------------------------------------- |
| [`mod.rs`](src-tauri/src/commands/mod.rs.md)        | Declares the commands, the only entry points for the interface.   |
| [`dialog.rs`](src-tauri/src/commands/dialog.rs.md)  | Helpers shared by the commands that show native dialogs.          |
| [`vault.rs`](src-tauri/src/commands/vault.rs.md)    | Status, create, unlock, lock and change master password.          |
| [`entries.rs`](src-tauri/src/commands/entries.rs.md) | List, read, create, update and delete entries.                   |
| [`files.rs`](src-tauri/src/commands/files.rs.md)    | List, add, save a copy of and delete files.                       |
| [`backup.rs`](src-tauri/src/commands/backup.rs.md)  | Create and restore backups.                                       |
| [`security.rs`](src-tauri/src/commands/security.rs.md) | Activity reports and copying an entry field to the clipboard.  |

## Interface (`src/`)

| File                                           | Description                                                     |
| ---------------------------------------------- | --------------------------------------------------------------- |
| [`main.tsx`](src/main.tsx.md)                  | Mounts the app in the page.                                     |
| [`App.tsx`](src/App.tsx.md)                    | Root component: chooses the screen from the vault status.       |
| [`passwordRules.ts`](src/passwordRules.ts.md)  | Master password rules mirrored from the core.                   |
| [`entryRules.ts`](src/entryRules.ts.md)        | Entry field limits and the entry search.                        |
| [`format.ts`](src/format.ts.md)                | Display formatting (file sizes).                                |
| [`vite-env.d.ts`](src/vite-env.d.ts.md)        | Type declarations for Vite's client environment.                |

### Command client (`src/api/`)

| File                                    | Description                                                 |
| --------------------------------------- | ----------------------------------------------------------- |
| [`client.ts`](src/api/client.ts.md)     | `call()`, `ApiError` and error helpers.                     |
| [`types.ts`](src/api/types.ts.md)       | Types shared with the core, and the error codes.            |
| [`vault.ts`](src/api/vault.ts.md)       | Vault commands.                                             |
| [`entries.ts`](src/api/entries.ts.md)   | Entry commands.                                             |
| [`files.ts`](src/api/files.ts.md)       | File commands.                                              |
| [`backup.ts`](src/api/backup.ts.md)     | Backup commands.                                            |
| [`security.ts`](src/api/security.ts.md) | Activity, copy and lock-event bindings.                     |

### Hooks (`src/hooks/`)

| File                                                          | Description                                              |
| ------------------------------------------------------------- | -------------------------------------------------------- |
| [`useActivityReporter.ts`](src/hooks/useActivityReporter.ts.md) | Reports user activity to the core, throttled.          |
| [`useApiError.ts`](src/hooks/useApiError.ts.md)               | Error message state that routes `locked` errors elsewhere. |
| [`useInitialLoad.ts`](src/hooks/useInitialLoad.ts.md)         | Loads data on mount and ignores late results.            |
| [`useVaultLockedEvent.ts`](src/hooks/useVaultLockedEvent.ts.md) | Reacts to the core locking the vault.                  |

### Components (`src/components/`)

| File                                                            | Description                                              |
| --------------------------------------------------------------- | -------------------------------------------------------- |
| [`ConfirmDialog.tsx`](src/components/ConfirmDialog.tsx.md)      | Modal confirmation for destructive actions.              |
| [`CreateBackupPanel.tsx`](src/components/CreateBackupPanel.tsx.md) | Settings panel that creates a backup.                 |
| [`EntryForm.tsx`](src/components/EntryForm.tsx.md)              | Form to create or edit an entry.                         |
| [`EntryList.tsx`](src/components/EntryList.tsx.md)              | Selectable list of entries.                              |
| [`EntryView.tsx`](src/components/EntryView.tsx.md)              | Read-only view of an entry with copy buttons.            |
| [`ErrorBanner.tsx`](src/components/ErrorBanner.tsx.md)          | Inline error message.                                    |
| [`FileTable.tsx`](src/components/FileTable.tsx.md)              | Table of stored files.                                   |
| [`PasswordInput.tsx`](src/components/PasswordInput.tsx.md)      | Password field with a show and hide toggle.              |
| [`RestoreBackupForm.tsx`](src/components/RestoreBackupForm.tsx.md) | Form that restores a backup.                          |

### Screens (`src/screens/`)

| File                                                  | Description                                                   |
| ----------------------------------------------------- | ------------------------------------------------------------- |
| [`SetupScreen.tsx`](src/screens/SetupScreen.tsx.md)   | First run: create the vault or restore a backup.              |
| [`UnlockScreen.tsx`](src/screens/UnlockScreen.tsx.md) | Asks for the master password.                                 |
| [`VaultScreen.tsx`](src/screens/VaultScreen.tsx.md)   | The unlocked vault: tab bar and the active tab.               |
| [`PasswordsTab.tsx`](src/screens/PasswordsTab.tsx.md) | Entry list, search and the view, edit and create panels.      |
| [`FilesTab.tsx`](src/screens/FilesTab.tsx.md)         | List, add, save a copy of and delete files.                   |
| [`SettingsTab.tsx`](src/screens/SettingsTab.tsx.md)   | Change the master password, create and restore backups.       |

## Scripts (`scripts/`)

| File                                                                      | Description                                                         |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| [`third-party-licenses.mjs`](scripts/third-party-licenses.mjs.md)         | Generates and checks `THIRD-PARTY-LICENSES.md` from `Cargo.lock`.   |
