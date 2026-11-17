# src/screens/FilesTab.tsx

## Purpose

Files tab: list, add, save a copy of, and delete encrypted files.

## Place in the architecture

One of the three tabs of [`VaultScreen`](VaultScreen.tsx.md). It uses the wrappers in
[`api/files.ts`](../api/files.ts.md), the table of
[`FileTable`](../components/FileTable.tsx.md) and a
[`ConfirmDialog`](../components/ConfirmDialog.tsx.md) before deleting. The core shows
the native dialogs, so this tab never handles file paths. Errors go through
[`useApiError`](../hooks/useApiError.ts.md), which sends a `locked` error to `onLocked`.

## Public API

Default export `FilesTab({ onLocked })`:

| Prop       | Type         | Description                                              |
| ---------- | ------------ | -------------------------------------------------------- |
| `onLocked` | `() => void` | Called when the core reports that the vault is locked.   |

## Internal structure

State: `files` (`null` while loading), `busy` (the label of the running operation, or
`null`), `status` (the last success message), `toDelete` and the error from
`useApiError`.

- `useInitialLoad(listFiles, setFiles, report)` loads the list on mount.
- `run(label, action)` runs one operation: it clears the error and the status, shows
  the label as the busy text, shows the message the action returns (or nothing for
  `null`), reports an error and always clears `busy`. Only one operation runs at a
  time, because the row buttons and Add are disabled while `busy` is set.
- `add` calls `importFiles`. If it fails, it reloads the list first and then rethrows,
  because the core adds files one by one and some may be stored even though a later
  one failed. A success reloads the list and says `Added “name”.` or `Added N files.`.
  A cancelled dialog (empty array) says nothing.
- `saveCopy(file)` calls `exportFile`. It says `Saved a copy of “name”.`, or nothing if
  the user cancelled.
- `remove(file)` closes the dialog, calls `deleteFile`, reloads and says `Deleted
  “name”.`

The view shows `Loading…`, an empty-state line, or the `FileTable`.

## Dependencies

- Internal: [`api/files.ts`](../api/files.ts.md), [`api/types.ts`](../api/types.ts.md),
  [`components/ConfirmDialog.tsx`](../components/ConfirmDialog.tsx.md),
  [`components/ErrorBanner.tsx`](../components/ErrorBanner.tsx.md),
  [`components/FileTable.tsx`](../components/FileTable.tsx.md),
  [`hooks/useApiError.ts`](../hooks/useApiError.ts.md),
  [`hooks/useInitialLoad.ts`](../hooks/useInitialLoad.ts.md).
- External: `react` (`useCallback`, `useState`).

## Usage example

As in [`VaultScreen`](VaultScreen.tsx.md). Checked by the screen tests:

```tsx
{tab === "files" && <FilesTab onLocked={onLocked} />}
```

## Edge cases, invariants and error handling

- **Observation, not fixed (from reading the code):** if the first load of the list
  fails, the error banner appears but `files` stays `null`, so the body keeps showing
  `Loading…` below the error. The tests only check that the alert appears. Adding a file
  reloads the list and replaces the placeholder.
- A list that fails after the user left the tab is ignored (the hook drops late
  results).
- A failed import still refreshes the list, so the screen shows what is really in the
  vault.
- Cancelling the delete confirmation or a failed deletion keeps the file in the list.
- The status line is hidden while an operation runs and replaced by the busy text.
- Large files can take a while. The tab shows `Encrypting files…` or
  `Decrypting “name”…` and disables the actions meanwhile.

## Security

- The tab sees only names, sizes and dates. File content never reaches the interface.
- A saved copy is plaintext on disk, outside the vault. The hint above the table says
  so before the user does it.
- Deleting asks for confirmation and removes both the encrypted data and the entry in
  the core.

## Related files and tests

- Related: [`VaultScreen.tsx`](VaultScreen.tsx.md),
  [`../components/FileTable.tsx`](../components/FileTable.tsx.md),
  [`../api/files.ts`](../api/files.ts.md),
  [`../hooks/useApiError.ts`](../hooks/useApiError.ts.md),
  [`../../src-tauri/src/commands/files.rs`](../../src-tauri/src/commands/files.rs.md).
- Tests: [`FilesTab.test.tsx`](../../../../src/screens/FilesTab.test.tsx) covers
  listing, the empty and loading states, adding (one, several, cancelled and failing
  part way), saving a copy, deleting and cancelling, errors, the locked vault and the
  busy state.
