# src-tauri/src/crypto/aead.rs

## Purpose

Encrypts and decrypts single, in-memory records with XChaCha20-Poly1305,
binding each record to its context through associated data.

## Place in the architecture

- Lowest-level building block for small records: entry fields, entry names and
  the wrapped vault key in the header.
- Large data (files, backups) does not go through this module. It uses
  [`stream.rs`](stream.rs.md), which reuses the cipher built here.
- It implements no primitive. The cipher comes from the `chacha20poly1305`
  crate.

## Public API

| Item                                                  | Description                                                                          |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `NONCE_LEN: usize = 24`                               | Nonce length in bytes.                                                               |
| `TAG_LEN: usize = 16`                                 | Poly1305 tag length in bytes.                                                        |
| `OVERHEAD: usize = 40`                                | Bytes a sealed record adds to its plaintext (`NONCE_LEN + TAG_LEN`).                 |
| `seal(key: &SecretKey, plaintext: &[u8], aad: &[u8])` | Returns `Result<Vec<u8>, CryptoError>` laid out as `nonce \|\| ciphertext \|\| tag`. |
| `open(key: &SecretKey, sealed: &[u8], aad: &[u8])`    | Returns `Result<Zeroizing<Vec<u8>>, CryptoError>` with the plaintext.                |

Errors:

- `seal`: `CryptoError::Random` if the nonce cannot be generated;
  `CryptoError::Encryption` if the cipher rejects the input.
- `open`: `CryptoError::Truncated` if `sealed` is shorter than `OVERHEAD`;
  `CryptoError::Decryption` for a wrong key, wrong `aad` or any modified byte.

Side effects: none besides reading the operating system random generator in
`seal`. The returned plaintext buffer is wiped when dropped.

The crate-visible function `cipher(key)` builds the `XChaCha20Poly1305` value
for a key. [`stream.rs`](stream.rs.md) uses it.

## Internal structure

- `seal` draws a fresh 24-byte nonce with [`random_array`](random.rs.md) and
  delegates to the private `seal_with_nonce`, which encrypts and prepends the
  nonce.
- `seal_with_nonce` exists so the test vector can fix the nonce. It is private
  because reusing a nonce under one key breaks confidentiality.
- `open` splits the input at `NONCE_LEN` and decrypts the rest, mapping every
  cipher failure to the single `Decryption` error.

## Dependencies

- Internal: [`mod.rs`](mod.rs.md) (`CryptoError`), [`keys.rs`](keys.rs.md)
  (`SecretKey`), [`random.rs`](random.rs.md) (`random_array`).
- External: `chacha20poly1305` (the audited cipher), `zeroize` (`Zeroizing`
  for the plaintext).

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::crypto::{aead, keys::SecretKey, CryptoError};

let key = SecretKey::generate()?;
let sealed = aead::seal(&key, b"hello", b"entry:42")?;
assert_eq!(sealed.len(), 5 + aead::OVERHEAD); // 45 bytes
let opened = aead::open(&key, &sealed, b"entry:42")?;
assert_eq!(&opened[..], b"hello");
// A different context does not authenticate:
assert_eq!(aead::open(&key, &sealed, b"entry:43").unwrap_err(), CryptoError::Decryption);
```

## Edge cases, invariants and error handling

- An empty plaintext is valid and yields exactly `OVERHEAD` bytes.
- Sealing the same data twice gives different output, because each call uses a
  new random nonce.
- Input shorter than `OVERHEAD` is `Truncated`. A cut that keeps at least
  `OVERHEAD` bytes fails authentication and is `Decryption`. So is appending
  bytes.
- Flipping any single byte (nonce, ciphertext or tag) is detected.
- Wrong key, wrong `aad` and tampering produce the same error on purpose.

## Security

- Callers must always pass an `aad` that names the record context (type, id,
  version), otherwise a valid record can be moved to another place.
- Random 192-bit nonces are safe for any realistic number of records under
  one key.
- The returned plaintext is in a `Zeroizing` buffer. Callers must not copy it
  into buffers that are not wiped.
- Errors never include key material or plaintext.

## Related files and tests

- Related: [`keys.rs`](keys.rs.md), [`random.rs`](random.rs.md),
  [`stream.rs`](stream.rs.md), [`mod.rs`](mod.rs.md). Used by
  [`vault/header.rs`](../vault/header.rs.md),
  [`vault/entries.rs`](../vault/entries.rs.md) and
  [`vault/files.rs`](../vault/files.rs.md).
- Tests: the inline `tests` module in
  [`aead.rs`](../../../../../src-tauri/src/crypto/aead.rs). It checks the
  draft-irtf-cfrg-xchacha-03 (A.3.1) vector, round trips, fresh nonces and the
  failure cases above.
