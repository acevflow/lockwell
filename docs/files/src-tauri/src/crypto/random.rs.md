# src-tauri/src/crypto/random.rs

## Purpose

Single source of randomness in the crypto layer: random bytes, fixed-size
arrays and 128-bit identifiers from the operating system generator, plus the
helpers to encode and validate identifiers.

## Place in the architecture

Keys ([`keys.rs`](keys.rs.md)), nonces ([`aead.rs`](aead.rs.md),
[`stream.rs`](stream.rs.md)), salts and record identifiers all come from here.
There is no fallback generator: if the system one fails, the operation fails
with `CryptoError::Random`.

## Public API

| Item                                                        | Description                                |
| ----------------------------------------------------------- | ------------------------------------------ |
| `ID_LEN: usize = 16`                                        | Identifier length in bytes.                |
| `fill_random(buffer: &mut [u8]) -> Result<(), CryptoError>` | Fills the buffer. An empty buffer is fine. |
| `random_array::<N>() -> Result<[u8; N], CryptoError>`       | Returns `N` random bytes.                  |
| `random_id() -> Result<String, CryptoError>`                | 32 lowercase hexadecimal characters.       |
| `is_valid_id(id: &str) -> bool`                             | `true` only for exactly 32 chars `0-9a-f`. |
| `to_hex(bytes: &[u8]) -> String`                            | Lowercase hexadecimal encoding.            |

The three random functions return `CryptoError::Random` if the operating
system generator fails.

## Internal structure

`fill_random` calls `getrandom::fill`. `random_array` and `random_id` build on
it. `to_hex` uses a lookup table of hex digits.

## Dependencies

- Internal: [`mod.rs`](mod.rs.md) (`CryptoError`).
- External: `getrandom` (operating system random source).

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::crypto::random::{random_id, is_valid_id, to_hex};

let id = random_id()?;                  // e.g. "f2714f8de3bba40783f2edf46067e06d"
assert!(is_valid_id(&id));
assert!(!is_valid_id("../etc"));
assert_eq!(to_hex(&[0, 15, 165, 255]), "000fa5ff");
```

## Edge cases, invariants and error handling

- `is_valid_id` rejects the empty string, wrong lengths (31 or 33 characters),
  uppercase hexadecimal, non-hexadecimal letters and path traversal strings.
- Two identifiers are different for every realistic purpose (128 random bits).

## Security

- Identifiers name records and files on disk, so they reveal nothing about the
  content.
- Identifiers end up in file paths. An identifier that comes from outside the
  process must pass `is_valid_id` before use. The accepted shape contains no
  path separators or `..` segments.
- `is_valid_id` checks the shape only. It does not check that the record
  exists.

## Related files and tests

- Related: [`keys.rs`](keys.rs.md), [`aead.rs`](aead.rs.md),
  [`stream.rs`](stream.rs.md), [`vault/mod.rs`](../vault/mod.rs.md),
  [`vault/entries.rs`](../vault/entries.rs.md),
  [`vault/backup.rs`](../vault/backup.rs.md).
- Tests: inline `tests` module in
  [`random.rs`](../../../../../src-tauri/src/crypto/random.rs).
