# src-tauri/src/commands/dialog.rs

Helpers shared by the commands that show native dialogs (`files.rs` and
`backup.rs`). They are private to the `commands` module.

- `run_blocking` runs a dialog on a blocking thread, so it does not hold up
  the async runtime. If the thread fails, it returns an `Io` error.
- `to_path` converts the location chosen in a dialog to a local path. It
  returns an `Input` error if the location is not a local file.
