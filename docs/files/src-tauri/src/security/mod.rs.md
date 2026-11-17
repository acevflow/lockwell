# src-tauri/src/security/mod.rs

## Purpose

Root of the security controls around the vault: automatic lock after
inactivity or when Windows locks, notification of the interface, and clearing of
copied secrets from the clipboard.

## Place in the architecture

- Declares [`autolock`](autolock.rs.md), [`clipboard`](clipboard.rs.md) and, on
  Windows, [`session_lock_windows`](session_lock_windows.rs.md).
- Starts the background threads. The decisions (what to lock and when) live in
  [`AppState`](../state.rs.md). This file only wires timers and events to it.
- When the core locks the vault on its own, it emits `VAULT_LOCKED_EVENT` so the
  interface drops its decrypted state.
- [`lib.rs`](../lib.rs.md) calls `spawn_idle_watcher` and `watch_system_lock`
  during setup. [`commands/security.rs`](../commands/security.rs.md) calls
  `schedule_clipboard_clear`.

## Public API

| Item                                                                  | Description                                                              |
| --------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `VAULT_LOCKED_EVENT = "vault-locked"`                                 | Event name the interface listens for.                                    |
| `enum LockReason { Idle, SystemLock }`                                | Serialized as `"idle"` and `"systemLock"` (camelCase).                   |
| `notify_locked(app: &AppHandle<R>, reason: LockReason)`               | Emits the event with payload `{"reason": ...}`.                          |
| `spawn_idle_watcher(app: AppHandle<R>) -> io::Result<()>`             | Starts the thread that locks after the inactivity timeout.               |
| `watch_system_lock(app: AppHandle<R>) -> Result<(), String>`          | Windows only. Locks the vault when the session locks.                    |
| `schedule_clipboard_clear(state: AppState, generation: u64) -> io::Result<()>` | Clears copy `generation` after 30 seconds if unchanged.         |

Errors: `spawn_idle_watcher` and `schedule_clipboard_clear` fail if the thread
cannot be started. `watch_system_lock` returns the text of the failure if the
watcher cannot start. `notify_locked` does not return an error. If the event
cannot be emitted, it writes a line to standard error that carries the error and
no vault data.

Side effects: each function that starts a thread creates it, and the idle
watcher runs until the process ends.

## Internal structure

- The idle watcher thread (`idle-watcher`) loops forever: it sleeps
  `CHECK_INTERVAL` (5 seconds), calls `AppState::lock_if_idle(Instant::now())`
  and notifies the interface with `LockReason::Idle` only if the vault was locked.
  The lock can be up to 5 seconds late.
- `watch_system_lock` gives [`session_lock_windows::watch`](session_lock_windows.rs.md)
  a callback. It calls `AppState::lock()` and notifies with
  `LockReason::SystemLock` only if the vault was unlocked.
- `schedule_clipboard_clear` starts a `clipboard-clear` thread per copy. It
  sleeps `CLEAR_AFTER`, then calls `AppState::clear_clipboard_if_unchanged`.
- The payload type `VaultLockedPayload` is private. The reason is serialized as
  `{"reason":"systemLock"}` or `{"reason":"idle"}`.

## Dependencies

- Internal: [`state.rs`](../state.rs.md) (`AppState`), the three submodules.
- External: `tauri` (`AppHandle`, `Emitter`, `Manager`, `Runtime`), `serde`
  (payload).

## Usage example

The thread-starting functions need a running Tauri application, so they were not
run on their own. The event payload was checked through the inline test, and the
clock and clipboard logic they use was run directly (see
[`autolock.rs`](autolock.rs.md#usage-example) and
[`clipboard.rs`](clipboard.rs.md#usage-example)). Wiring in
[`lib.rs`](../lib.rs.md):

```rust
security::spawn_idle_watcher(app.handle().clone())?;
#[cfg(windows)]
security::watch_system_lock(app.handle().clone())?;
```

## Edge cases, invariants and error handling

- The interface is told about a lock only if the vault was actually unlocked, so
  a lock that finds the vault locked is silent.
- If the event cannot be sent, the vault is already locked. The interface finds
  out on its next command, which fails with `locked`.
- One clipboard timer thread exists per copy. Each sleeps for 30 seconds even if
  the vault locks earlier. A newer copy makes older timers do nothing.
- The idle watcher thread cannot be stopped and ends with the process.

## Security

- Implements two requirements: lock after 5 minutes of inactivity and when
  Windows locks, and clear copied secrets after 30 seconds.
- Locking is decided in `AppState`, which also clears the clipboard, so every
  path (command, timer, system lock) behaves the same way.
- The event carries only the reason, never vault data.
- **Observation, not fixed:** a copied secret is not cleared if the app exits
  before the 30-second timer fires (see [`clipboard.rs`](clipboard.rs.md#security)).

## Related files and tests

- Related: [`autolock.rs`](autolock.rs.md), [`clipboard.rs`](clipboard.rs.md),
  [`session_lock_windows.rs`](session_lock_windows.rs.md),
  [`../state.rs`](../state.rs.md), [`lib.rs`](../lib.rs.md),
  [`commands/security.rs`](../commands/security.rs.md).
- Tests: inline `tests` module in
  [`mod.rs`](../../../../../src-tauri/src/security/mod.rs): the serialized event
  payload. Integration tests:
  [`security.rs`](../../../../../src-tauri/tests/security.rs). The interface side
  of the event is documented in `src/api/security.ts`.
