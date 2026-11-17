# src-tauri/src/commands/mod.rs

## Purpose

Root of the command layer: the Tauri commands are the only functions the
interface can call. This file declares the submodules.

## Place in the architecture

- Commands stay thin. They validate and convert their arguments and delegate to
  the vault ([`vault/`](../vault/mod.rs.md)) through
  [`AppState`](../state.rs.md).
- Most commands run through `AppState::with_vault` (or `with_vault_blocking`),
  which counts as user activity before and after the operation.
- A failure reaches the interface as `{ "code", "message" }` (see
  [`error.rs`](../error.rs.md) and `src/api/client.ts`).
- Native dialogs (open and save) are shown by the core, so the interface never
  handles file system paths.
- [`lib.rs`](../lib.rs.md) registers all 18 commands in `generate_handler!`. The
  capability file only lets the interface listen for events, so it cannot use
  dialogs or the clipboard directly.

| Submodule                       | Commands                                                                         |
| ------------------------------- | -------------------------------------------------------------------------------- |
| [`vault`](vault.rs.md)          | `vault_status`, `create_vault`, `unlock_vault`, `lock_vault`, `change_master_password` |
| [`entries`](entries.rs.md)      | `list_entries`, `get_entry`, `create_entry`, `update_entry`, `delete_entry`      |
| [`files`](files.rs.md)          | `list_files`, `import_files`, `export_file`, `delete_file`                       |
| [`backup`](backup.rs.md)        | `create_backup`, `restore_backup`                                                |
| [`security`](security.rs.md)    | `touch_activity`, `copy_entry_field`                                             |
| `dialog` (private, [doc](dialog.rs.md)) | Helpers shared by the commands that show dialogs.                        |

## Public API

`pub mod backup`, `pub mod entries`, `pub mod files`, `pub mod security` and
`pub mod vault`. `dialog` is private to the layer.

## Internal structure

Only the six module declarations.

## Dependencies

- Internal: the six submodules, and indirectly [`state.rs`](../state.rs.md) and
  [`vault/`](../vault/mod.rs.md).
- External: `tauri` (in the submodules).

## Usage example

The commands are invoked by the interface through Tauri, for example
`invoke("vault_status")`. The typed wrappers are in `src/api/`. Examples that
run the equivalent calls against the core are in each submodule document.

## Edge cases, invariants and error handling

- All commands except `vault_status`, `create_vault`, `unlock_vault`,
  `lock_vault`, `restore_backup` and `touch_activity` need an unlocked vault and
  fail with `locked` otherwise. `change_master_password` also needs it, and
  reports `locked` through the vault layer.
- `restore_backup` works on a locked vault or when none exists, because it
  replaces the vault folder.
- `lock_vault` and `touch_activity` return nothing and cannot fail.

## Security

- The interface only receives what a command returns. Passwords and notes are
  returned by `get_entry` only, and `copy_entry_field` never sends the value to
  the interface.
- Commands are the boundary where untrusted input enters, so identifiers,
  entry fields and passwords are validated by the vault layer behind them.

## Related files and tests

- Related: [`lib.rs`](../lib.rs.md), [`state.rs`](../state.rs.md),
  [`error.rs`](../error.rs.md), [`vault/mod.rs`](../vault/mod.rs.md).
- Tests: there is no test of this file. The commands are exercised through
  `AppState` in the integration tests in
  [`src-tauri/tests/`](../../../../../src-tauri/tests/), and the interface side
  is tested with a mocked `invoke` in `src/`.
