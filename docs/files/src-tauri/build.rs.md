# src-tauri/build.rs

## Purpose

Cargo build script. It runs `tauri_build::build()`, which prepares the Tauri
configuration, capabilities and icons that `tauri::generate_context!` embeds in
the application.

## Place in the architecture

Cargo runs this file before compiling the crate. It reads
[`tauri.conf.json`](../../../src-tauri/tauri.conf.json), the capability files under
`capabilities/` and the icons under `icons/`, validates them and generates the
data that [`lib.rs`](src/lib.rs.md) includes with `generate_context!()`. On
Windows it also embeds the icon and the version information in the executable.

## Public API

None. `fn main()` is the entry point of the build script and returns nothing. If
`tauri_build::build()` fails, the build fails.

## Internal structure

The whole file is one call:

```rust
fn main() {
    tauri_build::build()
}
```

## Dependencies

- Internal: [`lib.rs`](src/lib.rs.md) uses its output.
- External: `tauri-build` (a build dependency in `Cargo.toml`, version 2).

## Usage example

The script runs on every `cargo build`, `cargo test` and `npm run tauri build`
(`cargo test` was run in this check and builds it). There is nothing to call by
hand.

## Edge cases, invariants and error handling

- A malformed `tauri.conf.json`, a capability that refers to an unknown
  permission or a missing icon makes the build fail, with a message from
  `tauri-build`.
- The build depends on the files named above, so changing them triggers a
  rebuild.

## Security

The script does not handle secrets and makes no network connection. The
capability files it validates are part of the security configuration: the
interface can only listen for events (`core:event:allow-listen` and
`core:event:allow-unlisten`).

## Related files and tests

- Related: [`src/lib.rs`](src/lib.rs.md), [`src/main.rs`](src/main.rs.md),
  [`Cargo.toml`](../../../src-tauri/Cargo.toml).
- Tests: none. It is exercised by every build. See
  [`docs/development.md`](../../development.md) for the build commands.
