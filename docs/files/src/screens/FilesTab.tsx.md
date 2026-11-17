# src/screens/FilesTab.tsx

The Files section: a table of stored files (name, size, date added) with
"Add files", "Save a copy" and "Delete" (after confirmation). While an
operation runs, the buttons are disabled and a status message says what is
happening; the result or error is shown when it ends. A `locked` error
returns to the unlock screen.

If adding several files fails part way, the files added before the failure
stay in the vault, so the list is reloaded before the error is shown.

The list is loaded when the tab opens with `useInitialLoad`.
