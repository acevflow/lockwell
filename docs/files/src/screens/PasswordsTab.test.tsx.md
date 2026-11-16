# src/screens/PasswordsTab.test.tsx

Tests for the Passwords section against an in-memory fake of the core that,
like the core, rejects unknown entry fields:
listing without fetching passwords, the loading state, search, the selected
entry, masked password and reveal, create, required title, edit, cancelling
a new or edited entry, delete with confirmation, cancel and failure, entries
that cannot be opened, dismissing errors, save and load errors, copying the
password and copy failures, and the locked case.
