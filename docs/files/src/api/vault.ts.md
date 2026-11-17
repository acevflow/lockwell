# src/api/vault.ts

## Purpose

Typed wrappers for the vault commands: status, create, unlock, lock and change
the master password.

## Place in the architecture

One function per Rust command in
[`commands/vault.rs`](../../src-tauri/src/commands/vault.rs.md), all going through
[`call`](client.ts.md). The screens
[`SetupScreen`](../screens/SetupScreen.tsx.md),
[`UnlockScreen`](../screens/UnlockScreen.tsx.md) and
[`SettingsTab`](../screens/SettingsTab.tsx.md), and the root
[`App`](../App.tsx.md), call them.

## Public API

| Function                                                       | Command                  | Resolves to / errors                                                            |
| -------------------------------------------------------------- | ------------------------ | ------------------------------------------------------------------------------- |
| `getVaultStatus(): Promise<VaultStatus>`                       | `vault_status`           | `{ exists, unlocked }`.                                                         |
| `createVault(password: string): Promise<void>`                 | `create_vault`           | Creates the vault and unlocks it. `invalid_input` or `vault_exists`.            |
| `unlockVault(password: string): Promise<void>`                 | `unlock_vault`           | `wrong_password`, `vault_not_found`, `corrupt` or `unsupported_version`.        |
| `lockVault(): Promise<void>`                                   | `lock_vault`             | The core wipes its key and clears a copied secret.                              |
| `changeMasterPassword(current: string, next: string): Promise<void>` | `change_master_password` | `locked`, `wrong_password` or `invalid_input`.                            |

Every function rejects with an [`ApiError`](client.ts.md). The wrapper sends the
new password under the argument name `new`, as the Rust command expects, because
`new` is not usable as a parameter name in TypeScript:

```ts
call<void>("change_master_password", { current, new: next });
```

## Internal structure

No logic. Each function is a one-line call to `call` with the command name and
the argument object (camelCase keys that match the Rust parameter names).

## Dependencies

- Internal: [`client.ts`](client.ts.md) (`call`), [`types.ts`](types.ts.md)
  (`VaultStatus`).
- External: none directly. `@tauri-apps/api` is used through `client.ts`.

## Usage example

The calls are checked by the wrapper tests with the Tauri bridge mocked:

```ts
import { createVault, unlockVault, changeMasterPassword } from "./vault";

await createVault("correct horse battery");
await unlockVault("correct horse battery");
await changeMasterPassword("correct horse battery", "another long password");
// invoke("change_master_password", { current: "...", new: "..." })
```

## Edge cases, invariants and error handling

- The core enforces the password rules (minimum 12 characters, at most 1024). The
  copy in [`passwordRules.ts`](../passwordRules.ts.md) only gives early feedback.
- A failed unlock leaves the vault locked.
- `lockVault` resolves even if the vault was already locked.
- The key derivation can take a moment, so the promises of `createVault`,
  `unlockVault` and `changeMasterPassword` may stay pending for that time.

## Security

- Passwords are plain strings in JavaScript and cannot be wiped. They travel once
  to the core, which moves them into wiped buffers. Callers should drop their
  references as soon as the call ends, for example by clearing form state.
- Nothing here logs or stores a password.

## Related files and tests

- Related: [`client.ts`](client.ts.md), [`types.ts`](types.ts.md),
  [`../screens/SetupScreen.tsx`](../screens/SetupScreen.tsx.md),
  [`../screens/UnlockScreen.tsx`](../screens/UnlockScreen.tsx.md),
  [`../screens/SettingsTab.tsx`](../screens/SettingsTab.tsx.md),
  [`../../src-tauri/src/commands/vault.rs`](../../src-tauri/src/commands/vault.rs.md).
- Tests: [`api.test.ts`](../../../../src/api/api.test.ts) checks the command name
  and arguments of every wrapper.
