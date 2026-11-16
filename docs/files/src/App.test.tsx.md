# src/App.test.tsx

Tests for `App`: the loading state, the screen chosen for each vault status,
status errors, opening the vault once it is created, locking (including when
`lock_vault` fails) and returning to the unlock screen when the core reports
a locked vault, the lock event with each reason, throttled activity reports
and restoring a backup.
