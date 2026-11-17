# src/components/CreateBackupPanel.tsx

## Purpose

Settings panel that creates an encrypted backup of the whole vault.

## Place in the architecture

Used by [`SettingsTab`](../screens/SettingsTab.tsx.md). The button calls
[`createBackup`](../api/backup.ts.md), and the core shows the native save dialog and
writes the file ([`commands/backup.rs`](../../src-tauri/src/commands/backup.rs.md)).
Cancelling the dialog is not an error and shows nothing.

## Public API

Default export `CreateBackupPanel({ onError })`:

| Prop      | Type                                  | Description                                                                  |
| --------- | ------------------------------------- | ---------------------------------------------------------------------------- |
| `onError` | `(error: unknown) => string \| null`  | Reports an error. Returns the message to show, or `null` if it was handled.  |

The parent decides how an error is turned into text. For example, it can treat a
`locked` error by going back to the unlock screen and return `null`.

## Internal structure

State: `busy`, `error` and `success`. `create()` clears the messages, sets `busy`,
awaits `createBackup()` and, if it resolves to `true`, sets `Backup saved.` On a
rejection it sets `error` to whatever `onError` returns. A `finally` block always
clears `busy`. The panel shows an [`ErrorBanner`](ErrorBanner.tsx.md), a
`role="status"` success line and the button (`Create backup`, or `Saving backup…`
while busy).

## Dependencies

- Internal: [`api/backup.ts`](../api/backup.ts.md), [`ErrorBanner.tsx`](ErrorBanner.tsx.md).
- External: `react` (`useState`).

## Usage example

As in [`SettingsTab`](../screens/SettingsTab.tsx.md). Checked by the component tests:

```tsx
<CreateBackupPanel onError={backupError} />

// backupError in SettingsTab: a "locked" error goes back to the unlock screen and
// returns null (nothing to show), any other error becomes a message.
function backupError(cause: unknown): string | null {
  if (isLockedError(cause)) {
    onLocked();
    return null;
  }
  return errorMessage(cause);
}
```

## Edge cases, invariants and error handling

- A cancelled dialog (`false`) leaves the panel as it was: no message.
- Starting a new backup clears the previous success or error message.
- The button is disabled while a backup is being written.
- The panel explains that the backup opens with the current master password, so the
  user remembers which password a later restore needs.

## Security

- The backup file is encrypted. Its privacy depends on the master password it was
  made under.
- The interface never sees the path of the file. The core owns the dialog.

## Related files and tests

- Related: [`RestoreBackupForm.tsx`](RestoreBackupForm.tsx.md),
  [`../api/backup.ts`](../api/backup.ts.md),
  [`../screens/SettingsTab.tsx`](../screens/SettingsTab.tsx.md),
  [`../../src-tauri/src/vault/backup.rs`](../../src-tauri/src/vault/backup.rs.md).
- Tests: [`backup.test.tsx`](../../../../src/components/backup.test.tsx) covers
  creating and confirming, saying nothing when cancelled and showing the message that
  `onError` returns.
