# Changelog

All notable changes to lockwell will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `THIRD-PARTY-LICENSES.md`: licenses of the third-party software in a Linux
  build (the 278 Rust crates the runtime links, plus the system GTK and
  WebKitGTK libraries and what bundling them requires), to accompany Linux
  packages.

- Code coverage measurement for the interface (Vitest) and the Rust core
  (`npm run test:coverage`), also run in CI with the report uploaded.
- Many more automated tests: integration tests for the vault, files, backups
  and security in the Rust core, and tests for every command wrapper, hook,
  component and screen of the interface, plus end-to-end flows against a
  stateful fake core.
- Project documentation in `docs/`: an index, getting started, user guide,
  architecture, development guide, reference (commands, error codes, data and
  file formats) and a security document with the threat model. Linked from the
  README.
- `docs/manual-testing.md`: manual testing guide with 129 test cases
  (installation, vault, entries, files, master password change, backup and
  restore, persistence, security, interface, platforms and uninstallation), a
  smoke test, a results template, how to report defects and a traceability
  table to the automated tests. Linked from the README.
- `docs/files/`: a complete document for each of the 63 source code files, with
  its purpose, public API, internal structure (with diagrams where the flow is
  complex), dependencies, verified usage examples, edge cases, security notes
  and links to its tests, plus an index grouped by folder.

### Changed

- Internal refactoring of the Rust core and the interface (shared helpers for
  the file and backup commands, the vault and the initial list load; shorter
  functions; named constants). Behavior is unchanged.
- `docs/files/` now covers source code only. The short documents for tests,
  test support code and configuration files were removed, and each document
  links to the tests of its file.

### Fixed

- A password or username copied just before closing the app stayed on the
  clipboard, because the 30-second clearing timer ended with the process. The
  clipboard is now cleared when the app exits (only if it still holds the
  copied text).
- When adding several files failed part way, the Files list did not show the
  files that had already been added until the tab was opened again.
- File sizes just below a unit boundary were shown as "1000 KB" or "1000 MB"
  instead of "1 MB" or "1 GB".
- The vault was locked by the inactivity timer the moment a command that ran
  longer than 5 minutes (for example, adding a very large file) finished,
  instead of counting that command as activity.
- Saving a decrypted copy or a backup failed with "invalid filename" when the
  destination name was close to the 255-character limit, because the temporary
  file name was longer than the target's.

## [0.1.0] - 2026-11-16

First beta. It has not been audited.

### Added

- `README.md` with the project overview, planned features, status, tech stack,
  requirements, planned structure and license.
- `CONTRIBUTING.md` with how to report bugs, propose improvements, the project
  conventions and the pull request process.
- `CODE_OF_CONDUCT.md` based on the Contributor Covenant 2.1.
- `SECURITY.md` with the security policy and private vulnerability reporting.
- `LICENSE` with the GNU General Public License v3.0 or later.
- `.gitignore` for Rust, Tauri, Node.js and React, including secrets and
  local development data.
- Initial project structure: Tauri 2 desktop app skeleton with a React and
  TypeScript interface, Vitest and Testing Library, ESLint, Prettier, rustfmt
  and clippy.
- Cryptographic core: Argon2id key derivation, XChaCha20-Poly1305 record
  encryption and chunked STREAM encryption for large data, built on the
  RustCrypto crates and tested with published test vectors and failure cases.
- Vault storage: local SQLite database with encrypted records, a vault key
  wrapped with a key derived from the master password, and commands to create,
  unlock and lock the vault and to change the master password (at least
  12 characters).
- Password entries: title, username, password, URL and notes, each entry
  encrypted as a whole and bound to its id, with commands to list, read,
  create, update and delete them. Listing never returns passwords or notes.
- Interface: create the vault, unlock it, list and search password entries,
  view an entry with the password masked until shown, create, edit and delete
  entries, change the master password and lock the vault. Light and dark
  themes follow the system.
- Automatic lock after 5 minutes of inactivity and when Windows is locked;
  locking wipes the vault key from memory and the interface drops all
  decrypted data and explains why it locked.
- Copy a username or password to the clipboard straight from the core; it is
  cleared after 30 seconds (or when the vault locks) if still there, and on
  Windows it is kept out of the clipboard history and cloud sync.
- Encrypted files: add files to the vault, save decrypted copies and delete
  them. Content is encrypted in 64 KiB chunks and file names are encrypted
  too; the files on disk are named with random ids.
- Encrypted backup: save the whole vault (entries and files) in one
  `.lwbackup` file encrypted with the vault key, and restore it over the
  current vault or on a new computer. A restore is fully checked before it
  replaces anything.

### Security

- Strict Content Security Policy for the interface, frozen JavaScript
  prototypes, and a minimal window capability: the interface can only call the
  app commands and listen for the lock event. File dialogs and the clipboard
  are used from the core.

[Unreleased]: https://github.com/acevflow/lockwell/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/acevflow/lockwell/releases/tag/v0.1.0
