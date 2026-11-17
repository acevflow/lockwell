# src/components/ConfirmDialog.tsx

## Purpose

Modal confirmation dialog shown before destructive or irreversible actions.

## Place in the architecture

A presentational component with no state of its own. It is used by
[`PasswordsTab`](../screens/PasswordsTab.tsx.md) (delete an entry),
[`FilesTab`](../screens/FilesTab.tsx.md) (delete a file) and
[`RestoreBackupForm`](RestoreBackupForm.tsx.md) (replace the vault). The parent
decides when to render it and what happens on confirm or cancel.

## Public API

Default export `ConfirmDialog(props)`:

| Prop           | Type         | Description                                                                 |
| -------------- | ------------ | --------------------------------------------------------------------------- |
| `title`        | `string`     | Heading of the dialog. It also names the dialog for screen readers.          |
| `message`      | `string`     | Explains what will happen if the user confirms.                              |
| `confirmLabel` | `string`     | Text of the confirm button, such as `Delete`.                                |
| `onConfirm`    | `() => void` | Called when the user presses the confirm button.                             |
| `onCancel`     | `() => void` | Called by the Cancel button and by Escape (unless `busy`).                   |
| `busy`         | `boolean`    | Optional, default `false`. Disables both buttons while the action runs.      |

It returns a `div.dialog-backdrop` containing a `div.dialog` with
`role="alertdialog"`, `aria-modal="true"`, and `aria-labelledby` and
`aria-describedby` pointing to the title and message. Side effect: while mounted
it listens to `keydown` on `window`.

## Internal structure

- `useId` creates the ids that tie the title and the message to the dialog.
- A `useEffect` registers the Escape handler and removes it on cleanup. It depends
  on `busy` and `onCancel`, so it is registered again if either changes.
- The Cancel button has `autoFocus`, so the initial focus is on the safe choice.
- The confirm button uses the `danger` style.

## Dependencies

- Internal: none. Styles are in `src/styles.css`.
- External: `react` (`useEffect`, `useId`).

## Usage example

As in [`RestoreBackupForm`](RestoreBackupForm.tsx.md):

```tsx
{confirming && (
  <ConfirmDialog
    title="Replace this vault?"
    message="All current passwords and files will be replaced by the backup. This cannot be undone."
    confirmLabel="Replace vault"
    onCancel={() => setConfirming(false)}
    onConfirm={() => void restore()}
  />
)}
```

Checked by the component tests (`vitest run src/components`).

## Edge cases, invariants and error handling

- Cancel has the initial focus and Escape cancels, so a stray Enter never confirms
  by accident.
- While `busy` is true, both buttons are disabled and Escape does nothing.
- `onCancel` should be stable between renders, otherwise the Escape listener is
  registered again each time.
- **Observation, not fixed:** the dialog marks itself modal (`aria-modal`) but does
  not trap keyboard focus. Pressing Tab can move focus to the page behind it.
  The code contains no focus trap, and the tests do not check one.

## Security

The dialog protects against accidental data loss: destructive actions need an
explicit second step. It handles no secrets.

## Related files and tests

- Related: [`RestoreBackupForm.tsx`](RestoreBackupForm.tsx.md),
  [`../screens/PasswordsTab.tsx`](../screens/PasswordsTab.tsx.md),
  [`../screens/FilesTab.tsx`](../screens/FilesTab.tsx.md).
- Tests: [`components.test.tsx`](../../../../src/components/components.test.tsx)
  covers the initial focus on Cancel, confirming, cancelling with Escape, and
  ignoring Escape while busy.
