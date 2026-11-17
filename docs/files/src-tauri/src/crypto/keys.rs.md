# src-tauri/src/crypto/keys.rs

## Purpose

Defines `SecretKey`, the 256-bit symmetric key type that is wiped from memory
when dropped and never printed.

## Place in the architecture

Every symmetric key in the crypto layer is a `SecretKey`, so wiping and
redaction are enforced in one place. [`aead.rs`](aead.rs.md),
[`kdf.rs`](kdf.rs.md), [`stream.rs`](stream.rs.md) and the vault code take or
return it.

## Public API

| Item                                                        | Description                                                |
| ----------------------------------------------------------- | ---------------------------------------------------------- |
| `KEY_LEN: usize = 32`                                       | Key length in bytes.                                       |
| `struct SecretKey([u8; 32])`                                | Derives `Clone`, `Zeroize` and `ZeroizeOnDrop`.           |
| `SecretKey::generate() -> Result<Self, CryptoError>`        | Random key from the operating system. `Random` on failure. |
| `SecretKey::from_slice(&[u8]) -> Result<Self, CryptoError>` | Copies exactly 32 bytes. `InvalidKeyLength` otherwise.     |
| `SecretKey::as_bytes(&self) -> &[u8; 32]`                   | Read access to the bytes.                                  |
| `impl Debug`                                                | Prints `SecretKey(<redacted>)`.                            |

Crate-visible only: `SecretKey::zeroed()` and `as_mut_bytes()`, used by
[`kdf.rs`](kdf.rs.md) to write the Argon2 output directly into the key.

## Internal structure

A tuple struct over `[u8; 32]`. `ZeroizeOnDrop` overwrites the bytes on drop.
Constructors build a zeroed key and then fill it, so there is no intermediate
unwiped copy owned by this module.

## Dependencies

- Internal: [`mod.rs`](mod.rs.md) (`CryptoError`),
  [`random.rs`](random.rs.md) (`fill_random`).
- External: `zeroize` (wipe on drop).

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::crypto::keys::SecretKey;

let key = SecretKey::generate()?;
println!("{key:?}"); // SecretKey(<redacted>)
assert!(SecretKey::from_slice(&[0u8; 31]).is_err()); // InvalidKeyLength
```

## Edge cases, invariants and error handling

- Only 32-byte slices are accepted. 31 and 33 bytes are rejected, and so is an
  empty slice.
- Two generated keys differ.
- `Clone` is derived. Each clone is wiped on its own drop, but every clone is
  one more copy in memory, so clone only when a second owner is needed.
- The type does not implement `PartialEq`, so keys cannot be compared with a
  non-constant-time operator by accident. Tests compare `as_bytes()`.

## Security

- `from_slice` copies the bytes. The caller must wipe its own copy.
- `as_bytes` exposes the raw key. Callers must not copy it into buffers that
  are not wiped.
- `Debug` never shows the bytes, so keys cannot leak through logs or error
  formatting.

## Related files and tests

- Related: [`aead.rs`](aead.rs.md), [`kdf.rs`](kdf.rs.md),
  [`random.rs`](random.rs.md), [`stream.rs`](stream.rs.md),
  [`vault/session.rs`](../vault/session.rs.md).
- Tests: inline `tests` module in
  [`keys.rs`](../../../../../src-tauri/src/crypto/keys.rs): uniqueness, length
  checks, redacted `Debug` output and explicit zeroizing.
