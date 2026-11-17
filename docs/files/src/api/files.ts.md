# src/api/files.ts

## Purpose

Typed wrappers for the encrypted file commands: list, import, export and delete.

## Place in the architecture

One function per command in
[`commands/files.rs`](../../src-tauri/src/commands/files.rs.md), through
[`call`](client.ts.md). The core shows the native open and save dialogs, so the
interface never sees or sends a file system path. [`FilesTab`](../screens/FilesTab.tsx.md)
uses them.

## Public API

| Function                                      | Command        | Resolves to / errors                                                              |
| --------------------------------------------- | -------------- | --------------------------------------------------------------------------------- |
| `listFiles(): Promise<FileSummary[]>`         | `list_files`   | Name, size and date of each file. `locked`, `corrupt`.                            |
| `importFiles(): Promise<FileSummary[]>`       | `import_files` | The added files. An empty array means the user cancelled.                          |
| `exportFile(id: string): Promise<boolean>`    | `export_file`  | `true` if saved, `false` if cancelled. `invalid_input`, `not_found`, `corrupt`.   |
| `deleteFile(id: string): Promise<void>`       | `delete_file`  | Deletes the file and its encrypted data. `invalid_input`, `not_found`.            |

All of them reject with an [`ApiError`](client.ts.md).

## Internal structure

No logic: each function is one `call`. `importFiles` and `exportFile` take no
path. The core asks the user.

## Dependencies

- Internal: [`client.ts`](client.ts.md), [`types.ts`](types.ts.md) (`FileSummary`).
- External: none directly.

## Usage example

Checked by the wrapper tests with the bridge mocked:

```ts
import { importFiles, exportFile } from "./files";

const added = await importFiles();      // [] if the user closed the dialog
const saved = await exportFile(added[0].id); // false if cancelled
```

## Edge cases, invariants and error handling

- Cancelling a dialog is a normal result (`[]` or `false`), not an error.
- A multi-file import can fail part way. The core keeps the files added before
  the failure and rejects with an error that names the file, instead of resolving
  to the list. `FilesTab` reloads the list after a failed import for that reason.
- The error text for a rejected import has the form
  `could not add “name”: reason`.

## Security

- The interface never handles paths, so it cannot ask for a read or a write at an
  arbitrary location.
- An exported file is plaintext on disk and leaves the protection of the vault.
- Only metadata (name, size, date) reaches the interface. File contents do not.

## Related files and tests

- Related: [`client.ts`](client.ts.md), [`types.ts`](types.ts.md),
  [`../screens/FilesTab.tsx`](../screens/FilesTab.tsx.md),
  [`../components/FileTable.tsx`](../components/FileTable.tsx.md),
  [`../../src-tauri/src/commands/files.rs`](../../src-tauri/src/commands/files.rs.md).
- Tests: [`api.test.ts`](../../../../src/api/api.test.ts) checks each command name
  and arguments.
