# src-tauri/src/commands/backup.rs

Backup commands. The core shows the native save and open dialogs, filtered
to `.lwbackup` files.

| Command          | Arguments  | Result                                |
| ---------------- | ---------- | ------------------------------------- |
| `create_backup`  | none       | `true` if saved, `false` if cancelled |
| `restore_backup` | `password` | `true` if restored, `false` if cancelled |

- `create_backup` needs an unlocked vault and suggests the name
  `lockwell-backup-YYYY-MM-DD.lwbackup` (UTC date, computed with Howard
  Hinnant's `civil_from_days` algorithm).
- `restore_backup` works with or without an existing vault. `password` is the
  master password when the backup was made. Afterwards the vault is locked
  and a secret left on the clipboard is cleared.

The dialog helpers live in `dialog.rs`. Tests cover the date conversion and the suggested file name.
