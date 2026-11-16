# src-tauri/src/crypto/keys.rs

`SecretKey`: a 256-bit symmetric key that is overwritten with zeros when it is
dropped (`zeroize`) and whose `Debug` output is redacted.

- `SecretKey::generate()` creates a random key from the operating system.
- `SecretKey::from_slice()` builds a key from exactly 32 bytes and rejects any
  other length.

Tests cover generation, length checks, the redacted debug output and wiping.
