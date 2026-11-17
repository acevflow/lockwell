# src/hooks/useApiError.ts

## Purpose

React hook that holds the error message of a screen of the unlocked vault, and
handles a `locked` error by calling a callback instead of showing a message.

## Place in the architecture

- Used by [`PasswordsTab`](../screens/PasswordsTab.tsx.md) and
  [`FilesTab`](../screens/FilesTab.tsx.md), the two screens that call commands
  needing an unlocked vault.
- A `locked` error means the core locked the vault, for example after
  inactivity. The screen then has to go back to the unlock screen, which the
  `onLocked` callback does. Other errors are shown with an
  [`ErrorBanner`](../components/ErrorBanner.tsx.md).
- It builds on [`isLockedError` and `errorMessage`](../api/client.ts.md).

## Public API

`useApiError(onLocked: () => void)` returns `{ error, report, clear }`:

| Value                         | Description                                                                                   |
| ----------------------------- | --------------------------------------------------------------------------------------------- |
| `error: string \| null`       | The current message for people, or `null`.                                                    |
| `report(cause: unknown)`      | Registers a caught error. A `locked` error calls `onLocked`. Anything else sets `error`.      |
| `clear()`                     | Dismisses the message.                                                                        |

`onLocked` should be stable between renders, because `report` changes with it.
The hook throws nothing.

## Internal structure

One `useState<string | null>` and two `useCallback` functions. `report` depends on
`onLocked`. `clear` has no dependencies and keeps its identity.

## Dependencies

- Internal: [`api/client.ts`](../api/client.ts.md) (`errorMessage`, `isLockedError`).
- External: `react` (`useCallback`, `useState`).

## Usage example

As in [`PasswordsTab`](../screens/PasswordsTab.tsx.md). Behaviour checked by the hook
tests:

```tsx
const { error, report, clear } = useApiError(onLocked);

try {
  await deleteEntry(id);
} catch (cause) {
  report(cause); // "locked" goes to onLocked, others become a message
}
// <ErrorBanner message={error} onDismiss={clear} />
```

## Edge cases, invariants and error handling

- A `locked` error never sets the message, so the banner does not flash before the
  screen changes.
- Reporting a new error replaces the previous message.
- `clear` is safe to call when there is no error.
- The message is formatted by `errorMessage`: capitalized and ending with a
  period.
- If `onLocked` changes on every render, `report` changes too. Components that use
  `report` in an effect dependency, such as
  [`useInitialLoad`](useInitialLoad.ts.md), would then run it again.

## Security

- Messages come from the core and contain no secrets.
- Showing a generic message for a `locked` error avoids exposing screen state after
  the vault has locked.

## Related files and tests

- Related: [`../api/client.ts`](../api/client.ts.md),
  [`useInitialLoad.ts`](useInitialLoad.ts.md),
  [`../screens/PasswordsTab.tsx`](../screens/PasswordsTab.tsx.md),
  [`../screens/FilesTab.tsx`](../screens/FilesTab.tsx.md),
  [`../components/ErrorBanner.tsx`](../components/ErrorBanner.tsx.md).
- Tests: [`hooks.test.tsx`](../../../../src/hooks/hooks.test.tsx) covers the initial
  state, storing a message, `locked` going to the callback and clearing.
