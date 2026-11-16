# src-tauri/src/vault/header.rs

The key header of the vault (`VaultHeader`), stored in the `meta` table.

Key hierarchy: the master password and a random salt go through Argon2id to
produce a wrapping key (KEK). The vault key (DEK), a random 256-bit key that
encrypts all records and files, is stored encrypted with the KEK using
XChaCha20-Poly1305. Changing the master password only re-wraps the DEK.

Binary layout (108 bytes, little-endian): magic `LWVH`, format version,
Argon2id memory, iterations and parallelism, 16-byte salt, and the wrapped key
(nonce, encrypted key and tag). The first 36 bytes are the associated data of
the wrapped key, so changing the parameters or the salt is detected.

- `create()` generates a DEK and wraps it; `wrap()` re-wraps an existing DEK
  with a new password and a fresh salt; `unwrap_key()` recovers it.
- `from_bytes()` validates the magic, version, length and the Argon2id
  parameter bounds before any key derivation runs.

Tests cover the round trip, wrong passwords, tampering with every parameter,
salt and key byte, invalid encodings, newer versions and out-of-range
parameters.
