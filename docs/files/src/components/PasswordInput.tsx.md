# src/components/PasswordInput.tsx

## Purpose

A labelled password field with a Show or Hide toggle, used wherever a password is
typed.

## Place in the architecture

A controlled input shared by [`EntryForm`](EntryForm.tsx.md),
[`RestoreBackupForm`](RestoreBackupForm.tsx.md),
[`SetupScreen`](../screens/SetupScreen.tsx.md),
[`UnlockScreen`](../screens/UnlockScreen.tsx.md) and
[`SettingsTab`](../screens/SettingsTab.tsx.md). The parent owns the text. The
component owns only whether it is visible.

## Public API

Default export `PasswordInput(props)`:

| Prop           | Type                      | Description                                                      |
| -------------- | ------------------------- | ---------------------------------------------------------------- |
| `id`           | `string`                  | Id of the input. The label points to it.                         |
| `label`        | `string`                  | Visible label, also used in the toggle's accessible name.        |
| `value`        | `string`                  | Current text (controlled).                                       |
| `onChange`     | `(value: string) => void` | Called with the new text on every change.                        |
| `autoComplete` | `string`                  | `current-password`, `new-password` or `off`.                     |
| `autoFocus`    | `boolean`                 | Optional, default `false`. Focuses the field on mount.           |
| `required`     | `boolean`                 | Optional, default `false`. Marks the field as required.          |
| `describedBy`  | `string`                  | Optional. Id of an element that describes it, such as a hint.    |

## Internal structure

One `useState` for `visible` (initially `false`). The input switches between
`type="password"` and `type="text"`. The toggle button has `aria-pressed` and an
accessible name such as `Show password` or `Hide backup master password`, built
from the lowercased label. The input turns off spell checking, autocapitalization
and autocorrection, so the typed text is not sent to or altered by the browser.

## Dependencies

- Internal: none.
- External: `react` (`useState`).

## Usage example

As in [`EntryForm`](EntryForm.tsx.md):

```tsx
<PasswordInput
  id="entry-password"
  label="Password"
  value={entry.password}
  onChange={(value) => update("password", value)}
  autoComplete="off"
/>
```

Checked by the component tests.

## Edge cases, invariants and error handling

- The field is hidden by default and each instance starts hidden again when it is
  mounted again.
- The visibility state is local. Two fields on the same screen are toggled
  independently.
- It does not validate the text. The rules live in
  [`passwordRules.ts`](../passwordRules.ts.md) and in the core.
- `autoComplete` is required, so each use states whether the browser may offer a
  saved password.

## Security

- The text sits in a React state and in the DOM as a plain string, which JavaScript
  cannot wipe. Parents should clear their state after use, as
  [`RestoreBackupForm`](RestoreBackupForm.tsx.md) does after a successful restore.
- Turning off spell checking and autocorrection keeps the text from being sent to
  a spelling service.
- Showing the password is a choice of the user for that field only.

## Related files and tests

- Related: [`EntryForm.tsx`](EntryForm.tsx.md),
  [`RestoreBackupForm.tsx`](RestoreBackupForm.tsx.md),
  [`../screens/SetupScreen.tsx`](../screens/SetupScreen.tsx.md),
  [`../screens/UnlockScreen.tsx`](../screens/UnlockScreen.tsx.md),
  [`../screens/SettingsTab.tsx`](../screens/SettingsTab.tsx.md).
- Tests: [`components.test.tsx`](../../../../src/components/components.test.tsx)
  covers hiding until shown and reporting changes.
