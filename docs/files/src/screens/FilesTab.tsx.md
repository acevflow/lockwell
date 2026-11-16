# src/screens/FilesTab.tsx

The Files section: a table of stored files (name, size, date added) with
"Add files", "Save a copy" and "Delete" (after confirmation). While an
operation runs, the buttons are disabled and a status message says what is
happening; the result or error is shown when it ends. A `locked` error
returns to the unlock screen.
