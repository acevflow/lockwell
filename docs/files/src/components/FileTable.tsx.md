# src/components/FileTable.tsx

## Purpose

Table of the files stored in the vault, with a "Save a copy" and a "Delete" button on
each row.

## Place in the architecture

A presentational component used by [`FilesTab`](../screens/FilesTab.tsx.md). The tab
owns the data and the operations. The table only lists the files and reports the
user's choice. It shows only the metadata the core exposes
([`FileSummary`](../api/types.ts.md)): name, size and date. Contents never reach the
interface.

## Public API

Default export `FileTable(props)`:

| Prop         | Type                          | Description                                                       |
| ------------ | ----------------------------- | ----------------------------------------------------------------- |
| `files`      | `FileSummary[]`               | Files to list, in display order.                                  |
| `disabled`   | `boolean`                     | Disables the row buttons while an operation runs.                 |
| `onSaveCopy` | `(file: FileSummary) => void` | Called when the user asks for a decrypted copy.                   |
| `onDelete`   | `(file: FileSummary) => void` | Called when the user asks to delete. The caller confirms.         |

Columns: Name, Size, Added and an Actions column hidden visually but available to
screen readers. Each button has an accessible name that includes the file name:
`Save a copy of <name>` and `Delete <name>`.

## Internal structure

A plain function component. The size column uses
[`formatBytes`](../format.ts.md) and the date uses `toLocaleDateString`. Rows are keyed
by file id and headers use `scope="col"`.

## Dependencies

- Internal: [`api/types.ts`](../api/types.ts.md) (`FileSummary`),
  [`format.ts`](../format.ts.md) (`formatBytes`).
- External: none.

## Usage example

As in [`FilesTab`](../screens/FilesTab.tsx.md):

```tsx
<FileTable
  files={files}
  disabled={busy !== null}
  onSaveCopy={saveCopy}
  onDelete={setToDelete}
/>
```

The table is exercised through the tests of its screen.

## Edge cases, invariants and error handling

- The component does not render an empty-state message. The tab decides what to show
  when there are no files.
- File names can be long. The `file-name` cell is styled by `src/styles.css`.
- Names are rendered as text, never as HTML.
- The date has no time of day, only the local date.

## Security

- Only the name, the size and the date are shown.
- The "Save a copy" action produces a plaintext file outside the vault. The caller is
  responsible for making that clear to the user.

## Related files and tests

- Related: [`../screens/FilesTab.tsx`](../screens/FilesTab.tsx.md),
  [`../format.ts`](../format.ts.md), [`../api/types.ts`](../api/types.ts.md),
  [`ConfirmDialog.tsx`](ConfirmDialog.tsx.md).
- Tests: no test file of its own. It is covered by
  [`FilesTab.test.tsx`](../../../../src/screens/FilesTab.test.tsx).
