# src-tauri/src/crypto/aead.rs

Encrypts single records with XChaCha20-Poly1305.

- `seal(key, plaintext, aad)` returns `nonce (24 B) || ciphertext || tag (16 B)`
  with a fresh random nonce.
- `open(key, sealed, aad)` authenticates and decrypts. The plaintext is
  returned in a buffer that is wiped on drop.

The associated data (`aad`) is authenticated but not encrypted; callers use it
to bind each record to its context so records cannot be swapped.

Tests check the test vector from draft-irtf-cfrg-xchacha-03 (A.3.1) and the
failure cases: wrong key, wrong associated data, every tampered byte,
truncated input and appended bytes.
