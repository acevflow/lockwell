# src-tauri/src/vault/mod.rs

## Purpose

Root of the vault layer. It declares the submodules, re-exports the session
types and holds three small helpers shared by the submodules.

## Place in the architecture

The vault layer sits between the Tauri commands ([`commands/`](../commands/mod.rs.md))
and the [`crypto`](../crypto/mod.rs.md) and [`storage`](../storage/mod.rs.md)
modules. It decides what is encrypted, with which associated data, and where it
is stored.

| Submodule                                | Role                                                              |
| ---------------------------------------- | ----------------------------------------------------------------- |
| [`header`](header.rs.md)                 | Key header: Argon2id parameters, salt and the wrapped vault key.  |
| [`password_policy`](password_policy.rs.md) | Rules for the master password.                                  |
| [`session`](session.rs.md)               | `Vault`, the locked and unlocked session, password change.        |
| [`entries`](entries.rs.md)               | Password entries, as methods of `UnlockedVault`.                  |
| [`files`](files.rs.md)                   | Encrypted files, as methods of `UnlockedVault`.                   |
| [`backup`](backup.rs.md)                 | Encrypted backup and restore.                                     |

## Public API

- Re-exports: `Vault` and `VaultStatus` from [`session`](session.rs.md).
- Crate-private helpers (not part of the public API, used by the submodules):

| Helper                                          | Description                                                                                         |
| ----------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `random_suffix() -> AppResult<String>`          | 16 random hexadecimal digits, to name temporary files and folders. Fails with the random error.     |
| `decrypt_failure(error: CryptoError, damaged: &str) -> AppError` | An I/O error stays `AppError::Io` with its message. Any other error becomes `AppError::Corrupt(damaged)`. |
| `now_millis() -> i64`                           | Milliseconds since the Unix epoch. Returns 0 for a clock before 1970 and `i64::MAX` on overflow.    |

## Internal structure

Only the three helpers above and the module declarations. The helpers exist
here so that [`files.rs`](files.rs.md), [`entries.rs`](entries.rs.md) and
[`backup.rs`](backup.rs.md) do not each define their own copy.

## Dependencies

- Internal: all the submodules listed above, [`crypto/random.rs`](../crypto/random.rs.md),
  [`crypto/mod.rs`](../crypto/mod.rs.md), [`error.rs`](../error.rs.md).
- External: none beyond the standard library.

## Usage example

The helpers are crate-private, so they cannot be called from outside the
crate. Their behaviour is covered by the inline tests, which were run with the
test suite. The public entry point is [`Vault`](session.rs.md#usage-example).

## Edge cases, invariants and error handling

- `decrypt_failure` collapses authentication failures and truncation into one
  `Corrupt` message, so callers do not reveal why decryption failed. File
  system errors are kept as I/O errors, because they are not damage.
- Temporary names are random, so concurrent operations in one folder do not
  collide.

## Security

`random_suffix` and `decrypt_failure` handle no secrets. The messages passed as
`damaged` must be fixed text that never includes data from the vault.

## Related files and tests

- Related: [`session.rs`](session.rs.md), [`files.rs`](files.rs.md),
  [`backup.rs`](backup.rs.md), [`lib.rs`](../lib.rs.md).
- Tests: inline `tests` module in
  [`mod.rs`](../../../../../src-tauri/src/vault/mod.rs): suffix length and
  uniqueness, and the `decrypt_failure` mapping.
