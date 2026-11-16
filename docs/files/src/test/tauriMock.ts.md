# src/test/tauriMock.ts

Test helpers for the Tauri commands. `mockCommands()` makes the mocked
`invoke` answer each command with a handler; a command without a handler
fails, so unexpected calls surface. `failWith()` builds a handler that rejects
with a core error, `emitEvent()` delivers an event to the mocked listeners,
and `callsTo()` returns the arguments of every call to a
command.
