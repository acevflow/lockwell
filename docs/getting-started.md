# Getting started

**Purpose:** get lockwell running from source, understand its configuration
and complete a first session.
**Audience:** people who want to build and try lockwell. Contributors should
continue with the [development guide](development.md).

lockwell 0.1.0 installers for Windows (MSI and NSIS) are on the
[releases page](https://github.com/acevflow/lockwell/releases/latest), with a
`SHA256SUMS.txt` file to verify them. They are not code-signed, so Windows
SmartScreen may warn you. This guide shows how to build it from source.

## Table of contents

- [Requirements](#requirements)
- [Installation](#installation)
- [Configuration](#configuration)
- [First use](#first-use)
- [Where your data lives](#where-your-data-lives)

## Requirements

- Windows. It is the only supported platform. macOS and Linux are planned.
- [Git](https://git-scm.com/).
- Node.js and npm. CI uses Node 26; the project was verified with Node 26.13
  and npm 11.
- A Rust toolchain installed with `rustup`. The crate uses Rust edition 2024,
  so it needs a recent stable release; the project was verified with Rust
  1.99.
- The system dependencies from the official
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) (on
  Windows: Microsoft C++ Build Tools and WebView2).

## Installation

Run these commands from PowerShell:

```powershell
git clone https://github.com/acevflow/lockwell.git
cd lockwell
npm ci
```

Start the app in development mode (Vite dev server on port 1420 and the
desktop window):

```powershell
npm run tauri dev
```

Build the installable desktop app:

```powershell
npm run tauri build
```

The executable is `src-tauri/target/release/lockwell.exe`. On Windows the
build also produces an MSI installer in `src-tauri/target/release/bundle/msi/`
(`lockwell_0.1.0_x64_en-US.msi`) and an NSIS installer in
`src-tauri/target/release/bundle/nsis/` (`lockwell_0.1.0_x64-setup.exe`). To
build only the interface, run `npm run build`.

## Configuration

lockwell has no configuration file and no settings screen. Every option is a
constant compiled into the app, so changing one means changing the source and
rebuilding.

| Option | Default | Defined in |
| --- | --- | --- |
| Inactivity before the vault locks | 5 minutes | `IDLE_TIMEOUT` in `src-tauri/src/security/autolock.rs` |
| How often the inactivity check runs | 5 seconds | `CHECK_INTERVAL` in the same file |
| Clipboard cleared after copying | 30 seconds | `CLEAR_AFTER` in `src-tauri/src/security/clipboard.rs` |
| Master password length | 12 to 1024 characters | `src-tauri/src/vault/password_policy.rs` |
| Argon2id memory, iterations, lanes (new vaults) | 64 MiB, 3, 4 | `KdfParams::DEFAULT` in `src-tauri/src/crypto/kdf.rs` |
| Entry field limits | title 200, username 500, password 4096, URL 2048, notes 20000 characters | `src-tauri/src/vault/entries.rs` |
| File name limit | 255 characters | `src-tauri/src/vault/files.rs` |
| Encryption chunk size for files and backups | 64 KiB | `src-tauri/src/crypto/stream.rs` |
| Backup file extension | `.lwbackup` | `src-tauri/src/vault/backup.rs` |
| Vault folder | `<app data dir>\vault` (see below) | `VAULT_DIR` in `src-tauri/src/lib.rs` |
| Window size | 1000 x 680, minimum 720 x 480 | `src-tauri/tauri.conf.json` |
| Development server port | 1420 (strict) | `vite.config.ts` |
| Activity reports from the interface | at most one every 15 seconds | `src/hooks/useActivityReporter.ts` |

The interface keeps copies of the password and entry limits
(`src/passwordRules.ts`, `src/entryRules.ts`) and the texts that mention
"5 minutes" and "30 seconds". If you change a Rust constant, update these
copies too. Nothing checks that they agree.

Already created vaults store their own Argon2id parameters in the header, so
changing `KdfParams::DEFAULT` only affects vaults created afterwards.

## First use

1. Start the app. With no vault, it shows the setup screen.
2. Enter a master password of at least 12 characters and confirm it.
3. Tick the box that says you understand there is no recovery, then create
   the vault. It opens unlocked on the **Passwords** tab.
4. Add an entry with **New entry**, then lock the vault with **Lock**.
5. Next time, unlock with the master password.

The [user guide](user-guide.md) covers every feature. On a machine that
already has a backup, the setup screen also offers **Restore from a backup
instead**.

## Where your data lives

The vault folder is inside the application data directory for the identifier
`com.acevflow.lockwell`. On Windows this is:

```text
%APPDATA%\com.acevflow.lockwell\vault\
├── vault.db      # SQLite database with the key header and encrypted records
└── files\        # one encrypted <id>.lwf file per stored file
```

Keep encrypted backups somewhere else, for example an external drive. See the
[reference](reference.md#on-disk-formats) for the exact formats.
