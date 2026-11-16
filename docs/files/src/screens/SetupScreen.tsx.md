# src/screens/SetupScreen.tsx

First run: the user chooses the master password (twice) and confirms that a
forgotten password cannot be recovered. The rules are checked before calling
`create_vault`; errors from the core are shown in the form.
