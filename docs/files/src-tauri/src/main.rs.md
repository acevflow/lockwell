# src-tauri/src/main.rs

## Purpose

Binary entry point of the desktop app. It starts the application by calling
`lockwell_lib::run()`.

## Place in the architecture

The crate has a library (`lockwell_lib`, see [`lib.rs`](lib.rs.md)) and a binary.
All logic lives in the library, so this file has none. Keeping it minimal lets
the library be tested without starting a window.

## Public API

None. `fn main()` is the program entry point and returns nothing.

## Internal structure

- `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` hides the
  console window in release builds on Windows. Debug builds keep the console, so
  messages written to standard error stay visible during development. The
  attribute has no effect on other platforms.
- `main` calls `lockwell_lib::run()`.

## Dependencies

- Internal: [`lib.rs`](lib.rs.md) (`run`).
- External: none directly.

## Usage example

Run the app in development with `npm run tauri dev`, or build it with
`npm run tauri build`. The binary was not started in this check, because it opens
a window. The code is the whole file:

```rust
fn main() {
    lockwell_lib::run()
}
```

## Edge cases, invariants and error handling

- If `run` panics, the process ends. In a release build there is no console, so
  the message is not shown to the user.
- The `windows_subsystem` attribute must stay. Removing it would show an extra
  console window behind the app in release builds on Windows.

## Security

Nothing sensitive is handled here. A release build without a console also means
that diagnostic messages written to standard error (for example, the one from
[`security/mod.rs`](security/mod.rs.md) when the lock event cannot be sent) are
not visible to the user.

## Related files and tests

- Related: [`lib.rs`](lib.rs.md), [`build.rs`](../build.rs.md).
- Tests: none. The file has no logic. See
  [`docs/manual-testing.md`](../../../manual-testing.md) for the start-up checks.
