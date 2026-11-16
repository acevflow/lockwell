# src/screens/SettingsTab.tsx

The Settings section. Changing the master password asks for the current
password and the new one twice, checks the rules locally and calls
`change_master_password`. Success is announced with `role="status"`. Below
it, `CreateBackupPanel` creates an encrypted backup and `RestoreBackupForm`
replaces the vault with a backup, after which the app shows the unlock
screen.
