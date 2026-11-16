# src-tauri/src/crypto/mod.rs

Entry point of the cryptographic core. Declares the submodules (`aead`, `kdf`,
`keys`, `random`, `stream`) and defines `CryptoError`, the error type shared by
all of them. Error messages never contain keys, passwords or plaintext.

No cryptographic primitive is implemented here: every function wraps an
audited RustCrypto crate (`chacha20poly1305`, `aead-stream`, `argon2`).

Tests check the error messages and that I/O errors keep only their kind, never
a path.
