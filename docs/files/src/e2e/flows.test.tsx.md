# src/e2e/flows.test.tsx

End-to-end flows of the interface: the whole `App` against the fake core of
`src/test/fakeCore.ts`, driven only through what the user sees. Covers the
first run (create the vault, add an entry, copy it, lock, unlock and find it),
a wrong then right master password, changing the master password, the lock
after inactivity that forgets decrypted data, and restoring a backup on a new
computer.
