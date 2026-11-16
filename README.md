# lockwell

**A local-only, encrypted password and file manager for the desktop.**

lockwell keeps your passwords and private files in an encrypted vault on your
own computer. It never connects to the network, has no cloud account and sends
no telemetry: your data stays on your device, protected by a master password
that only you know.

It is built for people who want a simple, open-source vault for their secrets
without trusting a third-party service. The first target platform is Windows;
macOS and Linux will follow.

> [!WARNING]
> lockwell 0.1 is a beta that has **not been audited**. Do not rely on it as
> the only place where you keep important passwords or files.

## Features

- **100% local.** No network connections, no accounts, no telemetry.
- **Strong, standard cryptography.** Vault data is encrypted with
  XChaCha20-Poly1305, and the encryption key is derived from the master
  password with Argon2id. Only standard, audited libraries are used.
- **Encrypted password storage.** Entries (title, username, password, website
  and notes) are stored in a local SQLite database where every record is
  encrypted individually. Search the list, and reveal or copy a password only
  when you need it.
- **Encrypted files.** Files are stored as encrypted chunks inside the vault
  folder, and their file names are encrypted too.
- **Encrypted backups** of the whole vault in a single file, which can be
  restored over the current vault or on a new computer.
- **Automatic locking** after 5 minutes of inactivity and whenever Windows is
  locked. Secrets are wiped from memory when the vault locks.
- **Clipboard protection.** Copied passwords are cleared from the clipboard
  after 30 seconds and, on Windows, kept out of the clipboard history and
  cloud sync.
- **Master password** of at least 12 characters, which can be changed. There is
  no recovery mechanism: if you forget it, your data cannot be recovered.

## Project status

lockwell **0.1.0** is the first beta. It has not been audited.

| Phase | Scope                                                 | Status |
| ----- | ----------------------------------------------------- | ------ |
| 1     | Foundations: repository, skeleton, CI, docs           | Done   |
| 2     | Cryptographic core                                    | Done   |
| 3     | Vault and passwords, including master password change | Done   |
| 4     | Files                                                 | Done   |
| 5     | Encrypted backup                                      | Done   |
| 6     | Polish and v0.1 beta (not audited)                    | Done   |

Your vault is stored in `%APPDATA%\com.acevflow.lockwell\vault`. Keep encrypted
backups somewhere else, such as an external drive.

See the [changelog](CHANGELOG.md) for what has changed so far.

## Tech stack

- **Desktop framework:** [Tauri 2](https://tauri.app/), with the core written
  in Rust.
- **Interface:** TypeScript and React.
- **Encryption:** XChaCha20-Poly1305.
- **Key derivation:** Argon2id.
- **Storage:** local SQLite database with each record encrypted; files as
  encrypted chunked files in the vault folder.
- **Testing:** Rust tests, Vitest and Testing Library.
- **Code quality:** rustfmt, clippy, ESLint and Prettier.
- **CI:** GitHub Actions.

## Getting started

### Requirements

- [Git](https://git-scm.com/).
- The [Rust](https://www.rust-lang.org/tools/install) toolchain, installed with
  `rustup`.
- [Node.js](https://nodejs.org/) with npm.
- The system dependencies listed in the official
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) for
  your operating system.

### Getting the code

```powershell
git clone https://github.com/acevflow/lockwell.git
cd lockwell
```

### Common commands

Run these from the repository root:

| Task                         | Command                 |
| ---------------------------- | ----------------------- |
| Install dependencies         | `npm ci`                |
| Run the app in development   | `npm run tauri dev`     |
| Build the desktop app        | `npm run tauri build`   |
| Run the tests (Vitest, Rust) | `npm test`              |
| Run the tests with coverage  | `npm run test:coverage` |
| Lint (ESLint, clippy)        | `npm run lint`          |
| Check formatting             | `npm run format:check`  |
| Format the code              | `npm run format`        |
| Type check                   | `npm run typecheck`     |

See the [contributing guide](CONTRIBUTING.md) for more details.

## Project structure

The layout of the repository is:

```text
lockwell/
├── src-tauri/        # Rust core: cryptography, vault and storage
│   ├── src/          # Rust source code
│   ├── capabilities/ # Tauri permissions for the app windows
│   ├── icons/        # Application icons
│   └── tauri.conf.json
├── src/              # TypeScript and React interface
│   └── test/         # Test setup
├── docs/             # Project documentation
│   └── files/        # One document per code file, with an index (README.md)
├── .github/          # CI workflows and security configuration
├── index.html        # Interface entry page
├── package.json      # npm scripts and dependencies
├── vite.config.ts    # Vite and Vitest configuration
├── eslint.config.js  # ESLint configuration
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── LICENSE
├── README.md
└── SECURITY.md
```

## Security

Security is the core of this project. If you find a vulnerability, **do not
open a public issue**. Report it privately through
[GitHub private vulnerability reporting](https://github.com/acevflow/lockwell/security/advisories/new)
or by email to [acevflow@gmail.com](mailto:acevflow@gmail.com).

Read the [security policy](SECURITY.md) for the supported versions, what to
include in a report, what to expect and the scope.

## Contributing

Contributions are welcome. Please read:

- [Contributing guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [License](LICENSE)

## License

Copyright (C) 2026 acevflow.

lockwell is free software: you can redistribute it and/or modify it under the
terms of the [GNU General Public License](LICENSE) as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version (GPL-3.0-or-later).

lockwell is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE. See the [LICENSE](LICENSE) file for details.
