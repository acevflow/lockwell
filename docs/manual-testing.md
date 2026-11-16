# Manual testing guide

This guide explains how to test lockwell by hand, from installation to
uninstallation. It is written for testers who do not know the project: every
case says what to prepare, what to do and exactly what you should see.

## Contents

- [1. Purpose and scope](#1-purpose-and-scope)
- [2. How to use this guide](#2-how-to-use-this-guide)
- [3. Test environment](#3-test-environment)
- [4. Conventions](#4-conventions)
- [5. Smoke test](#5-smoke-test)
- [6. Test cases](#6-test-cases)
  - [6.1 Installation and first start](#61-installation-and-first-start)
  - [6.2 Creating the vault](#62-creating-the-vault)
  - [6.3 Unlocking and locking](#63-unlocking-and-locking)
  - [6.4 Password entries](#64-password-entries)
  - [6.5 Encrypted files](#65-encrypted-files)
  - [6.6 Changing the master password](#66-changing-the-master-password)
  - [6.7 Encrypted backup and restore](#67-encrypted-backup-and-restore)
  - [6.8 Persistence and integrity](#68-persistence-and-integrity)
  - [6.9 Security](#69-security)
  - [6.10 Interface and accessibility](#610-interface-and-accessibility)
  - [6.11 Platforms](#611-platforms)
  - [6.12 Uninstallation and cleanup](#612-uninstallation-and-cleanup)
- [7. Recording results](#7-recording-results)
- [8. Reporting defects](#8-reporting-defects)
- [9. Traceability to automated tests](#9-traceability-to-automated-tests)
- [10. Automated verification of this revision](#10-automated-verification-of-this-revision)

## 1. Purpose and scope

lockwell is a local-only, encrypted password and file manager for the desktop
(see the [README](../README.md)). This guide covers what the automated tests
cannot check on their own: the real desktop window, native file dialogs, the
Windows clipboard, Windows session locking, real timers, installers and what
is actually written to disk.

**In scope** (version 0.1.0):

- Installation from source and with the Windows installers (MSI and NSIS).
- Creating, unlocking and locking the vault.
- Password entries: create, view, search, edit, delete, copy.
- Encrypted files: add, list, save a decrypted copy, delete.
- Changing the master password.
- Encrypted backup and restore.
- Persistence and integrity of the data on disk.
- Security controls: automatic lock, lock on Windows session lock, clipboard
  clearing, no clear text on disk, no network connections.
- Uninstallation and cleanup.

**Out of scope:**

- macOS and Linux. They are not supported yet (see
  [6.11 Platforms](#611-platforms)).
- Security audits, memory forensics and penetration testing. Report suspected
  vulnerabilities privately as described in the [security policy](../SECURITY.md).
- GitHub repository settings (CodeQL, Dependabot, secret scanning), which are
  checked on GitHub, not in the app.

## 2. How to use this guide

1. Prepare the environment once, as described in
   [3. Test environment](#3-test-environment).
2. For a quick check of a build, run only the [smoke test](#5-smoke-test).
3. For a full test pass, run the cases of [section 6](#6-test-cases) in order.
   Later cases often reuse the state left by earlier ones; every case lists
   what it needs under **Preconditions**.
4. Copy the [results template](#7-recording-results) and fill it in as you go.
5. Report every failure as described in [8. Reporting defects](#8-reporting-defects).

> [!CAUTION]
> Never use real passwords, real personal files or your own vault for testing.
> Use only the fictitious test data of [3.4](#34-test-data). If you already use
> lockwell on the test computer, make an encrypted backup and move your vault
> folder away before you start (see [3.5](#35-returning-to-a-clean-state)).

## 3. Test environment

### 3.1 Supported platforms

| Platform                        | Status in 0.1.0                      |
| ------------------------------- | ------------------------------------ |
| Windows 11 (x64)                | Supported, primary target            |
| Windows 10 (x64), version 1809+ | Supported (needs Microsoft WebView2) |
| macOS                           | Not supported yet; not tested by CI  |
| Linux                           | Not supported yet; not tested by CI  |

The continuous integration runs on `windows-latest` only.

### 3.2 Requirements

To run an installer you need only Windows and the Microsoft Edge WebView2
runtime (included in Windows 11; the NSIS installer can install it on
Windows 10).

To build from source you also need:

- [Git](https://git-scm.com/).
- [Node.js](https://nodejs.org/) with npm. The CI uses Node.js 26.
- The [Rust](https://www.rust-lang.org/tools/install) toolchain (stable),
  installed with `rustup`.
- The Windows dependencies of the
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/):
  Microsoft C++ Build Tools ("Desktop development with C++") and WebView2.

Useful tools for some cases (all built into Windows): PowerShell, Notepad,
File Explorer, Task Manager and Resource Monitor.

### 3.3 Building and installing from scratch

Run in PowerShell:

```powershell
git clone https://github.com/acevflow/lockwell.git
cd lockwell
git checkout main
npm ci
npm run tauri build
```

The first build compiles all Rust dependencies and can take 10 minutes or
more. When it finishes you have:

| Output                | Path (relative to the repository root)                              |
| --------------------- | ------------------------------------------------------------------- |
| Standalone executable | `src-tauri\target\release\lockwell.exe`                             |
| MSI installer         | `src-tauri\target\release\bundle\msi\lockwell_0.1.0_x64_en-US.msi`  |
| NSIS installer        | `src-tauri\target\release\bundle\nsis\lockwell_0.1.0_x64-setup.exe` |

Three ways to run the app:

- **Installed:** run one of the installers (cases MT-003 and MT-004), then
  start lockwell from the Start menu. Use this for a release test pass.
- **Standalone:** run `src-tauri\target\release\lockwell.exe` directly.
- **Development:** `npm run tauri dev` (case MT-005). Development builds use a
  more permissive Content Security Policy for hot reload; do not use them for
  the security cases of [6.9](#69-security).

Record the commit you test with `git rev-parse --short HEAD`.

### 3.4 Test data

All data below is fictitious. Use it exactly as written so that results are
comparable between testers.

**Passwords**

| Name         | Value                 | Use                                  |
| ------------ | --------------------- | ------------------------------------ |
| `MASTER_1`   | `Test-Master-Pass-01` | Main master password (19 characters) |
| `MASTER_2`   | `Test-Master-Pass-02` | New master password after a change   |
| `SHORT_11`   | `Short-Pass1`         | 11 characters: one below the minimum |
| `EXACT_12`   | `Twelve-Chars`        | Exactly 12 characters                |
| `UNICODE_12` | `contraseña-ñ`        | 12 characters, 14 bytes in UTF-8     |
| `UNICODE_11` | `ñññññññññññ`         | 11 characters, 22 bytes in UTF-8     |
| `SPACES_12`  | 12 spaces             | Only whitespace                      |

**Entries**

| Field    | Entry A                            | Entry B                | Entry C      |
| -------- | ---------------------------------- | ---------------------- | ------------ |
| Title    | `Example Mail`                     | `bank of testing`      | `Zeta Forum` |
| Username | `tester@example.com`               | `test-user-42`         | (empty)      |
| Password | `Canary-Pass-4821!`                | `Another-Fake-Pass-77` | (empty)      |
| Website  | `https://mail.example.com`         | `https://bank.example` | (empty)      |
| Notes    | `Line one` + new line + `Line two` | (empty)                | (empty)      |

`Canary-Pass-4821` is a _canary_: a unique string used to prove that secrets
never appear in clear text on disk.

**Files.** Create them once with this PowerShell script. It writes a folder
`lockwell-test-data` in your user profile. Paste it into a PowerShell window;
if you save it as a `.ps1` file instead, save it as UTF-8 with BOM so that
Windows PowerShell 5.1 reads the non-ASCII file name correctly:

```powershell
$td = "$env:USERPROFILE\lockwell-test-data"
New-Item -ItemType Directory -Force $td | Out-Null
New-Item -ItemType File -Force "$td\empty.txt" | Out-Null
[IO.File]::WriteAllBytes("$td\one-byte.bin", [byte[]](65))
$rng = [Random]::new(1)
foreach ($spec in @(@('chunk-exact.bin', 65536), @('chunk-plus-one.bin', 65537), @('large-20MB.bin', 20000000))) {
  $bytes = New-Object byte[] $spec[1]
  $rng.NextBytes($bytes)
  [IO.File]::WriteAllBytes("$td\$($spec[0])", $bytes)
}
[IO.File]::WriteAllText("$td\canary-document.txt", "LOCKWELL-FILE-CANARY-93b1")
[IO.File]::WriteAllText("$td\informe-año-ü-文件.txt", "Unicode file name test")
[IO.File]::WriteAllText("$td\Alpha.txt", "alpha")
[IO.File]::WriteAllText("$td\beta.txt", "beta")
[IO.File]::WriteAllText("$td\not-a-backup.lwbackup", "this is not a backup")
Get-ChildItem $td | Select-Object Name, Length
```

Expected output: ten files, with lengths `empty.txt` 0, `one-byte.bin` 1,
`chunk-exact.bin` 65536, `chunk-plus-one.bin` 65537, `large-20MB.bin`
20000000, `canary-document.txt` 25, `Alpha.txt` 5, `beta.txt` 4,
`not-a-backup.lwbackup` 20 and `informe-año-ü-文件.txt` 22.

To compare a file before and after it goes through the vault, compute the
SHA-256 hash of both with `Get-FileHash`, as in MT-060; the two hashes must be
identical.

### 3.5 Returning to a clean state

lockwell keeps all its data in two folders:

| Folder                                  | Content                                                                       |
| --------------------------------------- | ----------------------------------------------------------------------------- |
| `%APPDATA%\com.acevflow.lockwell\vault` | The vault: `vault.db` (encrypted records) and `files\*.lwf` (encrypted files) |
| `%LOCALAPPDATA%\com.acevflow.lockwell`  | WebView2 browser data of the window (no vault data)                           |

To reset to a never-used state:

1. Close lockwell (check in Task Manager that no `lockwell.exe` is running).
2. Run in PowerShell:

   ```powershell
   Remove-Item -Recurse -Force "$env:APPDATA\com.acevflow.lockwell", "$env:LOCALAPPDATA\com.acevflow.lockwell" -ErrorAction SilentlyContinue
   ```

3. Start lockwell. The **Create your vault** screen appears.

> [!WARNING]
> This deletes the vault permanently. There is no recovery.

To keep a vault aside instead of deleting it, rename the `vault` folder (for
example to `vault-saved`) while lockwell is closed, and rename it back later.

## 4. Conventions

- Every case has a unique identifier `MT-NNN` (MT-001, MT-002, …). Identifiers
  never change meaning; new cases get new numbers.
- Each case has:
  - **Area**: the feature or quality it belongs to.
  - **Preconditions**: the state required before step 1.
  - **Steps**: numbered actions. Text in **bold** is a button, tab, field or
    window title as shown on screen. Text in `code` is typed exactly.
  - **Expected result**: what you must observe for the case to pass. Messages
    are quoted exactly; check them character by character.
  - **Result**: tick **Pass** or **Fail**. If anything differs from the expected
    result, the case fails.
- `MASTER_1`, `Entry A`, etc. refer to the [test data](#34-test-data).
- "The vault screen" is the screen with the **Passwords**, **Files** and
  **Settings** tabs and the **Lock** button.
- "Paste into Notepad" means: open Notepad, press Ctrl+V and look at the text.
- Unless a case says otherwise, run the app as an installed or standalone
  release build.

## 5. Smoke test

A 10-minute check of the essentials. Run it on every new build before the full
pass. Start from a clean state ([3.5](#35-returning-to-a-clean-state)).

| #   | Check                                                               | Case   |
| --- | ------------------------------------------------------------------- | ------ |
| S1  | The app starts and shows **Create your vault**                      | MT-006 |
| S2  | A vault is created with `MASTER_1`                                  | MT-011 |
| S3  | Entry A is created and shown with its password masked               | MT-031 |
| S4  | The password of Entry A is copied and pastes correctly              | MT-045 |
| S5  | Search finds Entry A by its username                                | MT-039 |
| S6  | `canary-document.txt` is added in **Files**                         | MT-056 |
| S7  | A decrypted copy of it is saved with the same SHA-256 hash          | MT-060 |
| S8  | **Lock** returns to the unlock screen                               | MT-027 |
| S9  | A wrong password is refused and `MASTER_1` unlocks                  | MT-024 |
| S10 | A backup is created                                                 | MT-078 |
| S11 | After closing and reopening the app, Entry A and the file are there | MT-092 |
| S12 | The canary does not appear in clear text in the vault folder        | MT-110 |

The smoke test passes only if all twelve checks pass.

## 6. Test cases

### 6.1 Installation and first start

#### MT-001 — Build the app from source

- **Area:** Installation
- **Preconditions:** Requirements of [3.2](#32-requirements) installed. Fresh
  clone of the repository.
- **Steps:**
  1. In the repository root, run `npm ci`.
  2. Run `npm run tauri build`.
- **Expected result:** Both commands end without errors. The three outputs
  listed in [3.3](#33-building-and-installing-from-scratch) exist.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-002 — Run the automated checks

- **Area:** Installation
- **Preconditions:** MT-001 passed. For coverage, the Rust component
  `llvm-tools-preview` is added (`rustup component add llvm-tools-preview`)
  and `cargo-llvm-cov` is installed (`cargo install cargo-llvm-cov --locked`).
- **Steps:**
  1. Run `npm run format:check`.
  2. Run `npm run typecheck`.
  3. Run `npm run lint`.
  4. Run `npm run test:coverage`.
- **Expected result:** Every command exits without errors. Step 4 reports all
  Vitest and Rust tests as passed and writes the reports to
  `coverage\frontend` and `coverage\core`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-003 — Install with the NSIS installer

- **Area:** Installation
- **Preconditions:** lockwell is not installed. Clean state
  ([3.5](#35-returning-to-a-clean-state)).
- **Steps:**
  1. Run `lockwell_0.1.0_x64-setup.exe`.
  2. Accept the defaults and finish the installer.
  3. Open the Start menu and search for `lockwell`.
- **Expected result:** The installer finishes without errors. A **lockwell**
  shortcut appears in the Start menu and **lockwell** is listed in
  **Settings > Apps > Installed apps** with version 0.1.0.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-004 — Install with the MSI installer

- **Area:** Installation
- **Preconditions:** lockwell is not installed (uninstall the NSIS version
  first with MT-127). Clean state.
- **Steps:**
  1. Run `lockwell_0.1.0_x64_en-US.msi`.
  2. Accept the defaults, approve the administrator prompt and finish.
  3. Open the Start menu and search for `lockwell`.
- **Expected result:** As in MT-003: a Start menu shortcut and an entry in
  **Installed apps** with version 0.1.0.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-005 — Start in development mode

- **Area:** Installation
- **Preconditions:** MT-001 step 1 done. Port 1420 free.
- **Steps:**
  1. Run `npm run tauri dev`.
  2. Wait until the window opens.
  3. Close the window.
- **Expected result:** A window titled **lockwell** opens and shows the
  create or unlock screen. The terminal shows no errors. Closing the window
  ends the command.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-006 — First start without a vault

- **Area:** Installation
- **Preconditions:** lockwell installed. Clean state.
- **Steps:**
  1. Start lockwell.
  2. Look at the window.
  3. In File Explorer, open `%APPDATA%`.
- **Expected result:** A window titled **lockwell**, about 1000 × 680 pixels,
  shows the heading **Create your vault**, two password fields (**Master
  password**, **Confirm master password**), the hint "At least 12 characters.
  A long phrase is easier to remember and harder to guess.", a warning that a
  forgotten master password cannot be recovered, an unticked checkbox, a
  **Create vault** button and a **Restore from a backup instead** link. The
  folder `%APPDATA%\com.acevflow.lockwell\vault` does not exist yet.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-007 — Start with an existing vault

- **Area:** Installation
- **Preconditions:** A vault exists (MT-011 passed). lockwell closed.
- **Steps:**
  1. Start lockwell.
- **Expected result:** The unlock screen appears: heading **lockwell**, text
  "Your vault is locked.", a **Master password** field with the focus in it
  and an **Unlock** button. No lock notice is shown.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-008 — Window minimum size

- **Area:** Installation
- **Preconditions:** lockwell running.
- **Steps:**
  1. Drag a corner of the window to make it as small as possible.
  2. Maximize the window, then restore it.
- **Expected result:** The window cannot be made smaller than about
  720 × 480 pixels. At every size the content stays usable: no control is cut
  off without a scroll bar. Maximize and restore work.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-009 — Two windows on the same vault

- **Area:** Installation
- **Preconditions:** A vault exists with no entries. lockwell closed.
- **Steps:**
  1. Start lockwell twice (two windows).
  2. In the first window, unlock with `MASTER_1`.
  3. In the second window, unlock with `MASTER_1`.
  4. In the first window, create Entry B.
  5. In the second window, open the **Files** tab and then the **Passwords**
     tab again.
  6. Close both windows, start lockwell once and unlock.
- **Expected result:** Both windows unlock. After step 5 the second window
  lists Entry B. No window shows an error. After step 6, Entry B is there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-010 — Start-up does not need the network

- **Area:** Installation
- **Preconditions:** lockwell installed. A vault exists.
- **Steps:**
  1. Turn on airplane mode or disconnect every network adapter.
  2. Start lockwell and unlock with `MASTER_1`.
  3. Open each tab.
- **Expected result:** The app starts, unlocks and shows every tab exactly as
  with a network connection.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.2 Creating the vault

All cases in this section start on the **Create your vault** screen (clean
state, MT-006). After a case that creates a vault, return to a clean state
before the next one.

#### MT-011 — Create a vault

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `MASTER_1` in **Master password** and in **Confirm master password**.
  2. Tick "I understand that a forgotten master password cannot be recovered."
  3. Click **Create vault**.
  4. Open `%APPDATA%\com.acevflow.lockwell\vault` in File Explorer.
- **Expected result:** The button shows "Creating vault…" for about a second,
  then the vault screen opens on the **Passwords** tab with "No entries yet.
  Add your first one." in the list and "Select an entry or add a new one." on
  the right. The vault folder contains `vault.db` and an empty `files`
  folder, and no file ending in `.creating`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-012 — Password one character too short

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `SHORT_11` in both password fields and tick the checkbox.
  2. Click **Create vault**.
- **Expected result:** The message "Use at least 12 characters." appears. No
  vault is created (the `vault` folder does not exist).
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-013 — Password of exactly 12 characters

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `EXACT_12` in both fields and tick the checkbox.
  2. Click **Create vault**.
- **Expected result:** The vault is created and the vault screen opens.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-014 — Length is counted in characters, not bytes

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `UNICODE_11` in both fields, tick the checkbox and click **Create
     vault**.
  2. Replace both fields with `UNICODE_12` and click **Create vault**.
  3. Click **Lock**, then unlock with `UNICODE_12`.
- **Expected result:** Step 1 shows "Use at least 12 characters." (11
  characters are refused although they are 22 bytes). Step 2 creates the
  vault. Step 3 unlocks it.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-015 — Password of only spaces

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `SPACES_12` (12 spaces) in both fields and tick the checkbox.
  2. Click **Create vault**.
- **Expected result:** The message "The password cannot be only spaces."
  appears. No vault is created.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-016 — Password longer than 1024 characters

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. In PowerShell, run `Set-Clipboard -Value ('a' * 1025)`.
  2. Paste into both password fields and tick the checkbox.
  3. Click **Create vault**.
  4. In PowerShell, run `Set-Clipboard -Value ('a' * 1024)`, paste it into
     both fields (replacing their content) and click **Create vault**.
- **Expected result:** Step 3 shows "Use at most 1024 characters." and no
  vault is created. Step 4 creates the vault.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-017 — Passwords that do not match

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `MASTER_1` in **Master password** and `MASTER_2` in **Confirm
     master password**. Tick the checkbox.
  2. Click **Create vault**.
- **Expected result:** The message "The passwords do not match." appears. No
  vault is created.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-018 — No-recovery acknowledgement is required

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `MASTER_1` in both fields. Leave the checkbox unticked.
  2. Click **Create vault**.
- **Expected result:** The message "Confirm that you understand the password
  cannot be recovered." appears. No vault is created.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-019 — Order of the validation messages

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Leave both fields empty and the checkbox unticked. Click **Create
     vault**.
  2. Type `MASTER_1` in the first field only. Click **Create vault**.
- **Expected result:** Step 1 shows "Use at least 12 characters." Step 2
  shows "The passwords do not match." Only one message is shown at a time.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-020 — Show and hide the typed password

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Type `MASTER_1` in **Master password**.
  2. Click the **Show** button next to that field.
  3. Click the same button again (now labelled **Hide**).
- **Expected result:** The password is masked by default, appears as
  `Test-Master-Pass-01` after step 2 and is masked again after step 3. The
  other field is not affected.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-021 — Create with the keyboard only

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Without clicking, type `MASTER_1` (the focus starts in **Master
     password**).
  2. Press Tab until the focus reaches **Confirm master password** and type
     `MASTER_1`.
  3. Press Tab until the focus reaches the checkbox and press Space.
  4. Press Enter.
- **Expected result:** The vault is created and the vault screen opens.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-022 — Leading and trailing spaces are part of the password

- **Area:** Vault creation
- **Preconditions:** **Create your vault** screen.
- **Steps:**
  1. Create a vault with `MASTER_1` followed by one space, in both fields.
  2. Click **Lock**.
  3. Try to unlock with `MASTER_1` (without the space).
  4. Unlock with `MASTER_1` followed by one space.
- **Expected result:** Step 3 shows "Incorrect master password." Step 4
  unlocks. The master password is used exactly as typed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.3 Unlocking and locking

Unless stated otherwise, these cases need a vault created with `MASTER_1`
(MT-011).

#### MT-023 — Unlock with the correct password

- **Area:** Unlock
- **Preconditions:** Unlock screen.
- **Steps:**
  1. Type `MASTER_1` and click **Unlock**.
- **Expected result:** The button shows "Unlocking…" while it works, then the
  vault screen opens on the **Passwords** tab with no entry selected.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-024 — Wrong master password

- **Area:** Unlock
- **Preconditions:** Unlock screen.
- **Steps:**
  1. Type `MASTER_2` and click **Unlock**.
  2. Type `MASTER_1` and click **Unlock**.
- **Expected result:** Step 1 shows "Incorrect master password.", empties the
  field and stays on the unlock screen. Step 2 unlocks.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-025 — Empty password

- **Area:** Unlock
- **Preconditions:** Unlock screen.
- **Steps:**
  1. Leave the field empty and click **Unlock**.
- **Expected result:** The message "Enter your master password." appears and
  the vault stays locked.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-026 — The password is case-sensitive

- **Area:** Unlock
- **Preconditions:** Unlock screen.
- **Steps:**
  1. Type `test-master-pass-01` (all lower case) and click **Unlock**.
- **Expected result:** "Incorrect master password." The vault stays locked.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-027 — Lock with the Lock button

- **Area:** Lock
- **Preconditions:** Vault unlocked, Entry A exists and is open on the right.
- **Steps:**
  1. Click **Lock** at the top right.
  2. Unlock with `MASTER_1`.
- **Expected result:** Step 1 shows the unlock screen with no notice; no
  entry data is visible anywhere. Step 2 opens the **Passwords** tab with no
  entry selected ("Select an entry or add a new one.").
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-028 — Repeated wrong attempts

- **Area:** Unlock
- **Preconditions:** Unlock screen.
- **Steps:**
  1. Enter a wrong password and click **Unlock**, five times in a row.
  2. Unlock with `MASTER_1`.
- **Expected result:** Each attempt shows "Incorrect master password." and
  the app stays responsive. Step 2 unlocks. (Version 0.1.0 has no attempt
  limit; each attempt costs one Argon2id key derivation.)
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-029 — Closing the window locks the vault

- **Area:** Lock
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Close the window with its **X** button.
  2. Check in Task Manager that `lockwell.exe` is no longer running.
  3. Start lockwell again.
- **Expected result:** The process ends when the window closes. On restart
  the unlock screen appears; the vault is not open.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-030 — Unlock with the keyboard only

- **Area:** Unlock
- **Preconditions:** Unlock screen.
- **Steps:**
  1. Without clicking, type `MASTER_1` and press Enter.
- **Expected result:** The vault unlocks.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.4 Password entries

Unless stated otherwise, these cases need an unlocked vault on the
**Passwords** tab.

#### MT-031 — Create an entry with every field

- **Area:** Entries
- **Preconditions:** No entries.
- **Steps:**
  1. Click **New entry**.
  2. Fill in **Title**, **Username**, **Password**, **Website** and **Notes**
     with the values of Entry A (press Enter between the two lines of the
     notes).
  3. Click **Add entry**.
- **Expected result:** The form shows the heading **New entry** with the focus
  in **Title**. After step 3 the entry appears in the list as **Example Mail**
  with `tester@example.com` below it, highlighted as selected. The right side
  shows the title, the username with a **Copy** button, the password as
  `••••••••••••` with **Show** and **Copy** buttons, the website, the notes on
  two lines and a **Last changed** date and time of now.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-032 — Title is required

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Click **New entry**, fill in only **Username** with `nobody` and click
     **Add entry**.
- **Expected result:** The message "The title is required." appears in the
  form. Nothing is added to the list.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-033 — A title of only spaces is refused

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Click **New entry**, type three spaces in **Title** and click **Add
     entry**.
- **Expected result:** "The title is required." Nothing is added.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-034 — Entry with only a title

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Create Entry C (title `Zeta Forum`, every other field empty).
- **Expected result:** The entry is saved. In the list it shows only its
  title. On the right, **Username**, **Password**, **Website** and **Notes**
  show "None", and there are no **Show** or **Copy** buttons.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-035 — Spaces are trimmed except in the password and notes

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Click **New entry**. Type `  Padded Title  ` in **Title**,
     `  padded-user  ` in **Username**, `  https://padded.example  ` in
     **Website** and `  padded-pass  ` in **Password** (two spaces before and
     after each value). Click **Add entry**.
  2. Click **Edit** and look at the fields. Click **Cancel**.
  3. Click the **Copy** button of the password and paste into Notepad between
     two `|` characters.
- **Expected result:** The entry is listed as **Padded Title**. In step 2,
  **Title**, **Username** and **Website** contain the values without the
  surrounding spaces. In step 3 Notepad shows `|  padded-pass  |`: the
  password keeps its spaces.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-036 — Multi-line notes are kept

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail**.
  2. Click **Edit** and look at **Notes**. Click **Cancel**.
- **Expected result:** The notes show `Line one` and `Line two` on separate
  lines, both in the view and in the edit form.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-037 — The list is sorted by title, ignoring case

- **Area:** Entries
- **Preconditions:** Entries A, B and C exist.
- **Steps:**
  1. Look at the order of the list.
- **Expected result:** The order is **bank of testing**, **Example Mail**,
  **Zeta Forum**.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-038 — Duplicate titles are allowed

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Create a new entry with title `Example Mail` and username `second-user`.
  2. Select each of the two **Example Mail** entries.
- **Expected result:** Both entries are listed. Each one opens with its own
  username. Delete the second one afterwards (MT-049).
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-039 — Search by username

- **Area:** Entries
- **Preconditions:** Entries A, B and C exist.
- **Steps:**
  1. Type `tester@` in the **Search** box.
- **Expected result:** Only **Example Mail** is listed. The list updates while
  typing, without pressing Enter.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-040 — Search by title or website, ignoring case

- **Area:** Entries
- **Preconditions:** Entries A, B and C exist.
- **Steps:**
  1. Type `BANK` in **Search**.
  2. Replace it with `example`.
  3. Replace it with `  zeta  ` (with spaces around it).
- **Expected result:** Step 1 lists only **bank of testing**. Step 2 lists
  **bank of testing** and **Example Mail** (both websites contain
  `example`). Step 3 lists only **Zeta Forum**.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-041 — Search never matches passwords or notes

- **Area:** Entries
- **Preconditions:** Entries A, B and C exist.
- **Steps:**
  1. Type `Canary` in **Search**.
  2. Replace it with `Line two`.
- **Expected result:** In both steps the list is empty and shows "No entries
  match your search."
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-042 — Clearing the search shows every entry

- **Area:** Entries
- **Preconditions:** A search is typed that hides some entries.
- **Steps:**
  1. Click in the **Search** box, press Ctrl+A and then Delete.
- **Expected result:** All entries are listed again, in title order.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-043 — Show and hide a stored password

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail**.
  2. Click **Show** next to the password.
  3. Click **Hide**.
  4. Click **Show**, then select **bank of testing**.
- **Expected result:** Step 1 shows `••••••••••••`. Step 2 shows
  `Canary-Pass-4821!`. Step 3 masks it again. In step 4 the password of
  **bank of testing** is masked: revealing is never carried over to another
  entry.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-044 — Edit an entry

- **Area:** Entries
- **Preconditions:** Entry B exists. Note its **Last changed** time.
- **Steps:**
  1. Wait one minute. Select **bank of testing** and click **Edit**.
  2. Change **Title** to `Bank of Testing (edited)` and **Password** to
     `Edited-Fake-Pass-78`.
  3. Click **Save changes**.
  4. Click **Show**.
- **Expected result:** The form heading is **Edit entry** and its fields hold
  the current values. After saving, the list and the view show the new title,
  the list stays sorted, step 4 shows `Edited-Fake-Pass-78`, and **Last
  changed** shows the time of the save.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-045 — Copy a password

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail** and click the **Copy** button of the password.
  2. Within 30 seconds, paste into Notepad.
- **Expected result:** The message "Password copied. It will be cleared from
  the clipboard in 30 seconds." appears. Notepad shows `Canary-Pass-4821!`.
  The password stays masked in the window.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-046 — Copy a username

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail** and click the **Copy** button of the username.
  2. Within 30 seconds, paste into Notepad.
- **Expected result:** The message "Username copied. It will be cleared from
  the clipboard in 30 seconds." appears. Notepad shows `tester@example.com`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-047 — Cancel an edit

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail** and click **Edit**.
  2. Change **Title** to `Should not be saved`.
  3. Click **Cancel**.
- **Expected result:** The view of **Example Mail** returns with its original
  values. The list still shows **Example Mail**.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-048 — Cancel a new entry

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Click **New entry**, type `Discarded` in **Title** and click **Cancel**.
- **Expected result:** The right side shows "Select an entry or add a new
  one." and no **Discarded** entry is listed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-049 — Delete an entry

- **Area:** Entries
- **Preconditions:** The duplicate **Example Mail** entry of MT-038 (username
  `second-user`) exists.
- **Steps:**
  1. Select the **Example Mail** entry with username `second-user` and click
     **Delete**.
  2. In the dialog, click **Delete**.
- **Expected result:** A dialog **Delete entry?** says "“Example Mail” will be
  deleted permanently." and the focus is on **Cancel**. After step 2 only one
  **Example Mail** entry (username `tester@example.com`) is listed and the
  right side shows "Select an entry or add a new one."
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-050 — Cancel a deletion

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail**, click **Delete**, then click **Cancel**.
  2. Click **Delete** again and press Escape.
  3. Click **Delete** again and press Enter.
- **Expected result:** After each step the dialog closes and **Example Mail**
  is still listed and shown. Enter does not confirm because the focus starts
  on **Cancel**.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-051 — Field length limits

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. In PowerShell, run `Set-Clipboard -Value ('t' * 201)`.
  2. Click **New entry** and paste into **Title**.
  3. Run `Set-Clipboard -Value ('u' * 501)` and paste into **Username**.
  4. Run `Set-Clipboard -Value ('w' * 2049)` and paste into **Website**.
  5. Run `Set-Clipboard -Value ('n' * 20001)` and paste into **Notes**.
  6. Click **Add entry**, then **Edit**, and check the length of each field
     (copy each one with Ctrl+A, Ctrl+C and run `(Get-Clipboard -Raw).Length`
     in PowerShell).
- **Expected result:** The fields accept at most 200, 500, 2048 and 20000
  characters: the extra character is not inserted. The entry saves, and the
  lengths are 200, 500, 2048 and 20000.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-052 — Password length limit

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. In PowerShell, run `Set-Clipboard -Value ('p' * 4097)`.
  2. Click **New entry**, type `Long password` in **Title**, paste into
     **Password** and click **Add entry**.
  3. Run `Set-Clipboard -Value ('p' * 4096)`, replace the password with it
     and click **Add entry**.
- **Expected result:** Step 2 shows "The password must have at most 4096
  characters." inside the form and saves nothing; the form stays usable.
  Step 3 saves the entry.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-053 — Unicode and emoji in every field

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Create an entry with title `Café 東京 🔐`, username `usuário`, password
     `пароль-密码-🔑` and notes `Ñandú ✓`.
  2. Lock, unlock and open the entry. Click **Show**.
- **Expected result:** Every value is shown exactly as typed, before and
  after locking.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-054 — Markup in fields is shown as plain text

- **Area:** Entries
- **Preconditions:** None.
- **Steps:**
  1. Create an entry with title `<b>bold</b>` and notes
     `<img src=x onerror=alert(1)>`.
  2. Select it.
- **Expected result:** The title and notes are shown literally, with the angle
  brackets. No text is bold, no image placeholder appears and no alert or
  dialog opens.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-055 — The website is not a link

- **Area:** Entries
- **Preconditions:** Entry A exists.
- **Steps:**
  1. Select **Example Mail** and click the website text.
- **Expected result:** The website is plain text. Clicking it opens nothing:
  no browser starts and the window does not navigate.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.5 Encrypted files

Unless stated otherwise, these cases need an unlocked vault on the **Files**
tab and the test files of [3.4](#34-test-data) in
`%USERPROFILE%\lockwell-test-data`. Save decrypted copies to a separate
folder, `%USERPROFILE%\lockwell-test-out` (create it first).

#### MT-056 — Add a file

- **Area:** Files
- **Preconditions:** No files in the vault.
- **Steps:**
  1. Check that the tab shows "No files yet. Add your first one."
  2. Click **Add files**.
  3. In the dialog, select `canary-document.txt` and click **Open**.
  4. Open `%APPDATA%\com.acevflow.lockwell\vault\files` in File Explorer.
- **Expected result:** The dialog is titled **Add files to the vault**. After
  step 3 the message "Added “canary-document.txt”." appears and the table
  lists **canary-document.txt** with size "25 bytes" and today's date. The
  `files` folder contains exactly one file, named with 32 lowercase
  hexadecimal characters and the extension `.lwf`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-057 — Add several files at once; sort order

- **Area:** Files
- **Preconditions:** MT-056 passed.
- **Steps:**
  1. Click **Add files**, select `Alpha.txt`, `beta.txt` and `one-byte.bin`
     (Ctrl+click) and click **Open**.
- **Expected result:** The message "Added 3 files." appears. The table lists,
  in this order: **Alpha.txt**, **beta.txt**, **canary-document.txt**,
  **one-byte.bin** (by name, ignoring case).
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-058 — Cancel adding files

- **Area:** Files
- **Preconditions:** None.
- **Steps:**
  1. Click **Add files** and close the dialog with **Cancel**.
- **Expected result:** No message appears and the table is unchanged.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-059 — Size display

- **Area:** Files
- **Preconditions:** None.
- **Steps:**
  1. Add `empty.txt`, `chunk-exact.bin`, `chunk-plus-one.bin` and
     `large-20MB.bin`.
  2. Read the **Size** column.
- **Expected result:** `empty.txt` "0 bytes", `one-byte.bin` (from MT-057)
  "1 byte", `chunk-exact.bin` "65.5 KB", `chunk-plus-one.bin` "65.5 KB",
  `large-20MB.bin` "20 MB". Sizes use decimal units (1 KB = 1000 bytes).
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-060 — Save a decrypted copy

- **Area:** Files
- **Preconditions:** `canary-document.txt` is in the vault.
- **Steps:**
  1. Click **Save a copy** in the row of **canary-document.txt**.
  2. In the dialog, go to `%USERPROFILE%\lockwell-test-out` and click
     **Save**.
  3. In PowerShell, run:

     ```powershell
     Get-FileHash -Algorithm SHA256 "$env:USERPROFILE\lockwell-test-data\canary-document.txt", "$env:USERPROFILE\lockwell-test-out\canary-document.txt"
     ```

- **Expected result:** The dialog is titled **Save a decrypted copy** and the
  file name is already filled in as `canary-document.txt`. After saving, the
  message "Saved a copy of “canary-document.txt”." appears. Both hashes are
  identical.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-061 — Content survives at every size boundary

- **Area:** Files
- **Preconditions:** MT-057 and MT-059 passed.
- **Steps:**
  1. Save a copy of `empty.txt`, `one-byte.bin`, `chunk-exact.bin`,
     `chunk-plus-one.bin` and `large-20MB.bin` into
     `%USERPROFILE%\lockwell-test-out`.
  2. In PowerShell, run:

     ```powershell
     foreach ($name in 'empty.txt', 'one-byte.bin', 'chunk-exact.bin', 'chunk-plus-one.bin', 'large-20MB.bin') {
       $a = (Get-FileHash "$env:USERPROFILE\lockwell-test-data\$name").Hash
       $b = (Get-FileHash "$env:USERPROFILE\lockwell-test-out\$name").Hash
       "{0}: {1}" -f $name, ($(if ($a -eq $b) { 'same' } else { 'DIFFERENT' }))
     }
     ```

- **Expected result:** Every line ends with `same`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-062 — Progress and responsiveness with a large file

- **Area:** Files
- **Preconditions:** `large-20MB.bin` is not in the vault.
- **Steps:**
  1. Click **Add files** and select `large-20MB.bin`.
  2. While it is being added, look at the tab and try to move the window.
- **Expected result:** While it works, the tab shows "Encrypting files…" and
  the **Add files**, **Save a copy** and **Delete** buttons are disabled. The
  window can be moved and does not show "Not responding". At the end, "Added
  “large-20MB.bin”." appears and the buttons are enabled again.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-063 — Unicode file names

- **Area:** Files
- **Preconditions:** None.
- **Steps:**
  1. Add `informe-año-ü-文件.txt`.
  2. Click **Save a copy** in its row.
- **Expected result:** The table shows the name exactly as
  `informe-año-ü-文件.txt`, and the save dialog proposes that same name.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-064 — Cancel saving a copy

- **Area:** Files
- **Preconditions:** At least one file in the vault.
- **Steps:**
  1. Click **Save a copy** and close the dialog with **Cancel**.
- **Expected result:** No message appears and no file is written.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-065 — Replace an existing file when saving a copy

- **Area:** Files
- **Preconditions:** MT-060 passed (the copy already exists in
  `lockwell-test-out`).
- **Steps:**
  1. In PowerShell, run
     `Set-Content "$env:USERPROFILE\lockwell-test-out\canary-document.txt" "old"`.
  2. Click **Save a copy** for **canary-document.txt**, choose the same
     folder and click **Save**.
  3. Confirm the replacement in the Windows dialog.
  4. Compare the hashes as in MT-060.
- **Expected result:** Windows asks whether to replace the existing file.
  After confirming, "Saved a copy of “canary-document.txt”." appears and the
  hashes are identical again.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-066 — A copy cannot be saved inside the vault folder

- **Area:** Files
- **Preconditions:** At least one file in the vault.
- **Steps:**
  1. Click **Save a copy**, go to `%APPDATA%\com.acevflow.lockwell\vault` and
     click **Save**.
  2. Click **Dismiss** on the error.
- **Expected result:** The error "Files inside the vault folder cannot be
  used." appears and nothing is written in the vault folder. **Dismiss**
  removes the error.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-067 — Vault files cannot be added to the vault

- **Area:** Files
- **Preconditions:** At least one file in the vault.
- **Steps:**
  1. Click **Add files**, go to `%APPDATA%\com.acevflow.lockwell\vault\files`
     and select a `.lwf` file.
- **Expected result:** An error appears that starts with "Could not add “",
  followed by the `.lwf` file name, and ends with "”: files inside the vault
  folder cannot be used." No file is added.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-068 — Delete a file

- **Area:** Files
- **Preconditions:** `Alpha.txt` is in the vault. Count the files in
  `%APPDATA%\com.acevflow.lockwell\vault\files`.
- **Steps:**
  1. Click **Delete** in the row of **Alpha.txt**.
  2. In the dialog, click **Delete**.
  3. Count the files in the `files` folder again.
- **Expected result:** A dialog **Delete file?** says "“Alpha.txt” will be
  deleted from the vault permanently." After step 2, "Deleted “Alpha.txt”."
  appears and the row is gone. The `files` folder has one file fewer.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-069 — Cancel deleting a file

- **Area:** Files
- **Preconditions:** At least one file in the vault.
- **Steps:**
  1. Click **Delete** in a row, then **Cancel**.
  2. Click **Delete** again and press Escape.
- **Expected result:** The dialog closes each time and the file is still
  listed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-070 — The same file added twice

- **Area:** Files
- **Preconditions:** `beta.txt` is in the vault.
- **Steps:**
  1. Add `beta.txt` again.
- **Expected result:** "Added “beta.txt”." The table lists **beta.txt**
  twice; each row can be saved and deleted on its own.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-071 — Adding a file leaves the original untouched

- **Area:** Files
- **Preconditions:** None.
- **Steps:**
  1. Run `Get-FileHash "$env:USERPROFILE\lockwell-test-data\one-byte.bin"`.
  2. Add `one-byte.bin` to the vault.
  3. Run the command of step 1 again.
- **Expected result:** The original file still exists and both hashes are
  identical. lockwell does not move or delete the original; deleting it
  securely is up to the user.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.6 Changing the master password

These cases use the **Change master password** form on the **Settings** tab,
in a vault created with `MASTER_1` that holds Entry A and
`canary-document.txt`.

#### MT-072 — Change the master password

- **Area:** Master password change
- **Preconditions:** Vault unlocked with `MASTER_1`.
- **Steps:**
  1. Type `MASTER_1` in **Current master password** and `MASTER_2` in **New
     master password** and **Confirm new master password**.
  2. Click **Change password**.
  3. Click **Lock** and try to unlock with `MASTER_1`.
  4. Unlock with `MASTER_2`.
  5. Open Entry A and save a copy of `canary-document.txt` (compare hashes as
     in MT-060).
- **Expected result:** The button shows "Changing…", then "Master password
  changed." appears and all three fields are emptied. Step 3 shows "Incorrect
  master password." Step 4 unlocks. In step 5, Entry A is intact and the
  hashes are identical.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-073 — Current password is required

- **Area:** Master password change
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Leave **Current master password** empty, type `MASTER_2` in both new
     fields and click **Change password**.
- **Expected result:** "Enter your current master password." The password is
  not changed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-074 — Wrong current password

- **Area:** Master password change
- **Preconditions:** Vault unlocked with `MASTER_1`.
- **Steps:**
  1. Type `Wrong-Current-Pass` as the current password and `MASTER_2` in both
     new fields. Click **Change password**.
  2. Lock and unlock with `MASTER_1`.
- **Expected result:** "The current master password is incorrect." The vault
  stays unlocked. Step 2 unlocks with `MASTER_1`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-075 — New password too short

- **Area:** Master password change
- **Preconditions:** Vault unlocked with `MASTER_1`.
- **Steps:**
  1. Type `MASTER_1` as the current password and `SHORT_11` in both new
     fields. Click **Change password**.
- **Expected result:** "New password: Use at least 12 characters." The
  password is not changed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-076 — New passwords do not match

- **Area:** Master password change
- **Preconditions:** Vault unlocked with `MASTER_1`.
- **Steps:**
  1. Type `MASTER_1` as the current password, `MASTER_2` as the new one and
     `Test-Master-Pass-03` as the confirmation. Click **Change password**.
- **Expected result:** "The new passwords do not match." The password is not
  changed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-077 — New password equal to the current one

- **Area:** Master password change
- **Preconditions:** Vault unlocked with `MASTER_1`.
- **Steps:**
  1. Type `MASTER_1` in all three fields. Click **Change password**.
- **Expected result:** "The new master password must be different from the
  current one." The password is not changed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.7 Encrypted backup and restore

These cases need a vault created with `MASTER_1` holding Entries A, B and C
and the file `canary-document.txt`. Backups are saved in
`%USERPROFILE%\lockwell-test-data`.

#### MT-078 — Create a backup

- **Area:** Backup
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Open **Settings** and click **Create backup** in **Encrypted backup**.
  2. Look at the proposed file name and the file type.
  3. Go to `%USERPROFILE%\lockwell-test-data`, change the name to
     `backup-1.lwbackup` and click **Save**.
- **Expected result:** The dialog is titled **Save an encrypted backup**,
  proposes `lockwell-backup-YYYY-MM-DD.lwbackup` with today's date in UTC (it
  can differ from the local date around midnight) and offers the file type
  **lockwell backup**. After saving, the button shows "Saving backup…" and
  then "Backup saved." appears. `backup-1.lwbackup` exists and is larger than
  0 bytes.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-079 — Cancel creating a backup

- **Area:** Backup
- **Preconditions:** Vault unlocked, **Settings** tab.
- **Steps:**
  1. Click **Create backup** and close the dialog with **Cancel**.
- **Expected result:** No message appears and no file is written.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-080 — A backup cannot be saved inside the vault folder

- **Area:** Backup
- **Preconditions:** Vault unlocked, **Settings** tab.
- **Steps:**
  1. Click **Create backup**, go to `%APPDATA%\com.acevflow.lockwell\vault`
     and click **Save**.
- **Expected result:** The error "Files inside the vault folder cannot be
  used." appears and no backup file is written there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-081 — Restore over the current vault

- **Area:** Restore
- **Preconditions:** MT-078 passed (`backup-1.lwbackup` holds Entries A, B, C
  and `canary-document.txt`).
- **Steps:**
  1. Create an entry titled `Added after backup`. Delete Entry C
     (**Zeta Forum**).
  2. In **Settings > Restore from a backup**, type `MASTER_1` in **Backup
     master password** and click **Choose backup and restore**.
  3. In the dialog **Replace this vault?**, click **Replace vault**.
  4. In the file dialog, select `backup-1.lwbackup` and click **Open**.
  5. Unlock with `MASTER_1`.
  6. Save a copy of `canary-document.txt` and compare hashes as in MT-060.
- **Expected result:** The section warns "Restoring replaces everything in
  this vault with the content of the backup." The confirmation says "All
  current passwords and files will be replaced by the backup. This cannot be
  undone." The file dialog is titled **Choose a backup to restore**. After
  step 4 the unlock screen shows "Backup restored. Unlock it with the master
  password it had when the backup was made." After step 5, Entries A, B and C
  are listed and **Added after backup** is not. The hashes are identical.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-082 — Restore needs the backup password

- **Area:** Restore
- **Preconditions:** Vault unlocked, **Settings** tab.
- **Steps:**
  1. Leave **Backup master password** empty and click **Choose backup and
     restore**.
- **Expected result:** "Enter the master password of the backup." No dialog
  opens.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-083 — Cancel the replacement

- **Area:** Restore
- **Preconditions:** Vault unlocked, **Settings** tab.
- **Steps:**
  1. Type `MASTER_1` in **Backup master password** and click **Choose backup
     and restore**.
  2. Click **Cancel** in **Replace this vault?**.
  3. Open the **Passwords** tab.
- **Expected result:** The dialog closes, no file dialog opens, the vault
  stays unlocked and every entry is still there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-084 — Cancel the file dialog of a restore

- **Area:** Restore
- **Preconditions:** Vault unlocked, **Settings** tab.
- **Steps:**
  1. Type `MASTER_1`, click **Choose backup and restore** and **Replace
     vault**.
  2. Close the file dialog with **Cancel**.
- **Expected result:** No message appears, the vault stays unlocked and
  unchanged, and **Choose backup and restore** can be clicked again.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-085 — Wrong backup password

- **Area:** Restore
- **Preconditions:** `backup-1.lwbackup` exists (made with `MASTER_1`).
- **Steps:**
  1. Type `MASTER_2` in **Backup master password**, click **Choose backup and
     restore**, **Replace vault**, and select `backup-1.lwbackup`.
  2. Open the **Passwords** tab.
- **Expected result:** "That password does not open this backup." The vault
  stays unlocked and unchanged.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-086 — A file that is not a backup

- **Area:** Restore
- **Preconditions:** Test data created.
- **Steps:**
  1. Type `MASTER_1`, click **Choose backup and restore**, **Replace vault**,
     and select `not-a-backup.lwbackup`.
- **Expected result:** "The vault data is damaged: the file is not a lockwell
  backup." The vault stays unlocked and unchanged.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-087 — A truncated backup

- **Area:** Restore
- **Preconditions:** `backup-1.lwbackup` exists.
- **Steps:**
  1. In PowerShell, create a copy cut to half its size:

     ```powershell
     $td = "$env:USERPROFILE\lockwell-test-data"
     Copy-Item "$td\backup-1.lwbackup" "$td\truncated.lwbackup"
     $fs = [IO.File]::Open("$td\truncated.lwbackup", 'Open')
     $fs.SetLength([long]($fs.Length / 2)); $fs.Close()
     ```

  2. Restore `truncated.lwbackup` with `MASTER_1` (confirm **Replace vault**).
- **Expected result:** "The vault data is damaged: the backup is damaged or
  incomplete." The vault stays unlocked and unchanged. No folder whose name
  contains `.restoring-` remains in `%APPDATA%\com.acevflow.lockwell`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-088 — A tampered backup

- **Area:** Restore
- **Preconditions:** `backup-1.lwbackup` exists.
- **Steps:**
  1. In PowerShell, create a copy with one bit flipped in the middle:

     ```powershell
     $td = "$env:USERPROFILE\lockwell-test-data"
     Copy-Item "$td\backup-1.lwbackup" "$td\tampered.lwbackup"
     $fs = [IO.File]::Open("$td\tampered.lwbackup", 'Open')
     $fs.Position = [long]($fs.Length / 2); $b = $fs.ReadByte()
     $fs.Position -= 1; $fs.WriteByte($b -bxor 1); $fs.Close()
     ```

  2. Restore `tampered.lwbackup` with `MASTER_1` (confirm **Replace vault**).
- **Expected result:** "The vault data is damaged: the backup is damaged or
  incomplete." The vault stays unlocked and unchanged.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-089 — Restore on a new computer

- **Area:** Restore
- **Preconditions:** `backup-1.lwbackup` exists. Clean state (or a second
  computer or Windows user), **Create your vault** screen.
- **Steps:**
  1. Click **Restore from a backup instead**.
  2. Click **Create a new vault instead**, then **Restore from a backup
     instead** again.
  3. Type `MASTER_1`, click **Choose backup and restore** and select
     `backup-1.lwbackup`.
  4. Unlock with `MASTER_1`.
- **Expected result:** Step 1 shows **Restore from a backup** without the
  replacement warning. Step 2 switches to the create form and back. Step 3
  asks for no confirmation and shows the unlock screen with "Backup restored.
  Unlock it with the master password it had when the backup was made." After
  step 4, Entries A, B, C and `canary-document.txt` are there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-090 — A backup keeps the password it was made with

- **Area:** Restore
- **Preconditions:** `backup-1.lwbackup` made with `MASTER_1`. Vault unlocked.
- **Steps:**
  1. Change the master password to `MASTER_2` (MT-072).
  2. Restore `backup-1.lwbackup` with `MASTER_2`.
  3. Restore `backup-1.lwbackup` with `MASTER_1`.
  4. Unlock with `MASTER_2`, then with `MASTER_1`.
- **Expected result:** Step 2 shows "That password does not open this
  backup." Step 3 restores. In step 4, `MASTER_2` is refused ("Incorrect
  master password.") and `MASTER_1` unlocks.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-091 — The backup file holds no clear text

- **Area:** Backup
- **Preconditions:** `backup-1.lwbackup` exists.
- **Steps:**
  1. In PowerShell, run:

     ```powershell
     Select-String -Path "$env:USERPROFILE\lockwell-test-data\backup-1.lwbackup" -Pattern 'Canary-Pass-4821', 'tester@example.com', 'Example Mail', 'canary-document', 'LOCKWELL-FILE-CANARY' -SimpleMatch
     ```

- **Expected result:** The command prints nothing.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.8 Persistence and integrity

These cases need a vault created with `MASTER_1` holding Entries A, B and C
and the files `canary-document.txt` and `beta.txt`.

#### MT-092 — Data survives closing and reopening

- **Area:** Persistence
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Close the window.
  2. Start lockwell and unlock with `MASTER_1`.
  3. Check the **Passwords** and **Files** tabs; save a copy of
     `canary-document.txt` and compare hashes as in MT-060.
- **Expected result:** Every entry, with all its fields, and every file are
  there. The hashes are identical.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-093 — Data survives a restart of Windows

- **Area:** Persistence
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Restart Windows without closing lockwell first.
  2. Start lockwell and unlock with `MASTER_1`.
- **Expected result:** The unlock screen appears (the vault is not open) and,
  after unlocking, all entries and files are there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-094 — Killing the process does not damage the vault

- **Area:** Integrity
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Edit Entry B and save it.
  2. In Task Manager, select **lockwell** and click **End task**.
  3. Start lockwell and unlock with `MASTER_1`.
- **Expected result:** The vault unlocks without errors and Entry B has the
  edited values.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-095 — Interrupting a file import

- **Area:** Integrity
- **Preconditions:** Vault unlocked, `large-20MB.bin` not in the vault.
- **Steps:**
  1. Add `large-20MB.bin` and, while "Encrypting files…" is shown, end the
     process in Task Manager.
  2. Start lockwell, unlock and open **Files**.
- **Expected result:** The vault unlocks without errors. Either
  `large-20MB.bin` is listed complete (its copy has the same hash as the
  original) or it is not listed at all; it is never listed with a wrong size
  or content. Every other file is intact. A hidden temporary file ending in
  `.tmp` may remain in the `files` folder; it is encrypted and is not listed.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-096 — A missing encrypted file is detected

- **Area:** Integrity
- **Preconditions:** Only `canary-document.txt` in the vault (delete the
  others). lockwell closed.
- **Steps:**
  1. Move the only `.lwf` file out of
     `%APPDATA%\com.acevflow.lockwell\vault\files` to
     `%USERPROFILE%\lockwell-test-out`.
  2. Start lockwell, unlock and open **Files**.
  3. Click **Save a copy** for **canary-document.txt** and save it to
     `lockwell-test-out`.
  4. On **Settings**, click **Create backup** and save it anywhere outside the
     vault folder.
  5. Close lockwell, move the `.lwf` file back, start lockwell and repeat
     step 3.
- **Expected result:** Step 2 lists the file. Step 3 shows "The vault data is
  damaged: the encrypted file is missing." and writes no file. Step 4 shows
  "The vault data is damaged: a stored file is missing." and writes no
  backup. After step 5 the copy is saved and its hash matches the original.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-097 — A damaged encrypted file is detected

- **Area:** Integrity
- **Preconditions:** Only `canary-document.txt` in the vault (delete the
  others). No file `canary-document.txt` in `%USERPROFILE%\lockwell-test-out`.
  lockwell closed. Make a copy of the whole `vault` folder first to restore it
  afterwards.
- **Steps:**
  1. In PowerShell, flip one bit near the end of the encrypted file:

     ```powershell
     $f = Get-ChildItem "$env:APPDATA\com.acevflow.lockwell\vault\files\*.lwf" | Select-Object -First 1
     $fs = [IO.File]::Open($f.FullName, 'Open')
     $fs.Position = $fs.Length - 5; $b = $fs.ReadByte()
     $fs.Position -= 1; $fs.WriteByte($b -bxor 1); $fs.Close()
     ```

  2. Start lockwell, unlock and save a copy of **canary-document.txt** into
     `%USERPROFILE%\lockwell-test-out`.
  3. Close lockwell and put the original `vault` folder back.
- **Expected result:** Step 2 shows "The vault data is damaged: the encrypted
  file failed to decrypt." and no `canary-document.txt` is written in
  `lockwell-test-out`.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-098 — Deleting the database file

- **Area:** Integrity
- **Preconditions:** A vault exists. lockwell closed. Make a copy of the
  `vault` folder first.
- **Steps:**
  1. Delete `%APPDATA%\com.acevflow.lockwell\vault\vault.db`.
  2. Start lockwell.
  3. Close lockwell and put the original `vault` folder back.
- **Expected result:** Step 2 shows **Create your vault**: without `vault.db`
  there is no vault. After step 3 the vault unlocks again with all its data.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-099 — The vault folder can be moved to another computer

- **Area:** Persistence
- **Preconditions:** A vault exists. A second Windows computer or user with
  lockwell installed and no vault.
- **Steps:**
  1. Close lockwell and copy the whole `%APPDATA%\com.acevflow.lockwell\vault`
     folder to the same path on the second computer or user.
  2. Start lockwell there and unlock with `MASTER_1`.
- **Expected result:** The vault unlocks and all entries and files are there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-100 — Reinstalling keeps the vault

- **Area:** Persistence
- **Preconditions:** lockwell installed with the NSIS installer. A vault
  exists. lockwell closed.
- **Steps:**
  1. Run the NSIS installer of the same or a newer build again and finish it.
  2. Start lockwell and unlock with `MASTER_1`.
- **Expected result:** The installer finishes without errors. The vault
  unlocks and all entries and files are there.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.9 Security

Run these cases with a release build (installed or standalone), not with
`npm run tauri dev`. They need a vault created with `MASTER_1` holding Entry A
and `canary-document.txt`.

#### MT-101 — Automatic lock after 5 minutes of inactivity

- **Area:** Automatic lock
- **Preconditions:** Vault unlocked, Entry A open with the password shown.
  Have a clock or stopwatch at hand.
- **Steps:**
  1. Note the time and do not touch the mouse or keyboard over the lockwell
     window (moving the mouse over the window counts as activity).
  2. Wait 6 minutes and look at the window.
- **Expected result:** Between about 4 minutes 45 seconds and 5 minutes
  10 seconds after the last activity, the window switches to the unlock
  screen with the notice "The vault was locked after 5 minutes of
  inactivity." No entry data is visible.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-102 — Activity postpones the automatic lock

- **Area:** Automatic lock
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. For 7 minutes, move the mouse over the lockwell window or press a key in
     it about once a minute.
- **Expected result:** The vault stays unlocked the whole time.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-103 — Activity in other apps does not count

- **Area:** Automatic lock
- **Preconditions:** Vault unlocked.
- **Steps:**
  1. Switch to another application (for example Notepad) and keep typing in
     it for 6 minutes, without touching the lockwell window.
  2. Switch back to lockwell.
- **Expected result:** lockwell shows the unlock screen with "The vault was
  locked after 5 minutes of inactivity."
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-104 — Lock when Windows is locked

- **Area:** System lock
- **Preconditions:** Vault unlocked, Entry A open.
- **Steps:**
  1. Press Win+L.
  2. Sign in to Windows again and look at lockwell.
- **Expected result:** lockwell shows the unlock screen with the notice "The
  vault was locked because Windows was locked."
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-105 — Lock when switching Windows user

- **Area:** System lock
- **Preconditions:** A second Windows account exists. Vault unlocked.
- **Steps:**
  1. Open the Start menu, click your account picture and choose the other
     account (switch user).
  2. Switch back to the first account.
- **Expected result:** lockwell shows the unlock screen with "The vault was
  locked because Windows was locked."
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-106 — The clipboard is cleared after 30 seconds

- **Area:** Clipboard
- **Preconditions:** Vault unlocked, Entry A open.
- **Steps:**
  1. Click **Copy** for the password.
  2. Wait 35 seconds without copying anything else.
  3. Paste into Notepad.
- **Expected result:** Nothing is pasted: the clipboard is empty.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-107 — Text copied afterwards is not cleared

- **Area:** Clipboard
- **Preconditions:** Vault unlocked, Entry A open.
- **Steps:**
  1. Click **Copy** for the password.
  2. Within 10 seconds, type `my own text` in Notepad, select it and press
     Ctrl+C.
  3. Wait 35 seconds and paste into Notepad.
- **Expected result:** `my own text` is pasted: lockwell only clears the
  clipboard if it still holds the copied secret.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-108 — Locking clears the clipboard at once

- **Area:** Clipboard
- **Preconditions:** Vault unlocked, Entry A open.
- **Steps:**
  1. Click **Copy** for the password.
  2. Immediately click **Lock**.
  3. Paste into Notepad.
- **Expected result:** Nothing is pasted.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-109 — Copied secrets stay out of the clipboard history

- **Area:** Clipboard
- **Preconditions:** Windows clipboard history turned on (**Settings > System
  > Clipboard > Clipboard history**). Vault unlocked, Entry A open.
- **Steps:**
  1. Copy `history check` from Notepad.
  2. In lockwell, click **Copy** for the password.
  3. Press Win+V.
- **Expected result:** The history lists `history check` but not
  `Canary-Pass-4821!`. (If clipboard sync across devices is on, the password
  is not synced either.)
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-110 — No clear text in the vault folder

- **Area:** Data at rest
- **Preconditions:** Entry A and `canary-document.txt` are in the vault.
  lockwell closed.
- **Steps:**
  1. In PowerShell, check that the search works on the test data (positive
     control):

     ```powershell
     Select-String -Path "$env:USERPROFILE\lockwell-test-data\canary-document.txt" -Pattern 'LOCKWELL-FILE-CANARY' -SimpleMatch
     ```

  2. Search the vault folder, in UTF-8 and in UTF-16:

     ```powershell
     $patterns = 'Canary-Pass-4821', 'tester@example.com', 'Example Mail', 'mail.example.com', 'Line two', 'canary-document', 'LOCKWELL-FILE-CANARY', 'Test-Master-Pass'
     $vaultFiles = Get-ChildItem -Recurse -File -Force "$env:APPDATA\com.acevflow.lockwell"
     $vaultFiles | Select-String -Pattern $patterns -SimpleMatch -Encoding UTF8
     $vaultFiles | Select-String -Pattern $patterns -SimpleMatch -Encoding Unicode
     ```

- **Expected result:** Step 1 prints one match. Step 2 prints nothing: no
  password, username, title, website, note, file name, file content or
  master password is stored in clear text.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-111 — No secrets in the WebView data

- **Area:** Data at rest
- **Preconditions:** All of section 6.4 run in this installation. lockwell
  closed.
- **Steps:**
  1. Run the search of MT-110 step 2 replacing the folder with
     `"$env:LOCALAPPDATA\com.acevflow.lockwell"`.
- **Expected result:** Nothing is printed: the window's browser storage,
  caches and logs hold no test secret.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-112 — File names are hidden on disk

- **Area:** Data at rest
- **Preconditions:** Several files are in the vault.
- **Steps:**
  1. Run `Get-ChildItem -Force "$env:APPDATA\com.acevflow.lockwell\vault\files" | Select-Object Name, Length`.
- **Expected result:** Every name is 32 lowercase hexadecimal characters
  followed by `.lwf`. No original name or extension (`.txt`, `.bin`) is
  visible, and each file is a little larger than the original it stores.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-113 — No network connections

- **Area:** Network
- **Preconditions:** lockwell running, vault unlocked.
- **Steps:**
  1. Use the app for a minute (open entries, copy, add a file, create a
     backup).
  2. In PowerShell, run:

     ```powershell
     $id = (Get-Process lockwell).Id
     Get-NetTCPConnection -OwningProcess $id -ErrorAction SilentlyContinue
     Get-NetUDPEndpoint -OwningProcess $id -ErrorAction SilentlyContinue
     ```

  3. Open Resource Monitor (`resmon`), **Network** tab, and look for
     `lockwell.exe` under **Network Activity** and **TCP Connections**.
- **Expected result:** Step 2 prints nothing. In step 3, `lockwell.exe` does
  not appear.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-114 — Developer tools are not available

- **Area:** Hardening
- **Preconditions:** Release build running.
- **Steps:**
  1. Click inside the window and press F12.
  2. Press Ctrl+Shift+I.
- **Expected result:** No developer tools window or panel opens.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-115 — Password fields stay private

- **Area:** Sensitive data handling
- **Preconditions:** Unlock screen; then an unlocked vault with Entry A.
- **Steps:**
  1. Type `Tset-Misspeled-Pasword` in the master password field and click
     **Show**.
  2. Unlock. Select Entry A, click **Edit** and look at **Password**.
- **Expected result:** In step 1 no spelling underline and no suggestion list
  appear. In step 2 the password is masked in the form until **Show** is
  clicked.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-116 — Error messages never show passwords

- **Area:** Sensitive data handling
- **Preconditions:** None.
- **Steps:**
  1. Run MT-024, MT-074 and MT-085 and read every message they show.
- **Expected result:** No message contains any typed password or any stored
  secret.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-117 — Permissions of the vault folder

- **Area:** Permissions
- **Preconditions:** A vault exists.
- **Steps:**
  1. Run `icacls "$env:APPDATA\com.acevflow.lockwell\vault"`. The first line
     of the output starts with the full path of the folder; write it down.
  2. Sign in as another standard (non-administrator) Windows user and try to
     open the path written down in step 1 in File Explorer.
- **Expected result:** Step 1 lists only your account, `SYSTEM` and
  `Administrators`; there is no entry for `Everyone` or `Users`. In step 2
  Windows denies access.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.10 Interface and accessibility

#### MT-118 — Light and dark themes follow Windows

- **Area:** Interface
- **Preconditions:** lockwell running.
- **Steps:**
  1. In **Settings > Personalization > Colors**, choose **Light**.
  2. Choose **Dark**.
- **Expected result:** lockwell follows the Windows mode (light background in
  step 1, dark in step 2). In both, all text, buttons, fields, messages and
  dialogs are readable.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-119 — Keyboard-only use of the vault

- **Area:** Accessibility
- **Preconditions:** Vault unlocked with Entry A.
- **Steps:**
  1. Using only Tab, Shift+Tab, Enter, Space and Escape: open the **Files**
     tab, then **Settings**, then **Passwords**.
  2. Open Entry A, show its password, start a deletion and cancel it with
     Escape.
  3. Reach the **Lock** button and press Enter.
- **Expected result:** Every control can be reached and used, the focus is
  always visible, and the vault locks at the end.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-120 — Switching tabs keeps the vault open

- **Area:** Interface
- **Preconditions:** Vault unlocked, Entry A open.
- **Steps:**
  1. Open **Files**, then **Settings**, then **Passwords**.
- **Expected result:** The selected tab is highlighted. The vault stays
  unlocked. Back on **Passwords**, the list is shown and no entry is selected.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-121 — Long values do not break the layout

- **Area:** Interface
- **Preconditions:** The entry of MT-051 (fields at their limits) exists.
- **Steps:**
  1. Select it in the list and scroll through the view.
- **Expected result:** The text wraps or scrolls inside the window; no text
  covers buttons and the window gets no horizontal scroll bar for the whole
  page.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.11 Platforms

lockwell 0.1.0 supports Windows only. macOS and Linux are planned: the code
for locking on a system lock and for keeping secrets out of the clipboard
history is Windows-specific, and the CI does not build or test other systems.
Testing on macOS or Linux is out of scope for this guide.

#### MT-122 — Windows 11

- **Area:** Platforms
- **Preconditions:** Windows 11 x64, clean state.
- **Steps:**
  1. Install with the NSIS installer and run the [smoke test](#5-smoke-test).
- **Expected result:** Every smoke check passes.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-123 — Windows 10

- **Area:** Platforms
- **Preconditions:** Windows 10 x64 (22H2 recommended), clean state.
- **Steps:**
  1. Install with the NSIS installer and run the smoke test.
  2. Run MT-104 and MT-109.
- **Expected result:** If WebView2 is missing, the installer installs it (this
  needs an internet connection during installation only). Every smoke check,
  MT-104 and MT-109 pass.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-124 — Display scaling

- **Area:** Platforms
- **Preconditions:** lockwell installed.
- **Steps:**
  1. In **Settings > System > Display**, set **Scale** to 150 %, start
     lockwell and open every screen and tab.
  2. Repeat at 200 % (if the display allows it) and at 100 %.
- **Expected result:** Text and icons are sharp, nothing overlaps and every
  control is reachable at each scale.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-125 — Standard user without administrator rights

- **Area:** Platforms
- **Preconditions:** A standard (non-administrator) Windows account.
- **Steps:**
  1. Sign in with that account and run the NSIS installer.
  2. Run the smoke test.
- **Expected result:** The installer finishes without asking for an
  administrator password, and every smoke check passes.
- **Result:**
  - [ ] Pass
  - [ ] Fail

### 6.12 Uninstallation and cleanup

#### MT-126 — Uninstall the NSIS version and keep the data

- **Area:** Uninstallation
- **Preconditions:** Installed with NSIS. A vault exists. lockwell closed.
- **Steps:**
  1. In **Settings > Apps > Installed apps**, uninstall **lockwell**.
  2. In the uninstaller, leave **Delete the application data** unticked and
     finish.
  3. Check that `%APPDATA%\com.acevflow.lockwell\vault` still exists.
  4. Reinstall with NSIS, start lockwell and unlock with `MASTER_1`.
- **Expected result:** lockwell disappears from the Start menu and
  **Installed apps**. The vault folder is kept. After reinstalling, the vault
  unlocks with all its data.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-127 — Uninstall the NSIS version and delete the data

- **Area:** Uninstallation
- **Preconditions:** Installed with NSIS. A vault exists. lockwell closed.
- **Steps:**
  1. Uninstall **lockwell** from **Installed apps**.
  2. In the uninstaller, tick **Delete the application data** and finish.
  3. Run:

     ```powershell
     Test-Path "$env:APPDATA\com.acevflow.lockwell", "$env:LOCALAPPDATA\com.acevflow.lockwell"
     ```

- **Expected result:** The app is removed and step 3 prints `False` twice.
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-128 — Uninstall the MSI version

- **Area:** Uninstallation
- **Preconditions:** Installed with MSI. A vault exists. lockwell closed.
- **Steps:**
  1. Uninstall **lockwell** from **Installed apps**.
  2. Check `%APPDATA%\com.acevflow.lockwell\vault`.
- **Expected result:** The app is removed from the Start menu and
  **Installed apps**. The vault folder is kept (remove it by hand with
  MT-129 if it is no longer needed).
- **Result:**
  - [ ] Pass
  - [ ] Fail

#### MT-129 — Remove every trace after testing

- **Area:** Cleanup
- **Preconditions:** lockwell uninstalled.
- **Steps:**
  1. Run:

     ```powershell
     Remove-Item -Recurse -Force "$env:APPDATA\com.acevflow.lockwell", "$env:LOCALAPPDATA\com.acevflow.lockwell", "$env:USERPROFILE\lockwell-test-data", "$env:USERPROFILE\lockwell-test-out" -ErrorAction SilentlyContinue
     Test-Path "$env:APPDATA\com.acevflow.lockwell", "$env:LOCALAPPDATA\com.acevflow.lockwell", "$env:USERPROFILE\lockwell-test-data", "$env:USERPROFILE\lockwell-test-out"
     ```

  2. If clipboard history is on, open Win+V and click **Clear all**.
- **Expected result:** Step 1 prints `False` four times. Nothing of lockwell
  or of the test data remains.
- **Result:**
  - [ ] Pass
  - [ ] Fail

## 7. Recording results

Copy this template into a new file or into the issue tracker for every test
pass. Fill in every field; write "N/A" with a reason for a case you do not
run.

```markdown
# lockwell manual test report

| Field         | Value                                                                |
| ------------- | -------------------------------------------------------------------- |
| Date          |                                                                      |
| Version       | 0.1.0                                                                |
| Commit        | (output of `git rev-parse --short HEAD`)                             |
| Build type    | NSIS / MSI / standalone exe / development                            |
| Tester        |                                                                      |
| Platform      | (Windows edition, version and build, e.g. Windows 11 Pro 24H2 26100) |
| Display scale |                                                                      |
| Scope         | Smoke test / full pass / selected areas                              |

## Results

| Case   | Result (Pass / Fail / N/A) | Defect | Notes |
| ------ | -------------------------- | ------ | ----- |
| MT-001 |                            |        |       |
| MT-002 |                            |        |       |
| …      |                            |        |       |

## Summary

- Cases run:
- Passed:
- Failed:
- Not applicable:

## Defects found

| Defect | Case | Severity (critical / major / minor) | Title | Issue link |
| ------ | ---- | ----------------------------------- | ----- | ---------- |
| D-1    |      |                                     |       |            |
```

Severity guide:

- **Critical:** data loss, data shown or stored in clear text, the vault opens
  without the right password, or the app cannot start.
- **Major:** a feature does not work and there is no workaround.
- **Minor:** wrong text, layout problems or a feature that works with a
  workaround.

## 8. Reporting defects

1. **Security problems** (for example MT-109 to MT-117 failing, or any secret
   visible where it should not be): do **not** open a public issue. Report
   them privately as described in the [security policy](../SECURITY.md).
2. **Other defects:** open an issue on GitHub with the **Bug report** template
   ([new issue](https://github.com/acevflow/lockwell/issues/new/choose)).
   Fill in:
   - **lockwell version** and **Operating system** / **Operating system
     version** from your report header;
   - **Steps to reproduce**: the steps of the failed case, starting with its
     identifier (for example "MT-066, step 1: …");
   - **Expected result**: copied from the case;
   - **Actual result**: what you saw, with the exact message text;
   - **Logs**: only if they contain no personal data.
3. Never attach a vault folder, a backup file, a master password or real
   data. Use only the [test data](#34-test-data), and tick the template's
   confirmation that the issue contains no sensitive data.
4. Write the issue link in the **Defects found** table of your report.

## 9. Traceability to automated tests

The automated suites are:

- **Interface (Vitest and Testing Library):** `src/**/*.test.ts(x)`. The
  Tauri core is replaced by a mock (`src/test/tauriMock.ts`) or by an
  in-memory fake (`src/test/fakeCore.ts`, used by the end-to-end flows in
  `src/e2e/flows.test.tsx`). They check screens, messages and flows, but not
  the real core, window, dialogs or clipboard.
- **Core (Rust):** unit tests in `src-tauri/src/**` and integration tests in
  `src-tauri/tests/`. They run the real vault, cryptography and storage on
  temporary folders, with a fake clipboard and simulated time.

The table lists, for each manual case, the automated tests that check the
same behaviour. "Manual only" means that no automated test covers it, because
it needs the real window, native dialogs, the Windows clipboard or session,
real time, installers or a second computer. A manual case stays useful even
when automated tests exist: it checks the real integration end to end.

| Case             | Automated tests covering the same behaviour (file › test)                                                                                                                                                                                                                                                                                                                                              | Manual only                |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------- |
| MT-001           | CI workflow `.github/workflows/ci.yml` (step "Build")                                                                                                                                                                                                                                                                                                                                                  |                            |
| MT-002           | CI workflow `.github/workflows/ci.yml` (all steps)                                                                                                                                                                                                                                                                                                                                                     |                            |
| MT-003, MT-004   | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-005           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-006           | `src/App.test.tsx` › shows the setup screen when there is no vault; `src-tauri/src/vault/session.rs` › new_folder_has_no_vault                                                                                                                                                                                                                                                                         | Window size                |
| MT-007           | `src/App.test.tsx` › shows the unlock screen when the vault is locked; `src/screens/UnlockScreen.test.tsx` › shows no notice by default                                                                                                                                                                                                                                                                |                            |
| MT-008           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-009           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-010           | None (no network code exists)                                                                                                                                                                                                                                                                                                                                                                          | Yes                        |
| MT-011           | `src/screens/SetupScreen.test.tsx` › creates the vault and reports success; `src/e2e/flows.test.tsx` › creates a vault, stores an entry, copies it and finds it after locking; `src-tauri/tests/vault_lifecycle.rs` › a_new_vault_can_be_reopened_from_disk; `src-tauri/src/vault/session.rs` › create_replaces_a_stale_temporary_file                                                                 | Folder contents            |
| MT-012, MT-013   | `src/screens/SetupScreen.test.tsx` › rejects a short password without calling the core; `src/passwordRules.test.ts`; `src-tauri/src/vault/password_policy.rs` › rejects_short_passwords, accepts_the_minimum_length; `src-tauri/tests/security.rs` › weak_master_passwords_are_refused                                                                                                                 |                            |
| MT-014           | `src/passwordRules.test.ts`; `src-tauri/src/vault/password_policy.rs` › counts_characters_not_bytes                                                                                                                                                                                                                                                                                                    |                            |
| MT-015           | `src/passwordRules.test.ts`; `src-tauri/src/vault/password_policy.rs` › rejects_only_whitespace                                                                                                                                                                                                                                                                                                        |                            |
| MT-016           | `src/passwordRules.test.ts`; `src-tauri/src/vault/password_policy.rs` › enforces_the_maximum_length                                                                                                                                                                                                                                                                                                    |                            |
| MT-017           | `src/screens/SetupScreen.test.tsx` › rejects mismatched passwords                                                                                                                                                                                                                                                                                                                                      |                            |
| MT-018           | `src/screens/SetupScreen.test.tsx` › requires acknowledging that there is no recovery                                                                                                                                                                                                                                                                                                                  |                            |
| MT-019           | Partly: `src/screens/SetupScreen.test.tsx` (each rule separately)                                                                                                                                                                                                                                                                                                                                      | Message order              |
| MT-020           | `src/components/components.test.tsx` › hides the value until shown                                                                                                                                                                                                                                                                                                                                     |                            |
| MT-021           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-022           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-023           | `src/screens/UnlockScreen.test.tsx` › unlocks with the password, disables the button while unlocking; `src-tauri/src/vault/session.rs` › lock_and_unlock                                                                                                                                                                                                                                               |                            |
| MT-024, MT-026   | `src/screens/UnlockScreen.test.tsx` › reports a wrong password and clears the field; `src/e2e/flows.test.tsx` › refuses a wrong master password and then unlocks; `src-tauri/src/vault/session.rs` › unlock_with_wrong_password_stays_locked                                                                                                                                                           |                            |
| MT-025           | `src/screens/UnlockScreen.test.tsx` › asks for a password before calling the core                                                                                                                                                                                                                                                                                                                      |                            |
| MT-027           | `src/App.test.tsx` › locks and returns to the unlock screen, leaves the vault screen even if locking fails; `src/screens/VaultScreen.test.tsx` › locks with the Lock button; `src-tauri/tests/vault_lifecycle.rs` › locked_vaults_refuse_every_operation                                                                                                                                               |                            |
| MT-028           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-029           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-030           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-031           | `src/screens/PasswordsTab.test.tsx` › creates an entry and shows it; `src/components/entries.test.tsx` › submits what was typed in every field; `src-tauri/src/vault/entries.rs` › create_and_get                                                                                                                                                                                                      |                            |
| MT-032, MT-033   | `src/screens/PasswordsTab.test.tsx` › requires a title; `src/components/entries.test.tsx` › requires a title that is not only spaces; `src-tauri/src/vault/entries.rs` › validation_rules                                                                                                                                                                                                              |                            |
| MT-034           | `src/components/entries.test.tsx` › shows None for empty fields and offers nothing to copy                                                                                                                                                                                                                                                                                                             |                            |
| MT-035           | `src-tauri/src/vault/entries.rs` › trims_single_line_fields_but_keeps_secrets_verbatim                                                                                                                                                                                                                                                                                                                 | Clipboard check            |
| MT-036           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-037           | `src-tauri/src/vault/entries.rs` › list_is_sorted_and_has_no_secrets                                                                                                                                                                                                                                                                                                                                   |                            |
| MT-038           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-039 to MT-042 | `src/screens/PasswordsTab.test.tsx` › filters the list with the search box; `src/entryRules.test.ts`                                                                                                                                                                                                                                                                                                   |                            |
| MT-043           | `src/screens/PasswordsTab.test.tsx` › shows an entry with the password masked until revealed; `src/components/entries.test.tsx` › shows and hides the password again                                                                                                                                                                                                                                   | Reset when switching entry |
| MT-044           | `src/screens/PasswordsTab.test.tsx` › edits an entry; `src-tauri/src/vault/entries.rs` › update_keeps_creation_time                                                                                                                                                                                                                                                                                    |                            |
| MT-045, MT-046   | `src/screens/PasswordsTab.test.tsx` › copies the password through the core and says when it is cleared; `src/components/entries.test.tsx` › says when the username was copied and when it will be cleared; `src-tauri/src/commands/security.rs` › copies_the_password_and_the_username                                                                                                                 | Real clipboard             |
| MT-047           | `src/screens/PasswordsTab.test.tsx` › returns to the entry when editing is cancelled                                                                                                                                                                                                                                                                                                                   |                            |
| MT-048           | `src/screens/PasswordsTab.test.tsx` › closes the new entry form when cancelled                                                                                                                                                                                                                                                                                                                         |                            |
| MT-049           | `src/screens/PasswordsTab.test.tsx` › deletes an entry after confirmation; `src-tauri/src/vault/entries.rs` › delete_removes_the_entry                                                                                                                                                                                                                                                                 |                            |
| MT-050           | `src/screens/PasswordsTab.test.tsx` › keeps the entry when deletion is cancelled; `src/components/components.test.tsx` › focuses Cancel and confirms on click, cancels with Escape                                                                                                                                                                                                                     |                            |
| MT-051           | `src/components/entries.test.tsx` › limits every field to what the core accepts; `src-tauri/src/vault/entries.rs` › accepts_every_field_at_its_limit                                                                                                                                                                                                                                                   | Paste behaviour            |
| MT-052           | `src-tauri/src/vault/entries.rs` › names_the_field_that_is_too_long; `src/screens/PasswordsTab.test.tsx` › shows save errors inside the form                                                                                                                                                                                                                                                           |                            |
| MT-053           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-054           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-055           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-056, MT-057   | `src/screens/FilesTab.test.tsx` › adds files and reloads the list, counts several added files; `src-tauri/src/vault/files.rs` › lists_sorted_by_name                                                                                                                                                                                                                                                   | Native dialog              |
| MT-058           | `src/screens/FilesTab.test.tsx` › says nothing when adding is cancelled                                                                                                                                                                                                                                                                                                                                | Native dialog              |
| MT-059           | `src/format.test.ts`; `src/screens/FilesTab.test.tsx` › lists files with their size                                                                                                                                                                                                                                                                                                                    |                            |
| MT-060, MT-061   | `src-tauri/src/vault/files.rs` › round_trips_an_empty_file, round_trips_a_small_file, round_trips_exact_chunk_multiples, round_trips_a_large_file; `src-tauri/tests/entries_files_flow.rs` › files_persist_across_instances_and_export_intact; `src/screens/FilesTab.test.tsx` › saves a copy                                                                                                          | Native dialog              |
| MT-062           | `src/screens/FilesTab.test.tsx` › shows what it is doing and disables the actions meanwhile                                                                                                                                                                                                                                                                                                            | Responsiveness             |
| MT-063           | `src-tauri/src/vault/files.rs` › keeps_unicode_names                                                                                                                                                                                                                                                                                                                                                   | Native dialog              |
| MT-064           | `src/screens/FilesTab.test.tsx` › says nothing when saving a copy is cancelled                                                                                                                                                                                                                                                                                                                         | Native dialog              |
| MT-065           | None (the native dialog asks before replacing)                                                                                                                                                                                                                                                                                                                                                         | Yes                        |
| MT-066, MT-067   | `src-tauri/src/vault/files.rs` › rejects_destinations_inside_the_vault; `src-tauri/tests/entries_files_flow.rs` › the_vault_folder_is_never_used_for_user_files; `src/screens/FilesTab.test.tsx` › dismisses an error                                                                                                                                                                                  |                            |
| MT-068           | `src/screens/FilesTab.test.tsx` › deletes after confirmation; `src-tauri/src/vault/files.rs` › delete_removes_blob_and_row                                                                                                                                                                                                                                                                             |                            |
| MT-069           | `src/screens/FilesTab.test.tsx` › keeps the file when deletion is cancelled                                                                                                                                                                                                                                                                                                                            |                            |
| MT-070           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-071           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-072           | `src/screens/SettingsTab.test.tsx` › changes the master password, clears every field after a change; `src/e2e/flows.test.tsx` › changes the master password and needs the new one afterwards; `src-tauri/src/vault/session.rs` › change_master_password; `src-tauri/src/vault/files.rs` › files_survive_a_password_change; `src-tauri/src/vault/entries.rs` › entries_survive_lock_and_password_change |                            |
| MT-073 to MT-076 | `src/screens/SettingsTab.test.tsx` › validates the new password locally, reports a wrong current password; `src-tauri/src/vault/session.rs` › change_master_password_checks_the_current_one, change_master_password_enforces_the_policy                                                                                                                                                                |                            |
| MT-077           | `src-tauri/src/vault/session.rs` › change_master_password_enforces_the_policy                                                                                                                                                                                                                                                                                                                          |                            |
| MT-078, MT-079   | `src/components/backup.test.tsx` › creates a backup and confirms it, says nothing when cancelled; `src-tauri/src/commands/backup.rs` › names_backups_by_date                                                                                                                                                                                                                                           | Native dialog              |
| MT-080           | `src-tauri/src/vault/backup.rs` › backups_never_go_inside_the_vault                                                                                                                                                                                                                                                                                                                                    |                            |
| MT-081           | `src-tauri/tests/backup_flow.rs` › restoring_replaces_newer_changes; `src-tauri/src/vault/backup.rs` › restore_brings_back_entries_and_files; `src/components/backup.test.tsx` › asks before replacing an existing vault; `src/App.test.tsx` › returns to the unlock screen after restoring a backup                                                                                                   |                            |
| MT-082 to MT-084 | `src/components/backup.test.tsx` › requires the password, does nothing when the replacement is cancelled, shows other errors and stays usable when cancelled                                                                                                                                                                                                                                           | Native dialog              |
| MT-085           | `src-tauri/tests/backup_flow.rs` › a_wrong_password_does_not_open_a_backup; `src-tauri/src/vault/backup.rs` › wrong_password_leaves_the_vault_untouched; `src/components/backup.test.tsx` › explains a wrong password                                                                                                                                                                                  |                            |
| MT-086           | `src-tauri/src/vault/backup.rs` › other_files_are_not_backups                                                                                                                                                                                                                                                                                                                                          |                            |
| MT-087, MT-088   | `src-tauri/tests/backup_flow.rs` › damaged_backups_leave_the_vault_untouched; `src-tauri/src/vault/backup.rs` › damaged_backups_are_rejected_without_side_effects, a_tampered_clear_prefix_is_rejected                                                                                                                                                                                                 |                            |
| MT-089           | `src-tauri/tests/backup_flow.rs` › a_backup_restores_on_another_computer; `src/e2e/flows.test.tsx` › restores a backup instead of creating a vault; `src/screens/SetupScreen.test.tsx` › can restore a backup instead, can go back to creating a vault                                                                                                                                                 |                            |
| MT-090           | `src-tauri/src/vault/backup.rs` › backup_uses_the_password_at_backup_time                                                                                                                                                                                                                                                                                                                              |                            |
| MT-091           | `src-tauri/src/vault/backup.rs` › backup_content_is_encrypted                                                                                                                                                                                                                                                                                                                                          |                            |
| MT-092           | `src-tauri/tests/entries_files_flow.rs` › entries_persist_across_instances, files_persist_across_instances_and_export_intact; `src-tauri/tests/vault_lifecycle.rs` › the_full_life_cycle_keeps_the_data                                                                                                                                                                                                | Real restart               |
| MT-093 to MT-095 | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-096           | `src-tauri/src/vault/files.rs` › a_missing_or_foreign_blob_is_corrupt; `src-tauri/src/vault/backup.rs` › a_backup_with_a_missing_file_is_refused                                                                                                                                                                                                                                                       |                            |
| MT-097           | `src-tauri/src/vault/files.rs` › a_damaged_blob_is_detected_and_nothing_is_written, swapped_blobs_are_detected; `src-tauri/src/vault/entries.rs` › tampered_rows_are_detected, swapped_rows_are_detected                                                                                                                                                                                               |                            |
| MT-098           | `src-tauri/src/vault/session.rs` › new_folder_has_no_vault                                                                                                                                                                                                                                                                                                                                             | Real files                 |
| MT-099           | `src-tauri/tests/backup_flow.rs` › a_backup_restores_on_another_computer (backup path only)                                                                                                                                                                                                                                                                                                            | Folder copy                |
| MT-100           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-101           | `src-tauri/src/security/autolock.rs` › is_idle_at_and_after_the_timeout; `src-tauri/src/state.rs` › lock_if_idle_waits_for_the_timeout; `src-tauri/tests/security.rs` › the_idle_lock_locks_and_clears_the_clipboard; `src/App.test.tsx` › locks when the core emits the lock event and says why; `src/e2e/flows.test.tsx` › explains a lock after inactivity and forgets what was shown               | Real 5-minute timer        |
| MT-102           | `src-tauri/src/security/autolock.rs` › activity_moves_the_deadline; `src-tauri/src/state.rs` › activity_postpones_the_idle_lock; `src/hooks/hooks.test.tsx` › reports at most once per interval; `src/screens/VaultScreen.test.tsx` › reports user activity to the core                                                                                                                                | Real timer                 |
| MT-103           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-104, MT-105   | `src-tauri/src/security/session_lock_windows.rs` › recognizes_only_session_lock_messages; `src/App.test.tsx` › explains a lock caused by Windows                                                                                                                                                                                                                                                       | Real Windows session       |
| MT-106           | `src-tauri/src/security/clipboard.rs` › clears_its_own_copy                                                                                                                                                                                                                                                                                                                                            | Real clipboard and time    |
| MT-107           | `src-tauri/src/security/clipboard.rs` › leaves_content_the_user_copied_since; `src-tauri/tests/security.rs` › locking_clears_a_copied_secret_but_not_user_text                                                                                                                                                                                                                                         | Real clipboard             |
| MT-108           | `src-tauri/src/state.rs` › lock_locks_and_clears_the_copied_secret; `src-tauri/src/security/clipboard.rs` › lock_clears_any_copy                                                                                                                                                                                                                                                                       | Real clipboard             |
| MT-109           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-110           | `src-tauri/tests/entries_files_flow.rs` › nothing_is_stored_in_clear_text; `src-tauri/src/vault/entries.rs` › values_are_encrypted_at_rest; `src-tauri/src/vault/files.rs` › names_and_content_are_not_visible_on_disk                                                                                                                                                                                 | Real installation          |
| MT-111           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-112           | `src-tauri/src/vault/files.rs` › names_and_content_are_not_visible_on_disk                                                                                                                                                                                                                                                                                                                             | Real installation          |
| MT-113           | None (no network code exists)                                                                                                                                                                                                                                                                                                                                                                          | Yes                        |
| MT-114           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-115           | `src/components/components.test.tsx` › hides the value until shown                                                                                                                                                                                                                                                                                                                                     | Spell check, suggestions   |
| MT-116           | `src-tauri/tests/security.rs` › errors_never_contain_passwords_data_or_paths; `src/screens/SetupScreen.test.tsx` › never shows the typed password in an error; `src/screens/UnlockScreen.test.tsx` › never shows the typed password in an error; `src/screens/SettingsTab.test.tsx` › shows other errors with their message and no password                                                            |                            |
| MT-117           | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |
| MT-118 to MT-121 | Partly: `src/screens/VaultScreen.test.tsx` › switches between tabs (MT-120)                                                                                                                                                                                                                                                                                                                            | Yes (visual)               |
| MT-122 to MT-125 | CI runs the automated suites on `windows-latest` only                                                                                                                                                                                                                                                                                                                                                  | Yes                        |
| MT-126 to MT-129 | None                                                                                                                                                                                                                                                                                                                                                                                                   | Yes                        |

**Cases that can only be tested by hand** (no automated equivalent): MT-003,
MT-004, MT-005, MT-008, MT-009, MT-010, MT-021, MT-022, MT-028, MT-029,
MT-030, MT-036, MT-038, MT-053, MT-054, MT-055, MT-065, MT-070, MT-071,
MT-093, MT-094, MT-095, MT-100, MT-103, MT-109, MT-111, MT-113, MT-114,
MT-117, MT-118, MT-119, MT-121 and MT-122 to MT-129.

## 10. Automated verification of this revision

When this guide was written, every check that can run from the command line
was run on commit `ce0fcbe` (version 0.1.0), starting from a clean install
(`npm ci` and `cargo clean`), on Windows 11 Pro Education (build 26200) with
Node.js 26.13.0, npm 11.20.0, Rust 1.99.1, cargo-llvm-cov 0.9.1 and
cargo-audit 0.23.0. Repeat these checks on every revision you test (MT-002).

| Check              | Command                                     | Result                                                                                                                                                                                         |
| ------------------ | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Formatting         | `npm run format:check`                      | Passed                                                                                                                                                                                         |
| Type check         | `npm run typecheck`                         | Passed                                                                                                                                                                                         |
| Lint               | `npm run lint`                              | Passed (no ESLint or clippy warnings)                                                                                                                                                          |
| Tests and coverage | `npm run test:coverage`                     | Passed on the second run: 196 Vitest tests in 17 files; 229 Rust tests (209 unit, 20 integration), 1 ignored. See the notes below                                                              |
| Interface coverage | (part of `test:coverage`)                   | 100 % statements, 98.55 % branches, 100 % functions, 100 % lines                                                                                                                               |
| Core coverage      | (part of `test:coverage`)                   | 89.91 % regions, 80.94 % functions, 89.23 % lines                                                                                                                                              |
| Build              | `npm run tauri build`                       | Passed: `lockwell.exe`, MSI and NSIS installers built                                                                                                                                          |
| npm audit          | `npm audit`                                 | 7 moderate advisories, all in development dependencies (`minimatch` through `typescript-eslint`); `npm audit --omit=dev` finds 0                                                               |
| Rust audit         | `cargo audit` (in `src-tauri`)              | 0 vulnerabilities; 2 warnings: `proc-macro-error` unmaintained (RUSTSEC-2024-0370) and `glib` 0.18.5 unsound (RUSTSEC-2024-0429), both pulled in only by the Linux (GTK) dependencies of Tauri |
| Start-up           | Start `lockwell.exe`, wait 10 seconds, stop | The process stays running with a window titled **lockwell**, opens no TCP or UDP endpoint and does not create the vault folder                                                                 |

Notes:

- On the first run, `src/App.test.tsx` › reports activity to the core at most
  once per interval failed (expected one `touch_activity` call, got none)
  while a release build was compiling in parallel. It passed in three
  isolated runs and in a second full run, so it looks timing-sensitive under
  heavy load. Because `test:coverage` stops at the first failing suite, the
  Rust coverage did not run on the first attempt.
- The ignored Rust test is `src-tauri/src/vault/entries.rs` ›
  debug_output_hides_secrets. It records a known finding: `Entry` and
  `EntryInput` derive `Debug`, which would print the password and notes in
  clear text if a value were ever logged. MT-111 and MT-116 check that no
  secret reaches files or messages in practice.
- Not verified automatically, because they need a person, the real window or
  another system: every case marked "Manual only" in
  [section 9](#9-traceability-to-automated-tests), screen walkthroughs (no
  browser automation can drive the native Tauri window, and the interface
  does not work in a plain browser without the Tauri core), installers and
  uninstallers, Windows session locking, the real clipboard and its history,
  real 5-minute and 30-second timers, Windows 10, display scaling, other
  accounts, macOS and Linux (not supported), and the GitHub security features
  (CodeQL, Dependabot, secret scanning), which run on GitHub.
- The PowerShell commands of this guide (test data, hashes, truncation,
  bit flips, clear-text search and network checks) were run against copies
  of the test data and behaved as described.
