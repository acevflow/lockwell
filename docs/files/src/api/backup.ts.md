# src/api/backup.ts

## Purpose

Typed wrappers for the encrypted backup commands: create a backup and restore
one.

## Place in the architecture

Two functions over the commands in
[`commands/backup.rs`](../../src-tauri/src/commands/backup.rs.md), through
[`call`](client.ts.md). The core shows the native save and open dialogs.
[`CreateBackupPanel`](../components/CreateBackupPanel.tsx.md) and
[`RestoreBackupForm`](../components/RestoreBackupForm.tsx.md) use them.

## Public API

| Function                                      | Command          | Resolves to / errors                                                                     |
| --------------------------------------------- | ---------------- | ---------------------------------------------------------------------------------------- |
| `createBackup(): Promise<boolean>`            | `create_backup`  | `true` if saved, `false` if cancelled. `locked`, `invalid_input`, `corrupt`, `io`.       |
| `restoreBackup(password: string): Promise<boolean>` | `restore_backup` | Replaces the vault and leaves it locked. `false` if cancelled. `invalid_input`, `wrong_password`, `corrupt`, `unsupported_version`. |

`password` for `restoreBackup` is the master password the vault had when the
backup was made, not the current one. Both functions reject with an
[`ApiError`](client.ts.md).

## Internal structure

No logic: each function is one `call` with the command name and, for the restore,
`{ password }`.

## Dependencies

- Internal: [`client.ts`](client.ts.md).
- External: none directly.

## Usage example

Checked by the wrapper tests with the bridge mocked:

```ts
import { createBackup, restoreBackup } from "./backup";

const saved = await createBackup();                       // false if cancelled
const restored = await restoreBackup("password at backup time");
// after a restore the vault is locked: the user unlocks it again
```

## Edge cases, invariants and error handling

- Cancelling a dialog is a normal result (`false`), not an error.
- An empty password, or one longer than 1024 characters, is rejected by the core
  with `invalid_input` before any dialog is shown.
- After a successful restore the vault is locked and the data is the one in the
  backup, so the screen must go back to the unlock step.
- A wrong password leaves the current vault untouched.

## Security

- The password is a plain string in JavaScript and cannot be wiped. It travels
  once to the core.
- The backup file is encrypted. Its privacy depends on the master password it was
  made under.
- The interface never sees the backup path.

## Related files and tests

- Related: [`client.ts`](client.ts.md),
  [`../components/CreateBackupPanel.tsx`](../components/CreateBackupPanel.tsx.md),
  [`../components/RestoreBackupForm.tsx`](../components/RestoreBackupForm.tsx.md),
  [`../../src-tauri/src/commands/backup.rs`](../../src-tauri/src/commands/backup.rs.md).
- Tests: [`api.test.ts`](../../../../src/api/api.test.ts) checks each command name
  and arguments.
