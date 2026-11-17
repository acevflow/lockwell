# src-tauri/src/commands/backup.rs

## Purpose

Commands to create an encrypted backup and to restore one. The core shows the
native save and open dialogs.

## Place in the architecture

Wrappers over [`vault/backup.rs`](../vault/backup.rs.md) through
[`AppState`](../state.rs.md), with the dialog helpers of
[`dialog.rs`](dialog.rs.md). The interface calls them through
`src/api/backup.ts`.

## Public API

| Command                                                    | Description                                                                                                           | Errors (`code`)                                                              |
| ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `create_backup() -> bool` (async)                          | Asks where to save a backup and writes it. `false` means cancelled.                                                   | `locked`, `invalid_input` (destination inside the vault), `corrupt`, `io`.   |
| `restore_backup(password: String) -> bool` (async)         | Asks for a backup file and replaces the vault with it. `password` is the master password the vault had when the backup was made. The vault is left locked. `false` means cancelled. | `invalid_input`, `wrong_password`, `corrupt`, `unsupported_version`, `io`.   |

Side effects: dialogs are shown. `create_backup` writes the file. `restore_backup`
replaces the vault folder, locks the vault and clears the clipboard.

## Internal structure

- `create_backup` checks that the vault is unlocked, builds the suggested file
  name `lockwell-backup-YYYY-MM-DD.lwbackup`, shows a save dialog with a
  `lockwell backup` filter and calls `UnlockedVault::create_backup`.
- `restore_backup` first validates the password (see below), then shows an open
  dialog with the same filter and calls `Vault::restore_backup`. After a
  successful restore it calls `state.lock()` again, which also clears a secret
  left on the clipboard by the replaced vault.
- `backup_file_name(millis)` formats the date. `civil_date(days)` converts days
  since 1970-01-01 to a UTC year, month and day with Howard Hinnant's
  `civil_from_days` algorithm, so no date library is needed. Negative days are
  handled (`-1` is 1969-12-31).

## Dependencies

- Internal: [`dialog.rs`](dialog.rs.md), [`state.rs`](../state.rs.md),
  [`vault/backup.rs`](../vault/backup.rs.md), [`vault/mod.rs`](../vault/mod.rs.md)
  (`now_millis`), [`vault/password_policy.rs`](../vault/password_policy.rs.md)
  (`MAX_MASTER_PASSWORD_CHARS`), [`error.rs`](../error.rs.md).
- External: `tauri`, `tauri_plugin_dialog`, `zeroize`.

## Usage example

The dialogs need a window, so the commands were not run end to end. The core
calls were run against the library (see
[`vault/backup.rs`](../vault/backup.rs.md#usage-example)), and the file name logic
was run through the inline tests:

```text
backup_file_name(1_794_830_400_000) == "lockwell-backup-2026-11-16.lwbackup"
backup_file_name(0)                 == "lockwell-backup-1970-01-01.lwbackup"
```

## Edge cases, invariants and error handling

- Cancelling a dialog returns `false`, not an error.
- `restore_backup` rejects an empty password, or one longer than 1024
  characters, with `invalid_input` (`enter the master password of the backup`)
  before any dialog is shown. It does not apply the minimum length rule, because
  the backup may come from a vault made under older rules.
- On any restore error the current vault is left as it was.
- The date in the suggested name is in UTC, so close to midnight it can differ
  from the user's local date.

## Security

- The password is moved into a `Zeroizing` string at once.
- The backup is only as private as the master password it was made under. A
  restore needs that password, not the current one.
- The destination and the source come from native dialogs, never from the
  interface.

## Related files and tests

- Related: [`mod.rs`](mod.rs.md), [`dialog.rs`](dialog.rs.md),
  [`vault/backup.rs`](../vault/backup.rs.md), [`state.rs`](../state.rs.md).
- Tests: inline `tests` module in
  [`backup.rs`](../../../../../src-tauri/src/commands/backup.rs): date conversion
  and file naming. Integration tests:
  [`backup_flow.rs`](../../../../../src-tauri/tests/backup_flow.rs).
