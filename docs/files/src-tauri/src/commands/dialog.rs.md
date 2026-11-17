# src-tauri/src/commands/dialog.rs

## Purpose

Helpers shared by the commands that show native file dialogs: running a blocking
dialog off the async runtime, and converting the chosen location to a local path.

## Place in the architecture

The module is private to [`commands`](mod.rs.md) (`mod dialog;`). It is used by
[`files.rs`](files.rs.md) and [`backup.rs`](backup.rs.md). A blocking dialog must
not run on the async runtime threads, and its result is a `FilePath` that may not
be a local path.

## Public API

Both functions are `pub(super)`, visible only inside the `commands` module.

| Item                                                                                  | Description                                                              |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `async run_blocking<T: Send + 'static>(dialog: impl FnOnce() -> T + Send + 'static) -> AppResult<T>` | Runs `dialog` on a blocking thread and returns its result. |
| `to_path(path: FilePath) -> AppResult<PathBuf>`                                       | Converts a dialog result to a local path.                                |

Errors:

- `run_blocking`: `AppError::Io("the file dialog failed")` if the blocking task
  fails to complete.
- `to_path`: `AppError::InvalidInput("the selected location is not a local file")`
  if the location is not a local path (for example a URL).

## Internal structure

`run_blocking` calls `tauri::async_runtime::spawn_blocking` and awaits it.
`to_path` calls `FilePath::into_path`.

## Dependencies

- Internal: [`error.rs`](../error.rs.md).
- External: `tauri` (the async runtime) and `tauri_plugin_dialog` (`FilePath`).

## Usage example

Usage as in [`files.rs`](files.rs.md):

```rust
let chosen = run_blocking(move || app.dialog().file().blocking_pick_files()).await?;
let Some(paths) = chosen else { return Ok(Vec::new()) };   // the user cancelled
let paths = paths.into_iter().map(to_path).collect::<AppResult<Vec<_>>>()?;
```

These helpers need a running Tauri application and a window, so they were not
executed on their own.

## Edge cases, invariants and error handling

- A cancelled dialog is not an error here. The dialog returns `None` and the
  command handles it.
- The error messages are fixed text and carry no path.

## Security

Keeping dialogs in the core means the interface never sees or sets file system
paths, so it cannot ask the core to read or write an arbitrary location.

## Related files and tests

- Related: [`files.rs`](files.rs.md), [`backup.rs`](backup.rs.md),
  [`mod.rs`](mod.rs.md).
- Tests: none of its own. It depends on the dialog plugin, which cannot run in
  an automated test. The dialog flows are covered in
  [`docs/manual-testing.md`](../../../../manual-testing.md).
