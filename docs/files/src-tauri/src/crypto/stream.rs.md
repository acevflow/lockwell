# src-tauri/src/crypto/stream.rs

Encrypts data of any size in 64 KiB chunks with the STREAM construction
(`aead-stream`, big-endian 32-bit counter) over XChaCha20-Poly1305. Used for
files and backups.

Layout: `nonce prefix (19 B) || chunk 0 || ... || last chunk`, where every
chunk carries its own 16-byte tag. STREAM authenticates the position of every
chunk and marks the last one, so reordered, dropped, duplicated or truncated
chunks are detected.

- `encrypt(key, aad, reader, writer)` and `decrypt(...)` work on any
  `Read`/`Write` and return the number of plaintext bytes.
- `encrypted_len()` gives the encrypted size for a plaintext size.

Decryption writes plaintext as it goes, so callers write to a temporary
destination and discard it on error. Chunk buffers are wiped on drop and are
allocated with room for the tag so they never reallocate.

Tests cover sizes around the chunk boundary (including empty input), wrong
key and associated data, tampering, reordered, dropped and appended chunks,
truncation and I/O errors.
