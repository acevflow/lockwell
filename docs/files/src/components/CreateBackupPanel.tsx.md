# src/components/CreateBackupPanel.tsx

Settings panel to create an encrypted backup of the whole vault. It explains
that the backup opens with the current master password, shows "Backup saved."
on success and nothing if the user cancels. Errors go through `onError`, which
can handle a locked vault.
