# src/screens/UnlockScreen.tsx

Asks for the master password and calls `unlock_vault`. A wrong password shows
"Incorrect master password." and clears the field. It can show a notice that
explains an automatic lock.
