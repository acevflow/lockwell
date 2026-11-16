# src/api/api.test.ts

Tests for the command wrappers in `vault.ts`, `entries.ts`, `files.ts`,
`backup.ts` and `security.ts`: each calls its command with the exact argument
names the core expects and turns a core error into an `ApiError`. Also covers
`onVaultLocked`: it passes on the lock reason, ignores other events and
resolves to the function that stops listening.
