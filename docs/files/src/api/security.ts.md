# src/api/security.ts

## Purpose

The interface's side of the automatic lock and the clipboard clearing: reporting
user activity, copying an entry field, and listening for the lock event.

## Place in the architecture

The behaviour itself lives in the core
([`security/`](../../src-tauri/src/security/mod.rs.md) and
[`commands/security.rs`](../../src-tauri/src/commands/security.rs.md)). This file
only calls the commands and subscribes to the event.
[`VaultScreen`](../screens/VaultScreen.tsx.md) uses the activity and event
bindings through the hooks
[`useActivityReporter`](../hooks/useActivityReporter.ts.md) and
[`useVaultLockedEvent`](../hooks/useVaultLockedEvent.ts.md).
[`EntryView`](../components/EntryView.tsx.md) and
[`PasswordsTab`](../screens/PasswordsTab.tsx.md) use the copy binding.

## Public API

| Item                                                         | Description                                                                                  |
| ------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| `VAULT_LOCKED_EVENT = "vault-locked"`                        | Event the core emits when it locks the vault on its own. Same name as in `security/mod.rs`.  |
| `type LockReason = "idle" \| "systemLock"`                   | Why the core locked the vault.                                                               |
| `type CopyableField = "username" \| "password"`              | A field that can be copied.                                                                  |
| `CLIPBOARD_CLEAR_SECONDS = 30`                               | Seconds after which the core clears a copied secret. Used only to tell the user.             |
| `touchActivity(): Promise<void>`                             | Records user activity, postponing the automatic lock (`touch_activity`).                     |
| `copyEntryField(id: string, field: CopyableField): Promise<void>` | Copies the field to the clipboard from the core (`copy_entry_field`). `locked`, `invalid_input`, `not_found`, `io`. An empty field gives `invalid_input`. |
| `onVaultLocked(handler: (reason: LockReason) => void): Promise<UnlistenFn>` | Calls `handler` each time the core locks the vault. Resolves to the function that unsubscribes. |

## Internal structure

`touchActivity` and `copyEntryField` are one `call` each. `onVaultLocked` uses
`listen` from `@tauri-apps/api/event` on `VAULT_LOCKED_EVENT` and passes
`event.payload.reason` to the handler. The event payload from the core is
`{ "reason": "idle" }` or `{ "reason": "systemLock" }`.

## Dependencies

- Internal: [`client.ts`](client.ts.md) (`call`).
- External: `@tauri-apps/api/event` (`listen`, `UnlistenFn`).

## Usage example

Checked by the wrapper tests with the bridge mocked:

```ts
import { onVaultLocked, copyEntryField, touchActivity } from "./security";

const unlisten = await onVaultLocked((reason) => {
  // reason is "idle" or "systemLock": drop decrypted state and show the unlock screen
});
await copyEntryField(id, "password");   // the value never reaches the interface
await touchActivity();
unlisten();                              // when the listener is no longer needed
```

## Edge cases, invariants and error handling

- `CLIPBOARD_CLEAR_SECONDS` mirrors `CLEAR_AFTER` in
  [`security/clipboard.rs`](../../src-tauri/src/security/clipboard.rs.md). Nothing
  checks that the two agree, so a change in one needs a change in the other. It
  only affects the text shown to the user, because the core does the clearing.
- The event is emitted only when the core locks on its own. A lock requested by the
  user through [`lockVault`](vault.ts.md) does not emit it.
- If the event is lost, the next command fails with `locked` and the interface
  can recover from that.
- `onVaultLocked` returns a promise, so the subscription is not active until it
  resolves. Callers must call the returned function to avoid duplicate handlers.

## Security

- `copyEntryField` keeps the secret out of the interface: the core reads the value
  and writes the clipboard. The clipboard is cleared after 30 seconds if
  unchanged, and when the vault locks.
- Calling `touchActivity` resets the inactivity clock, so it should follow real
  user actions only.
- The lock event carries only the reason, never vault data.

## Related files and tests

- Related: [`client.ts`](client.ts.md),
  [`../hooks/useActivityReporter.ts`](../hooks/useActivityReporter.ts.md),
  [`../hooks/useVaultLockedEvent.ts`](../hooks/useVaultLockedEvent.ts.md),
  [`../components/EntryView.tsx`](../components/EntryView.tsx.md),
  [`../../src-tauri/src/security/mod.rs`](../../src-tauri/src/security/mod.rs.md),
  [`../../src-tauri/src/commands/security.rs`](../../src-tauri/src/commands/security.rs.md).
- Tests: [`api.test.ts`](../../../../src/api/api.test.ts) checks the command
  arguments and that the lock-event listener passes the reason to the handler.
