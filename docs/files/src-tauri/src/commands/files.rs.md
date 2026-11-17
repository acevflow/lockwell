# src-tauri/src/commands/files.rs

Encrypted file commands. The native open and save dialogs
(`tauri-plugin-dialog`) are shown from the core, so the interface never
handles file system paths.

| Command        | Arguments | Result                                       |
| -------------- | --------- | -------------------------------------------- |
| `list_files`   | none      | `FileSummary[]`                              |
| `import_files` | none      | added `FileSummary[]` (empty if cancelled)   |
| `export_file`  | `id`      | `true` if saved, `false` if cancelled        |
| `delete_file`  | `id`      | none                                         |

Dialogs and encryption run on blocking threads (the dialog helpers live in
`dialog.rs`). The vault is not held while
a dialog is open. When saving, the native dialog has already asked before
replacing an existing file.

When adding several files fails for one of them, the error names it
(`import_error`). Tests cover that message.
