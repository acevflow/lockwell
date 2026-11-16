# src/screens/PasswordsTab.test.tsx

Tests for the Passwords section against an in-memory fake of the core that,
like the core, rejects unknown entry fields:
listing without fetching passwords, search, masked password and reveal,
create, required title, edit, delete with confirmation and cancel, save and
load errors, copying the password and copy failures, and the locked case.
