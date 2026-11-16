# src/components/RestoreBackupForm.tsx

Form to restore an encrypted backup. It asks for the master password the vault
had when the backup was made; the core then shows the file dialog. When a
vault exists (`replacesVault`), it warns and asks for confirmation before
replacing it. A wrong password shows "That password does not open this
backup." Used in the settings and on the setup screen.
