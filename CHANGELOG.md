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

[Unreleased]: https://github.com/acevflow/lockwell/commits/main
