# scripts/third-party-licenses.mjs

## Purpose

Generates [`THIRD-PARTY-LICENSES.md`](../../../THIRD-PARTY-LICENSES.md), the list of
third-party software in a Linux build and the license each piece declares. It also
checks that the committed file is up to date.

## Place in the architecture

A maintenance script that runs on the developer's machine with Node.js. It is not part
of the app and is not shipped. It asks `cargo` for the dependency tree and metadata of
the Linux target and renders them, with a fixed section about the system libraries
(WebKitGTK and GTK), into Markdown.

## Public API

Run from the repository root:

| Command                  | Effect                                                             |
| ------------------------ | ------------------------------------------------------------------ |
| `npm run licenses`       | Writes `THIRD-PARTY-LICENSES.md`.                                  |
| `npm run licenses:check` | Exits with code 1 if the file differs from what would be written.  |

Exported for the tests:

- `TARGET`: the Rust target that is listed (`x86_64-unknown-linux-gnu`).
- `parseTree(text)`: the `name version` pairs in the output of `cargo tree --prefix none`.
- `selectCrates(metadata, wanted, rootName)`: crates of `cargo metadata` that are
  wanted, without the root crate, sorted by name and version.
- `licenseOf(crate)` and `sourceOf(crate)`: the declared license (`A/B` written as
  `A OR B`, `see source` when missing) and the best link to the source.
- `render(crates)`: the whole document.
- `generate()`: runs `cargo` and returns the document.

## Internal structure

1. `cargo tree --target <TARGET> -e normal --prefix none --locked` gives the runtime
   dependencies (build-time and test-only crates are left out).
2. `cargo metadata --filter-platform <TARGET> --locked` gives each crate's license and
   repository.
3. `selectCrates` joins the two. `cargo tree` can still print a few crates of other
   platforms that have no metadata for this target; they are ignored.
4. `render` writes the document. The output has no dates or other changing data, so the
   same `Cargo.lock` always gives the same file, which is what makes `--check` work.

## Dependencies

- Internal: none. Its output is the root file `THIRD-PARTY-LICENSES.md`.
- External: Node.js built-in modules only (`node:child_process`, `node:fs`,
  `node:url`, `node:process`) and the `cargo` command. No npm dependency.

## Usage example

```text
$ npm run licenses:check
THIRD-PARTY-LICENSES.md is up to date.
```

This was run when the file was written. After `Cargo.lock` changes, `npm run licenses`
updates the file; review the diff before committing.

## Edge cases, invariants and error handling

- `cargo` is run with `--locked`, so the script never changes `Cargo.lock` and fails if
  it is out of sync with `Cargo.toml`. If `cargo` is missing or fails, the script stops
  with its error.
- A crate with no declared license is listed as `see source` instead of being hidden;
  review those by hand.
- Only the license each crate declares is listed. The license texts and copyright
  notices are in the crates' sources, and the version and license of GTK and WebKitGTK
  must be taken from the packages that are actually bundled.
- The document covers the Linux target only. Windows and macOS builds use other
  dependencies.

## Security

It reads local files and runs `cargo` on the local checkout; it makes no network
request, reads no secrets and writes only `THIRD-PARTY-LICENSES.md`.

## Related files and tests

- Related: [`THIRD-PARTY-LICENSES.md`](../../../THIRD-PARTY-LICENSES.md),
  [`src-tauri/Cargo.toml`](../../../src-tauri/Cargo.toml),
  [`docs/development.md`](../../development.md).
- Tests: [`scripts/third-party-licenses.test.mjs`](../../../scripts/third-party-licenses.test.mjs)
  (parsing, selection and sorting, license and source fallbacks, rendering).
