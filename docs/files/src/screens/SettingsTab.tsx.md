# src/screens/SettingsTab.tsx

The Settings section. Changing the master password asks for the current
password and the new one twice, checks the rules locally and calls
`change_master_password`. Success is announced with `role="status"`.
