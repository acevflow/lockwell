# Changelog

All notable changes to lockwell will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/acevflow/lockwell/commits/main
