# src-tauri/src/crypto/mod.rs

## Purpose

Root of the cryptographic core. Declares the submodules and the
`CryptoError` type shared by all of them.

## Place in the architecture

The crypto layer is a thin, tested wrapper around audited RustCrypto crates.
No primitive is implemented here. Code outside the layer obtains keys and
ciphertext only through these modules:

| Module                   | Role                                                    |
| ------------------------ | ------------------------------------------------------- |
| [`kdf`](kdf.rs.md)       | Argon2id key derivation from the master password.      |
| [`aead`](aead.rs.md)     | XChaCha20-Poly1305 for single records.                  |
| [`stream`](stream.rs.md) | STREAM construction for files and backups of any size.  |
| [`keys`](keys.rs.md)     | `SecretKey`, wiped from memory on drop.                 |
| [`random`](random.rs.md) | Random bytes and identifiers from the operating system. |

The application error type in [`error.rs`](../error.rs.md) wraps `CryptoError`.

## Public API

`pub enum CryptoError` (`Debug`, `Error`, `PartialEq`, `Eq`):

| Variant                          | Message                                               | Meaning                                                 |
| -------------------------------- | ----------------------------------------------------- | ------------------------------------------------------- |
| `Random`                         | the system random number generator is unavailable     | The operating system generator failed.                  |
| `InvalidKdfParams(&'static str)` | invalid key derivation parameters: {reason}           | Parameters outside the accepted range.                  |
| `KeyDerivation`                  | key derivation failed                                 | Argon2id failed.                                        |
| `InvalidKeyLength`               | invalid key length                                    | A key was not 32 bytes.                                 |
| `Encryption`                     | encryption failed                                     | For example, input too long.                            |
| `Decryption`                     | decryption failed: wrong key or the data is corrupted | Authentication failed: wrong key, context or tampering. |
| `Truncated`                      | the encrypted data is truncated                       | Shorter than the format requires.                       |
| `Io(String)`                     | input/output error: {kind}                            | Reading or writing a stream failed.                     |

`impl From<std::io::Error> for CryptoError` keeps only the `ErrorKind` text.

## Internal structure

The file declares `pub mod` for the five submodules and the error enum, built
with `thiserror`.

## Dependencies

- Internal: the five submodules listed above.
- External: `thiserror` (derives `Display` and `Error`).

## Usage example

Mirrors the inline test and was checked by running the test suite:

```rust
use lockwell_lib::crypto::CryptoError;

let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "C:\\private\\file.txt");
let mapped = CryptoError::from(error);
assert!(!mapped.to_string().contains("private")); // the path is dropped
```

## Edge cases, invariants and error handling

- A wrong key, a wrong context and tampering all give `Decryption`, so the
  error does not help tell them apart.
- `Truncated` is only returned when the data is shorter than the minimum
  format size. Other cuts fail authentication and give `Decryption`.
- `Io` carries only the `ErrorKind` text, never the original message.

## Security

Messages never include key material, passwords or plaintext. Original I/O
messages are discarded because they may contain file paths.

## Related files and tests

- Related: [`error.rs`](../error.rs.md), [`lib.rs`](../lib.rs.md).
- Tests: inline `tests` module in
  [`mod.rs`](../../../../../src-tauri/src/crypto/mod.rs): the I/O error
  mapping and the stable message text.
