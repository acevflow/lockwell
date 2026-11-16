# src/api/backup.ts

Wrappers for the backup commands: `createBackup()` and
`restoreBackup(password)`. Both resolve to `false` if the user cancels the
file dialog, which the core shows.
