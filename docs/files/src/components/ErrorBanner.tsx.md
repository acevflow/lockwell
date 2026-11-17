# src/components/ErrorBanner.tsx

## Purpose

Inline error message for forms and tabs, announced to screen readers, with an
optional Dismiss button.

## Place in the architecture

The common way the interface shows a failed operation. It is used by
[`PasswordsTab`](../screens/PasswordsTab.tsx.md),
[`FilesTab`](../screens/FilesTab.tsx.md),
[`SettingsTab`](../screens/SettingsTab.tsx.md),
[`SetupScreen`](../screens/SetupScreen.tsx.md),
[`UnlockScreen`](../screens/UnlockScreen.tsx.md),
[`EntryForm`](EntryForm.tsx.md), [`CreateBackupPanel`](CreateBackupPanel.tsx.md)
and [`RestoreBackupForm`](RestoreBackupForm.tsx.md). The text usually comes from
[`errorMessage`](../api/client.ts.md) or from [`useApiError`](../hooks/useApiError.ts.md).

## Public API

Default export `ErrorBanner({ message, onDismiss })`:

| Prop        | Type                 | Description                                                              |
| ----------- | -------------------- | ------------------------------------------------------------------------ |
| `message`   | `string \| null`     | Text to show. Nothing is rendered when it is `null` or empty.            |
| `onDismiss` | `() => void`         | Optional. Adds a Dismiss button that calls it when pressed.              |

It renders `div.error-banner` with `role="alert"` containing the message in a
`span`. The Dismiss button is a `button.link` with the accessible name
`Dismiss error`.

## Internal structure

A plain function component: an early `return null` when there is no message,
otherwise the banner. No state and no effects.

## Dependencies

- Internal: none. Styles are in `src/styles.css`.
- External: none.

## Usage example

As in [`PasswordsTab`](../screens/PasswordsTab.tsx.md):

```tsx
const { error, report, clear } = useApiError(onLocked);
// ...
<ErrorBanner message={error} onDismiss={clear} />
```

Checked by the component tests.

## Edge cases, invariants and error handling

- An empty string behaves like `null`: nothing is shown.
- Without `onDismiss` there is no button, which is used where the form clears the
  error itself on the next attempt.
- `role="alert"` makes assistive technology announce the message as soon as it
  appears.

## Security

The text is rendered as React text, never as HTML, so a message cannot inject
markup. Messages come from the core and contain no secrets.

## Related files and tests

- Related: [`../hooks/useApiError.ts`](../hooks/useApiError.ts.md),
  [`../api/client.ts`](../api/client.ts.md), and every component listed above.
- Tests: [`components.test.tsx`](../../../../src/components/components.test.tsx)
  covers rendering nothing without a message, the announcement and dismissing.
