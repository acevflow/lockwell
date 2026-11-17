# src/components/EntryView.tsx

## Purpose

Read-only view of a password entry, with a masked password, a Show toggle and
buttons to copy the username or the password.

## Place in the architecture

Used by [`PasswordsTab`](../screens/PasswordsTab.tsx.md) once an entry has been
loaded with [`getEntry`](../api/entries.ts.md). Copying calls `onCopy`, which the
tab connects to [`copyEntryField`](../api/security.ts.md). The core then puts the
value on the clipboard without it passing through the interface and clears it after
30 seconds.

## Public API

Default export `EntryView(props)`:

| Prop       | Type                                      | Description                                                                |
| ---------- | ----------------------------------------- | -------------------------------------------------------------------------- |
| `entry`    | `Entry`                                   | The complete entry, including its password.                                |
| `onEdit`   | `() => void`                              | Called by the Edit button.                                                 |
| `onDelete` | `() => void`                              | Called by the Delete button. The caller asks for confirmation.             |
| `onCopy`   | `(field: CopyableField) => Promise<void>` | Copies a field. A rejected promise is reported by the caller.              |

It shows the title, the username, the password, the website, the notes and the time
of the last change (`updatedAt`, formatted with `toLocaleString`). Empty fields show
`None` and offer nothing to copy.

## Internal structure

- State: `showPassword` (initially `false`) and `copied`, the last field copied
  successfully.
- The masked password is a fixed row of 12 dots, whatever the real length, so the
  mask does not reveal the length.
- `copy(field)` clears `copied`, awaits `onCopy` and sets `copied` on success. On
  failure it does nothing, because the caller has already shown the error. The
  `copy` function of [`PasswordsTab`](../screens/PasswordsTab.tsx.md) reports the
  error and then throws it again, which this `catch` absorbs.
- After a copy the view says `<Field> copied. It will be cleared from the clipboard
  in 30 seconds.` The number comes from `CLIPBOARD_CLEAR_SECONDS`.
- The caller gives each entry its own instance with a `key`, so the shown state
  never carries over to another entry.

## Dependencies

- Internal: [`api/security.ts`](../api/security.ts.md) (`CLIPBOARD_CLEAR_SECONDS`,
  `CopyableField`), [`api/types.ts`](../api/types.ts.md) (`Entry`).
- External: `react` (`useState`).

## Usage example

As in [`PasswordsTab`](../screens/PasswordsTab.tsx.md). Checked by the component
tests:

```tsx
<EntryView
  key={mode.entry.id}
  entry={mode.entry}
  onEdit={() => setMode({ kind: "edit", entry: mode.entry })}
  onDelete={() => setConfirmDelete(true)}
  onCopy={(field) => copy(mode.entry.id, field)}
/>
```

## Edge cases, invariants and error handling

- The password stays masked until the user presses Show. Pressing Hide masks it
  again.
- A failed copy shows no success message.
- A new copy replaces the previous message.
- **Observation, not fixed:** the success message stays on screen until the next copy
  or until the view changes. It keeps saying the value will be cleared in 30 seconds
  after that time has passed. The core clears the clipboard on time, but the message
  does not update.
- Dates use the browser locale.

## Security

- The password is in the DOM only while shown. The default view contains the mask.
- Copying never reveals the value: the core writes the clipboard and the interface
  only asks for it. The copy therefore works with the password hidden.
- The entry object in memory still holds the password as a plain string, which
  JavaScript cannot wipe. [`PasswordsTab`](../screens/PasswordsTab.tsx.md) keeps it
  in its `mode` state and replaces it when another entry is selected or the mode
  changes.

## Related files and tests

- Related: [`EntryList.tsx`](EntryList.tsx.md), [`EntryForm.tsx`](EntryForm.tsx.md),
  [`../api/security.ts`](../api/security.ts.md),
  [`../screens/PasswordsTab.tsx`](../screens/PasswordsTab.tsx.md).
- Tests: [`entries.test.tsx`](../../../../src/components/entries.test.tsx) covers the
  masked fields, showing and hiding, empty fields, the copy message, a failed copy,
  the change date and the Edit and Delete buttons.
