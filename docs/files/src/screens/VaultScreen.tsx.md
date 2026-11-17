# src/screens/VaultScreen.tsx

## Purpose

Screen shown while the vault is unlocked: the tab bar, the Lock button and the active
tab.

## Place in the architecture

[`App`](../App.tsx.md) shows it when the vault is unlocked. Everything decrypted lives
in the state of this component tree, so it is discarded when the vault locks and the
screen unmounts. While mounted it reports user activity to the core
([`useActivityReporter`](../hooks/useActivityReporter.ts.md)) and listens for the core
locking the vault on its own
([`useVaultLockedEvent`](../hooks/useVaultLockedEvent.ts.md)). It hosts the three
tabs: [`PasswordsTab`](PasswordsTab.tsx.md), [`FilesTab`](FilesTab.tsx.md) and
[`SettingsTab`](SettingsTab.tsx.md).

## Public API

Default export `VaultScreen({ onLock, onLocked, onRestored })`:

| Prop         | Type                                | Description                                                                                  |
| ------------ | ----------------------------------- | -------------------------------------------------------------------------------------------- |
| `onLock`     | `() => void`                        | Locks the vault (the Lock button).                                                           |
| `onLocked`   | `(reason?: LockReason) => void`     | Called when the core locked the vault on its own (with the reason) or a command reports `locked` (without one). |
| `onRestored` | `() => void`                        | Called after a backup was restored. The vault is then locked.                                |

`onLocked` should be stable between renders, because the lock-event listener is
registered again when it changes. [`App`](../App.tsx.md) passes a memoized function.

## Internal structure

- A `TABS` list (Passwords, Files, Settings) and a `tab` state, initially
  `passwords`.
- The header has the brand, a `nav` with `role="tablist"` whose buttons have
  `role="tab"`, `aria-selected` and `aria-controls`, and the Lock button.
- The body is a `main` with `role="tabpanel"` labelled by the active tab. Only the
  active tab is rendered.

## Dependencies

- Internal: [`api/security.ts`](../api/security.ts.md) (`LockReason`),
  [`hooks/useActivityReporter.ts`](../hooks/useActivityReporter.ts.md),
  [`hooks/useVaultLockedEvent.ts`](../hooks/useVaultLockedEvent.ts.md),
  [`PasswordsTab.tsx`](PasswordsTab.tsx.md), [`FilesTab.tsx`](FilesTab.tsx.md),
  [`SettingsTab.tsx`](SettingsTab.tsx.md).
- External: `react` (`useState`).

## Usage example

As in [`App`](../App.tsx.md). Checked by the screen tests:

```tsx
<VaultScreen onLock={lock} onLocked={showLocked} onRestored={showRestored} />
```

## Edge cases, invariants and error handling

- Switching tabs unmounts the previous one, so what it held (a selected entry with its
  password, a search text, a half-typed form) is discarded.
- A lock from the core, from the Lock button or from a `locked` error in a tab all end
  with this screen unmounted, so its decrypted state goes with it.
- **Observation, not fixed:** the bar uses the ARIA `tablist` roles but has no
  arrow-key handling. Each tab is a button reached with Tab, whereas the tab pattern
  expects the arrow keys to move between tabs. The code contains no key handling for
  it.

## Security

- Discarding the tab state when switching tabs or locking limits how long decrypted
  data stays in the interface.
- Mounting `useActivityReporter` only here means an idle lock screen does not postpone
  the lock.

## Related files and tests

- Related: [`../App.tsx`](../App.tsx.md), [`PasswordsTab.tsx`](PasswordsTab.tsx.md),
  [`FilesTab.tsx`](FilesTab.tsx.md), [`SettingsTab.tsx`](SettingsTab.tsx.md),
  [`../hooks/useActivityReporter.ts`](../hooks/useActivityReporter.ts.md),
  [`../hooks/useVaultLockedEvent.ts`](../hooks/useVaultLockedEvent.ts.md).
- Tests: [`VaultScreen.test.tsx`](../../../../src/screens/VaultScreen.test.tsx)
  covers the initial tab, switching, the Lock button, a lock by the core with its
  reason, a `locked` error from a tab and the activity reports.
