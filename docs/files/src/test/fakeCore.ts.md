# src/test/fakeCore.ts

`installFakeCore()`: a stateful, in-memory stand-in for the core used by the
end-to-end tests. It keeps the vault, its master password, the entries, what
was copied to the clipboard and an optional backup for the restore dialog, and
answers like the core: `locked` for commands of a locked vault,
`wrong_password` for a wrong password and `invalid_input` for unknown entry
fields. It never reads the clock.
