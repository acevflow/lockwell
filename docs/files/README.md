# Code file index

One document per code file. Each document describes what the file does and
how it fits in the project.

## Interface (`src/`)

| File                                                        | Description                                             |
| ----------------------------------------------------------- | ------------------------------------------------------- |
| [`src/main.tsx`](src/main.tsx.md)                           | Mounts the React app into the page.                     |
| [`src/App.tsx`](src/App.tsx.md)                             | Root component: picks the screen from the vault status. |
| [`src/App.test.tsx`](src/App.test.tsx.md)                   | Tests for the root component.                           |
| [`src/styles.css`](src/styles.css.md)                       | Global styles, light and dark themes.                   |
| [`src/passwordRules.ts`](src/passwordRules.ts.md)           | Master password rules mirrored from the core.           |
| [`src/passwordRules.test.ts`](src/passwordRules.test.ts.md) | Tests for the master password rules.                    |
| [`src/entryRules.ts`](src/entryRules.ts.md)                 | Entry field limits and search.                          |
| [`src/entryRules.test.ts`](src/entryRules.test.ts.md)       | Tests for the entry search.                             |
| [`src/vite-env.d.ts`](src/vite-env.d.ts.md)                 | Vite client type declarations.                          |

### Command client (`src/api/`)

| File                                              | Description                              |
| ------------------------------------------------- | ---------------------------------------- |
| [`api/types.ts`](src/api/types.ts.md)             | Types shared with the core.              |
| [`api/client.ts`](src/api/client.ts.md)           | `call()`, `ApiError` and error messages. |
| [`api/client.test.ts`](src/api/client.test.ts.md) | Tests for the command client.            |
| [`api/vault.ts`](src/api/vault.ts.md)             | Vault commands.                          |
| [`api/entries.ts`](src/api/entries.ts.md)         | Entry commands.                          |

### Screens (`src/screens/`)

| File                                                                    | Description                             |
| ----------------------------------------------------------------------- | --------------------------------------- |
| [`screens/SetupScreen.tsx`](src/screens/SetupScreen.tsx.md)             | Create the vault.                       |
| [`screens/SetupScreen.test.tsx`](src/screens/SetupScreen.test.tsx.md)   | Tests for the setup screen.             |
| [`screens/UnlockScreen.tsx`](src/screens/UnlockScreen.tsx.md)           | Unlock with the master password.        |
| [`screens/UnlockScreen.test.tsx`](src/screens/UnlockScreen.test.tsx.md) | Tests for the unlock screen.            |
| [`screens/VaultScreen.tsx`](src/screens/VaultScreen.tsx.md)             | Unlocked vault: tabs and Lock button.   |
| [`screens/PasswordsTab.tsx`](src/screens/PasswordsTab.tsx.md)           | Password entries.                       |
| [`screens/PasswordsTab.test.tsx`](src/screens/PasswordsTab.test.tsx.md) | Tests for the password entries section. |
| [`screens/SettingsTab.tsx`](src/screens/SettingsTab.tsx.md)             | Settings: change the master password.   |
| [`screens/SettingsTab.test.tsx`](src/screens/SettingsTab.test.tsx.md)   | Tests for the settings section.         |

### Components and hooks (`src/components/`, `src/hooks/`)

| File                                                                      | Description                              |
| ------------------------------------------------------------------------- | ---------------------------------------- |
| [`components/PasswordInput.tsx`](src/components/PasswordInput.tsx.md)     | Password field with Show/Hide.           |
| [`components/ErrorBanner.tsx`](src/components/ErrorBanner.tsx.md)         | Announced error message.                 |
| [`components/ConfirmDialog.tsx`](src/components/ConfirmDialog.tsx.md)     | Confirmation for destructive actions.    |
| [`components/EntryList.tsx`](src/components/EntryList.tsx.md)             | List of entries.                         |
| [`components/EntryForm.tsx`](src/components/EntryForm.tsx.md)             | Create or edit an entry.                 |
| [`components/EntryView.tsx`](src/components/EntryView.tsx.md)             | Read-only entry with masked password.    |
| [`components/components.test.tsx`](src/components/components.test.tsx.md) | Tests for the small components.          |
| [`hooks/useApiError.ts`](src/hooks/useApiError.ts.md)                     | Error state that handles a locked vault. |

### Tests (`src/test/`)

| File                                            | Description                         |
| ----------------------------------------------- | ----------------------------------- |
| [`test/setup.ts`](src/test/setup.ts.md)         | Vitest setup and Tauri mock.        |
| [`test/tauriMock.ts`](src/test/tauriMock.ts.md) | Helpers to fake the Tauri commands. |

## Core (`src-tauri/`)

| File                                                  | Description                               |
| ----------------------------------------------------- | ----------------------------------------- |
| [`src-tauri/src/main.rs`](src-tauri/src/main.rs.md)   | Desktop binary entry point.               |
| [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs.md)     | Builds and runs the Tauri application.    |
| [`src-tauri/build.rs`](src-tauri/build.rs.md)         | Tauri build script.                       |
| [`src-tauri/src/error.rs`](src-tauri/src/error.rs.md) | `AppError`, serialized for the interface. |
| [`src-tauri/src/state.rs`](src-tauri/src/state.rs.md) | Shared state: the vault behind a mutex.   |

### Commands (`src-tauri/src/commands/`)

| File                                                          | Description                                    |
| ------------------------------------------------------------- | ---------------------------------------------- |
| [`commands/mod.rs`](src-tauri/src/commands/mod.rs.md)         | Module entry point.                            |
| [`commands/vault.rs`](src-tauri/src/commands/vault.rs.md)     | Create, unlock, lock, change master password.  |
| [`commands/entries.rs`](src-tauri/src/commands/entries.rs.md) | List, read, create, update and delete entries. |

### Vault (`src-tauri/src/vault/`)

| File                                                                    | Description                                         |
| ----------------------------------------------------------------------- | --------------------------------------------------- |
| [`vault/mod.rs`](src-tauri/src/vault/mod.rs.md)                         | Module entry point.                                 |
| [`vault/header.rs`](src-tauri/src/vault/header.rs.md)                   | Key header: Argon2id parameters, salt, wrapped key. |
| [`vault/password_policy.rs`](src-tauri/src/vault/password_policy.rs.md) | Master password rules.                              |
| [`vault/session.rs`](src-tauri/src/vault/session.rs.md)                 | `Vault`: create, unlock, lock, change password.     |
| [`vault/entries.rs`](src-tauri/src/vault/entries.rs.md)                 | Encrypted password entries and their validation.    |

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

## Tooling

| File                                      | Description                    |
| ----------------------------------------- | ------------------------------ |
| [`vite.config.ts`](vite.config.ts.md)     | Vite and Vitest configuration. |
| [`eslint.config.js`](eslint.config.js.md) | ESLint configuration.          |
