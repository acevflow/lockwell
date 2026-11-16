# Code file index

One document per code file. Each document describes what the file does and
how it fits in the project.

## Interface (`src/`)

| File                                        | Description                         |
| ------------------------------------------- | ----------------------------------- |
| [`src/main.tsx`](src/main.tsx.md)           | Mounts the React app into the page. |
| [`src/App.tsx`](src/App.tsx.md)             | Root component of the interface.    |
| [`src/App.test.tsx`](src/App.test.tsx.md)   | Smoke test for the root component.  |
| [`src/test/setup.ts`](src/test/setup.ts.md) | Vitest setup for Testing Library.   |
| [`src/vite-env.d.ts`](src/vite-env.d.ts.md) | Vite client type declarations.      |

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
