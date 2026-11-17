# src-tauri/src/error.rs

## Purpose

Defines `AppError`, the error type returned by every vault operation and command,
and how it is serialized for the interface.

## Place in the architecture

- The [`vault`](vault/mod.rs.md), [`storage`](storage/mod.rs.md),
  [`commands`](commands/mod.rs.md) and [`security`](security/mod.rs.md) layers all
  return `AppResult<T>`.
- [`crypto`](crypto/mod.rs.md) has its own `CryptoError`, which this file converts.
- A failed command reaches the interface as `{ "code": "...", "message": "..." }`.
  The `code` is stable and meant for programs. The `message` is meant for people.
  The interface reads it in `src/api/client.ts`.

## Public API

`pub enum AppError` (`Debug`, `Error`, `PartialEq`, `Eq`):

| Variant                       | `code`                | Message                                                         |
| ----------------------------- | --------------------- | --------------------------------------------------------------- |
| `WrongPasswordOrCorrupt`      | `wrong_password`      | the master password is incorrect or the vault is damaged        |
| `Locked`                      | `locked`              | the vault is locked                                             |
| `VaultExists`                 | `vault_exists`        | a vault already exists                                          |
| `VaultNotFound`               | `vault_not_found`     | no vault has been created yet                                   |
| `InvalidInput(String)`        | `invalid_input`       | the given text                                                  |
| `NotFound(&'static str)`      | `not_found`           | `<item> not found`, for example `entry not found`               |
| `Corrupt(String)`             | `corrupt`             | the vault data is damaged: `<detail>`                           |
| `UnsupportedVersion`          | `unsupported_version` | the vault was created by a newer version of lockwell            |
| `Crypto(String)`              | `crypto`              | cryptographic error: `<detail>`                                 |
| `Storage(String)`             | `storage`             | storage error: `<detail>`                                       |
| `Io(String)`                  | `io`                  | file system error: `<detail>`                                   |

Functions and traits:

- `AppError::code(&self) -> &'static str`: the stable code.
- `AppError::invalid(message: impl Into<String>) -> AppError`: shortcut for
  `InvalidInput`.
- `impl Serialize`: writes `{"code": ..., "message": ...}`.
- `impl From<CryptoError>`: `Decryption` and `Truncated` become `Corrupt`, `Io`
  stays `Io`, and every other error becomes `Crypto`.
- `impl From<std::io::Error>`: keeps only the error kind, as `Io`.
- `impl From<rusqlite::Error>`: becomes `Storage` with SQLite's message.
- `pub type AppResult<T> = Result<T, AppError>`.

## Internal structure

`AppError` derives its `Display` with `thiserror`. `code` is an exhaustive `match`,
so adding a variant without a code does not compile. `Serialize` is written by
hand, with two fields, so the shape does not depend on the variant.

## Dependencies

- Internal: [`crypto/mod.rs`](crypto/mod.rs.md) (`CryptoError`).
- External: `thiserror` (derives `Error`), `serde` (the `Serialize` implementation),
  `rusqlite` (the `From` conversion).

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::error::AppError;

serde_json::to_string(&AppError::Locked)?;
// {"code":"locked","message":"the vault is locked"}
serde_json::to_string(&AppError::Corrupt("bad header".into()))?;
// {"code":"corrupt","message":"the vault data is damaged: bad header"}

AppError::from(CryptoError::Decryption);   // Corrupt("decryption failed: wrong key or the data is corrupted")
AppError::from(CryptoError::Random);       // Crypto("the system random number generator is unavailable")
AppError::from(io_error_with_a_path);      // Io("entity not found"), the path is dropped
```

## Edge cases, invariants and error handling

- `WrongPasswordOrCorrupt` is one error for two causes, on purpose. Callers that
  know a decryption failure means a wrong password map it themselves.
- Every variant has a unique, stable code. The interface depends on these strings.
- `NotFound` carries a `&'static str`, so it can only name a fixed item such as
  `entry` or `file`, never user data.
- `From<CryptoError>` does not turn authentication failures into
  `WrongPasswordOrCorrupt`. That mapping is explicit in
  [`vault/header.rs`](vault/header.rs.md).

## Security

- Messages never contain passwords, keys or decrypted content. Variants carry
  only text that is safe to show: validation messages, error kinds and fixed
  descriptions.
- The I/O conversion keeps only the `ErrorKind`, because the original message may
  include file paths.
- `Storage` passes on SQLite's own message. The code relies on those messages
  describing the query and never the (encrypted) values. For example, a duplicate
  id gives `UNIQUE constraint failed: entries.id`.
- Because one wrong-password error also covers a damaged key header, an attacker
  who can see the error learns nothing about which case happened.

## Related files and tests

- Related: [`crypto/mod.rs`](crypto/mod.rs.md), [`state.rs`](state.rs.md),
  [`commands/mod.rs`](commands/mod.rs.md), [`vault/header.rs`](vault/header.rs.md),
  [`storage/db.rs`](storage/db.rs.md).
- Tests: inline `tests` module in
  [`error.rs`](../../../../src-tauri/src/error.rs): serialization shape, every
  code, messages, and the three conversions. The integration test
  `errors_never_contain_passwords_data_or_paths` in
  [`security.rs`](../../../../src-tauri/tests/security.rs) checks that errors do
  not leak secrets or paths.
