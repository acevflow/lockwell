# Security

**Purpose:** describe what lockwell protects, against whom, how, and where its
protection ends.
**Audience:** users deciding whether to trust lockwell, and contributors and
reviewers working on security-sensitive code.

> [!WARNING]
> lockwell 0.1 is a beta that has **not been audited**. Do not rely on it as
> the only place where you keep important passwords or files.

This document explains the design. The policy for reporting vulnerabilities,
supported versions and scope is in [SECURITY.md](../SECURITY.md); the short
version is [at the end of this page](#reporting-a-vulnerability).

## Table of contents

- [What is protected](#what-is-protected)
- [Threat model](#threat-model)
- [How the protection works](#how-the-protection-works)
- [Assumptions](#assumptions)
- [Limits and known caveats](#limits-and-known-caveats)
- [Reporting a vulnerability](#reporting-a-vulnerability)

## What is protected

- **Confidentiality** of passwords, usernames, websites, notes, file contents
  and file names stored in the vault.
- **Integrity** of the vault: a modified, swapped, truncated or reordered
  record, file or backup is detected and rejected.
- **Secrets in use**: the master password, the vault key and decrypted data are
  kept in memory only while needed.

Not hidden: the existence of the vault folder, the number and approximate size
of entries and files (each record and file is a separate blob), and file and
modification times of the vault files, which the operating system records.

## Threat model

| Scenario | In scope? | Outcome |
| --- | --- | --- |
| Someone obtains a copy of the vault folder or of a backup (stolen disk, cloud storage of the backup, lost drive) | Yes | They get encrypted data. Guessing needs the master password, and each guess costs one Argon2id derivation (64 MiB of memory, 3 iterations by default). A strong master password is the main defence. |
| Someone modifies the vault files or a backup | Yes | Authentication fails (`corrupt` or `wrong_password`). Records cannot be moved between ids because each one is bound to its id as associated data. |
| Someone walks up to the computer while the vault is locked | Yes | The vault is locked and needs the master password. |
| Someone walks up while the vault is unlocked | Reduced | The vault locks after 5 minutes of inactivity and when Windows locks. Until then, anyone using the session can read it. |
| A crafted backup or header tries to exhaust memory or write outside the vault | Yes | Argon2id parameters are bounded, ids must be 32 lowercase hexadecimal characters, restore accepts only known file names, each once, and restore works in a staging folder. |
| Secrets left on the clipboard | Yes | The core clears the clipboard 30 seconds after a copy (only if it still holds the copied text), on lock and when the app exits. On Windows the copy is excluded from clipboard history and cloud sync. |
| Malware or a debugger running as the user, keyloggers, a compromised operating system | No | Such software can read the master password as it is typed and the memory of the running app. |
| Physical access with the vault unlocked, forced disclosure of the password | No | Not defended. |
| Forgotten master password | No | There is no recovery by design. |

## How the protection works

- **Encryption.** XChaCha20-Poly1305 for records, and its STREAM construction
  for files and backups (64 KiB chunks). Each record uses a fresh random
  24-byte nonce.
- **Key derivation.** The master password goes through Argon2id with a random
  salt to give a key-encryption key, which unwraps a random vault key. The
  parameters are stored in the header and bounded when read.
- **Binding.** Associated data ties each object to its id and format (see the
  [reference](reference.md#associated-data)), and the header parameters and
  salt are authenticated as associated data of the wrapped key.
- **No custom cryptography.** The primitives come from the RustCrypto crates
  (`argon2`, `chacha20poly1305`, `aead-stream`, `sha2`) and randomness from the
  operating system through `getrandom`, with no fallback.
- **Uniform failure.** A wrong password and a tampered header give the same
  error. Error messages carry only fixed text, validation messages and error
  kinds, never secrets or paths.
- **Memory.** The vault key (`SecretKey`), decrypted entries, JSON buffers,
  command passwords and stream buffers are wiped on drop. Locking drops the key
  and closes the database. `Debug` output of `SecretKey` is redacted.
- **Locking.** After 5 minutes without activity (checked every 5 seconds), on
  the Windows session lock, on the Lock button and after a restore. The app
  refuses to start on Windows if the session-lock watcher cannot be registered.
- **Clipboard.** The interface never receives the copied value. The core keeps
  only a SHA-256 digest and a counter to decide whether to clear, so an older
  timer cannot clear a newer copy.
- **Interface isolation.** A strict Content Security Policy (`default-src
  'self'`, connections only to the Tauri IPC, no objects, frames or form
  targets), `freezePrototype`, and a capability set that allows the window only
  to listen to events. File dialogs and the clipboard are driven by the core.
- **No network.** The core has no networking code, and the Content Security
  Policy only allows the local IPC. Nothing is sent anywhere, and there is no
  telemetry. The manual test plan includes a check of this
  ([manual-testing.md](manual-testing.md)).
- **Safe writes.** Files and exports are written to a temporary file, flushed
  and renamed. A restore is checked in a staging folder before it replaces the
  vault. SQLite runs with `secure_delete` on.
- **Unsafe code.** The only `unsafe` Rust is the Win32 session-lock watcher
  (`src-tauri/src/security/session_lock_windows.rs`).
- **Supply chain.** CI builds and tests on every change, CodeQL scans the
  repository weekly and on every change, Dependabot proposes updates, and the
  workflow actions are pinned to commit hashes.

## Assumptions

- The operating system, the Rust toolchain, the WebView2 runtime and the
  cryptographic crates behave correctly.
- The user chooses a strong, unique master password. The 12-character minimum
  is a floor, not a guarantee of strength.
- The computer is free of malware while the vault is in use.
- The operating system's random source is secure.

## Limits and known caveats

- **Not audited.** No independent security review has been done.
- **JavaScript memory.** When you reveal a password, or edit an entry, the
  decrypted text is a JavaScript string in the WebView. It cannot be wiped
  deterministically; the interface drops it when the tab changes or the vault
  locks, but the engine may keep it until garbage collection.
- **Lock timing.** The inactivity timer ignores time spent in a native file
  dialog, so the vault can lock while a dialog is open. The operation then
  fails with `locked` after the dialog closes. Lock requests also wait for a
  running vault operation (for example a large import or backup) to finish,
  because commands hold the vault for their whole duration.
- **Activity granularity.** The interface reports activity at most every 15
  seconds, so the lock can come up to 15 seconds earlier than 5 minutes, never
  later.
- **Fixed settings.** The lock time, clipboard time and Argon2id cost are
  compiled in and cannot be changed by the user.
- **Backups carry the old key header.** A backup opens with the master password
  the vault had when the backup was made, not the current one. Old backups
  stay as weak as the old password.
- **Export overwrites.** `export_file` replaces the destination chosen in the
  save dialog; it relies on the dialog's confirmation.
- **Crash leftovers.** A forced shutdown can leave temporary files (listed in
  the [reference](reference.md#vault-folder)) next to the vault. They hold only
  encrypted data.
- **Single platform.** Only Windows is supported. Locking when the operating system
  session locks does not exist on other platforms yet.
- **Wiping depends on the crates.** `SecretKey` is `Clone`; clones are wiped on
  drop, but copies made by the operating system (swap, hibernation, crash
  dumps) are outside the app's control.
- **Clipboard exclusion** from history and cloud sync relies on the Windows
  clipboard monitoring flags set through the `arboard` crate. It is checked by
  manual tests rather than automated tests.

## Reporting a vulnerability

Do **not** open a public issue, discussion or pull request.

1. Open the repository's
   [private vulnerability report form](https://github.com/acevflow/lockwell/security/advisories/new)
   (Security tab, **Report a vulnerability**).
2. Describe the problem, the steps to reproduce it, the affected version or
   commit, your Windows version and the impact. Use only made-up test data:
   never include real passwords, vault files or personal data.
3. As an alternative, email [acevflow@gmail.com](mailto:acevflow@gmail.com)
   with a subject starting `[lockwell security]`.

You can expect an acknowledgement within 7 days and an assessment within 30
days; these are goals for a single-maintainer project, not guarantees. Details
about supported versions, scope and coordinated disclosure are in
[SECURITY.md](../SECURITY.md).
