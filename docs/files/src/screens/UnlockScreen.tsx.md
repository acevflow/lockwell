# src/screens/UnlockScreen.tsx

## Purpose

Screen shown while the vault is locked. It asks for the master password.

## Place in the architecture

[`App`](../App.tsx.md) shows it when a vault exists and is locked: on start, after a
lock by the user, after an automatic lock and after a restore. It calls
[`unlockVault`](../api/vault.ts.md) and reports success with `onUnlocked`. The
`notice` explains why the vault is locked when it was not the user's choice, for
example the 5-minute inactivity lock.

## Public API

Default export `UnlockScreen({ onUnlocked, notice })`:

| Prop         | Type         | Description                                              |
| ------------ | ------------ | -------------------------------------------------------- |
| `onUnlocked` | `() => void` | Called once the vault is unlocked.                       |
| `notice`     | `string`     | Optional. Why the vault was locked automatically.        |

## Internal structure

State: `password`, `error` and `busy`. On submit it refuses an empty password
(`Enter your master password.`) without calling the core. Otherwise it sets `busy`
and calls `unlockVault`. On success it clears the password and calls `onUnlocked`.
On failure it shows `Incorrect master password.` for a `wrong_password` error and the
formatted message for any other, clears the password and enables the button again.

The notice is a `p.notice` with `role="status"`. The password field is a
[`PasswordInput`](../components/PasswordInput.tsx.md) with `autoComplete="current-password"`
and `autoFocus`.

## Dependencies

- Internal: [`api/client.ts`](../api/client.ts.md) (`errorMessage`,
  `isWrongPasswordError`), [`api/vault.ts`](../api/vault.ts.md),
  [`components/ErrorBanner.tsx`](../components/ErrorBanner.tsx.md),
  [`components/PasswordInput.tsx`](../components/PasswordInput.tsx.md).
- External: `react` (`useState`, `FormEvent`).

## Usage example

As in [`App`](../App.tsx.md). Checked by the screen tests:

```tsx
<UnlockScreen
  notice={phase.notice}
  onUnlocked={() => setPhase({ kind: "unlocked" })}
/>
```

## Edge cases, invariants and error handling

- The typed password is cleared after every attempt, successful or not.
- The button reads `Unlocking…` and is disabled while the core derives the key, which
  takes a moment by design.
- A wrong password and a damaged key header give the same message, because the core
  reports them as one error.
- The screen has no counter or delay between attempts. Each attempt costs one
  Argon2id derivation in the core, which is what slows guessing.
- A vault that cannot be read (`corrupt`, `unsupported_version`, `vault_not_found`)
  shows the core's message.

## Security

- The password is a plain string in state until cleared. It is cleared right after
  the attempt.
- Error messages never include the typed password. A test checks it.
- The notice tells the user when the lock was automatic, so an unexpected lock is
  never silent.

## Related files and tests

- Related: [`../App.tsx`](../App.tsx.md), [`SetupScreen.tsx`](SetupScreen.tsx.md),
  [`VaultScreen.tsx`](VaultScreen.tsx.md), [`../api/vault.ts`](../api/vault.ts.md),
  [`../components/PasswordInput.tsx`](../components/PasswordInput.tsx.md).
- Tests: [`UnlockScreen.test.tsx`](../../../../src/screens/UnlockScreen.test.tsx)
  covers unlocking, a wrong password and clearing the field, other errors, the empty
  password, the notice, the disabled button and the absence of the password in errors.
