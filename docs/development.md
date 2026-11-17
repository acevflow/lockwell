# Development guide

**Purpose:** describe how the repository is organised and how to build, test,
change and extend lockwell.
**Audience:** contributors. Read [CONTRIBUTING.md](../CONTRIBUTING.md) first
for the contribution process (branches, commits, pull requests); this guide
does not repeat it.

## Table of contents

- [Project structure](#project-structure)
- [Environment](#environment)
- [Commands](#commands)
- [Tests](#tests)
- [Continuous integration](#continuous-integration)
- [Conventions](#conventions)
- [Extending the project](#extending-the-project)

## Project structure

```text
lockwell/
├── src-tauri/               # Rust core
│   ├── src/
│   │   ├── main.rs, lib.rs  # Entry point, app setup, command registration
│   │   ├── error.rs         # AppError and its {code, message} serialization
│   │   ├── state.rs         # AppState shared by commands and watchers
│   │   ├── commands/        # Tauri command handlers (vault, entries, files, backup, security)
│   │   ├── vault/           # Session, header, entries, files, backup, password policy
│   │   ├── crypto/          # kdf, aead, stream, keys, random
│   │   ├── storage/         # SQLite access
│   │   └── security/        # Idle lock, clipboard, Windows session lock
│   ├── tests/               # Integration tests (common/ holds fixtures)
│   ├── capabilities/        # Tauri permissions of the window
│   └── tauri.conf.json      # App identifier, window, CSP
├── src/                     # TypeScript and React interface
│   ├── api/                 # One wrapper per command, ApiError, shared types
│   ├── screens/             # Setup, Unlock, Vault and the three tabs
│   ├── components/          # Reusable pieces (forms, lists, dialogs)
│   ├── hooks/               # Activity reporting, lock event, initial load
│   ├── e2e/                 # End-to-end flows against the fake core
│   └── test/                # Vitest setup, tauriMock and fakeCore
├── scripts/                 # Maintenance scripts (third-party licenses)
├── docs/                    # This documentation
│   └── files/               # One document per code file
├── .github/                 # CI, CodeQL, Dependabot, issue and PR templates
└── package.json, vite.config.ts, eslint.config.js, tsconfig*.json
```

Unit tests sit next to the code (`*.test.ts` and `*.test.tsx` in `src/`,
`#[cfg(test)]` modules in Rust). Each code file has a matching document in
[`files/`](files/README.md); see the [architecture](architecture.md) for how
the pieces fit together.

## Environment

See [Getting started](getting-started.md#requirements) for the versions
verified with this documentation. Rust must include `rustfmt` and `clippy`.
For coverage you also need:

```powershell
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
```

Then install the exact locked dependencies:

```powershell
npm ci
```

## Commands

Run every command from the repository root.

| Task | Command |
| --- | --- |
| Run the app in development | `npm run tauri dev` |
| Start only the Vite dev server | `npm run dev` |
| Build the interface | `npm run build` |
| Build the desktop app | `npm run tauri build` |
| Run all tests (Vitest, then `cargo test`) | `npm test` |
| Run tests with coverage (reports in `coverage/frontend` and `coverage/core`) | `npm run test:coverage` |
| Lint (ESLint, then clippy with `-D warnings`) | `npm run lint` |
| Check formatting (Prettier, then `cargo fmt --check`) | `npm run format:check` |
| Apply formatting | `npm run format` |
| Type check (`tsc --noEmit` for the app and for the Node config) | `npm run typecheck` |
| Regenerate `THIRD-PARTY-LICENSES.md` from `Cargo.lock` (needs `cargo`) | `npm run licenses` |
| Check that `THIRD-PARTY-LICENSES.md` is up to date | `npm run licenses:check` |

To run a subset of the tests:

```powershell
# One Vitest file
npx vitest run src/screens/PasswordsTab.test.tsx

# One Rust integration suite, or tests whose name contains "backup"
cargo test --manifest-path src-tauri/Cargo.toml --test backup_flow
cargo test --manifest-path src-tauri/Cargo.toml backup
```

In debug builds the cryptographic crates (`argon2`, `blake2`, `chacha20`,
`poly1305`) are compiled with optimisations (`[profile.dev.package.*]` entries in
`src-tauri/Cargo.toml`), so key derivation is not unusably slow. Integration
tests use deliberately weak Argon2id parameters (64 KiB, 1 iteration, 1 lane)
from `src-tauri/tests/common/mod.rs`; never use them outside tests.

## Tests

| Layer | Tooling | Where |
| --- | --- | --- |
| Rust unit tests | `cargo test` | `#[cfg(test)]` modules next to the code. They include a reference vector for Argon2id, the XChaCha20-Poly1305 test vector, tamper, truncation and wrong-key cases. |
| Rust integration tests | `cargo test` | `src-tauri/tests/`: `vault_lifecycle`, `entries_files_flow`, `backup_flow`, `security`. |
| Interface unit and component tests | Vitest and Testing Library | `*.test.ts(x)` next to the code. |
| Interface end-to-end flows | Vitest | `src/e2e/flows.test.tsx`, using `src/test/fakeCore.ts` as an in-memory stand-in for the core. |
| Manual tests | by hand | [manual-testing.md](manual-testing.md). |

Interface tests never call the real core: `src/test/tauriMock.ts` replaces
`invoke`. `fakeCore.ts` implements the vault and entry commands, the clipboard
and `restore_backup`; it does not implement the file commands
(`import_files`, `export_file`, `delete_file`) or `create_backup`, and
`list_files` always returns an empty list. Those paths are covered by the Rust
integration tests and by component tests with individual mocks.

## Continuous integration

`.github/workflows/ci.yml` runs on pushes and pull requests to `main`, on
`windows-latest`. In order: checkout, Node.js, Rust (stable with clippy,
rustfmt, llvm-tools), Rust cache, `cargo-llvm-cov`, `npm ci`, `format:check`,
`licenses:check` (fails if `THIRD-PARTY-LICENSES.md` is out of date after a
`Cargo.lock` change; fix it with `npm run licenses`; for Dependabot pull
requests it only shows a warning, so regenerate the file before merging),
`typecheck`, `lint`,
`test:coverage`, upload of the coverage report and
`npm run tauri build`. `.github/workflows/codeql.yml` runs CodeQL for
JavaScript/TypeScript, Rust and GitHub Actions on pushes, pull requests and
weekly. Dependabot (`.github/dependabot.yml`) proposes weekly updates for npm,
Cargo and GitHub Actions.

## Releasing

To publish a version, update the version in `package.json`,
`package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and
`src-tauri/Cargo.lock`, turn `Unreleased` in `CHANGELOG.md` into the new
version, merge to `main` and push an annotated tag `vX.Y.Z`.
`.github/workflows/release.yml` runs on `v*` tags: it checks that the tag
matches the app version, builds the MSI and NSIS installers on
`windows-latest`, computes `SHA256SUMS.txt` and creates the GitHub release with
the notes of that version from `CHANGELOG.md`. The installers are not
code-signed.

A change is ready when `npm run format:check`, `npm run typecheck`,
`npm run lint` and `npm test` all pass.

## Conventions

- Branch, commit and pull request rules are in
  [CONTRIBUTING.md](../CONTRIBUTING.md#project-conventions).
- Rust: rustfmt and clippy without warnings. TypeScript and React: Prettier
  (default options) and ESLint, with strict TypeScript.
- Comments and documentation comments are in English. Update them in the same
  commit as the code they describe, and update the matching
  [`files/`](files/README.md) document. When you add or delete a code file,
  add or delete its document and update the index.
- Cryptographic code uses published, audited crates only, and every
  cryptographic function has tests with known vectors and failure cases.
- Secrets are wrapped in types that wipe on drop (`SecretKey`, `Zeroizing`,
  zeroized structs) and never appear in errors, logs or `Debug` output.
- Anything that reaches a file path or SQL must be validated as an id first
  (`is_valid_id` in `src-tauri/src/crypto/random.rs`).
- The core must not open network connections.

## Extending the project

### Add a command

Example: a command that needs an unlocked vault.

1. **Vault logic.** Add the operation to the right module in
   `src-tauri/src/vault/` (for example a method on `UnlockedVault`), with unit
   tests that cover success, a locked vault and invalid input. Return
   `AppResult` and use an existing `AppError` variant, or add one in
   `src-tauri/src/error.rs` together with its `code` string.
2. **Handler.** Add a thin `#[tauri::command]` function in
   `src-tauri/src/commands/`, calling `state.with_vault(|vault| ...)` so the
   call counts as activity. For slow work (key derivation, file encryption)
   make the command `async` and use `state.with_vault_blocking(...)`, as
   `create_vault` and `import_files` do. Do not return secrets unless the
   interface truly needs them.
3. **Register it** in `tauri::generate_handler![...]` in
   `src-tauri/src/lib.rs`.
4. **Permissions.** Custom commands registered this way need no change in
   `src-tauri/capabilities/default.json`, which only allows event listening.
   If you use a Tauri plugin from the interface, add its permission there and
   check the Content Security Policy in `src-tauri/tauri.conf.json`.
5. **Interface wrapper.** Add a typed function in `src/api/` that calls
   `call<T>("command_name", { camelCaseArgs })` and extend
   `src/api/types.ts` (including `ErrorCode` if you added a code).
6. **Tests.** Add API, component and integration tests; extend
   `src/test/fakeCore.ts` if end-to-end flows should cover it.
7. **Docs.** Update the matching `docs/files/` documents and, if the behavior
   is visible, the [reference](reference.md), the
   [user guide](user-guide.md) and `CHANGELOG.md`.

### Change a stored format

Every format has a version (`FORMAT_VERSION` in `vault/header.rs`,
`SCHEMA_VERSION` in `storage/db.rs`, `BACKUP_FORMAT_VERSION` in
`vault/backup.rs`). Opening a newer version fails with `unsupported_version`;
an unknown older database version fails with `corrupt`. A format change must
bump the version, keep reading older versions or provide a migration, and be
reflected in the [reference](reference.md#on-disk-formats).

### Add a platform

The idle lock and clipboard code are cross-platform. The session-lock watcher
exists only on Windows (`security/session_lock_windows.rs`, compiled with
`#[cfg(windows)]`), and the text of the lock notice in `src/App.tsx` mentions
Windows. A new platform needs its own watcher, an update of that text and an
entry in the CI matrix in `.github/workflows/ci.yml`.
