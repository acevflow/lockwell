# src-tauri/src/crypto/kdf.rs

Derives a 256-bit key from the master password with Argon2id v1.3.

- `KdfParams` holds the memory cost (KiB), iterations and parallelism.
  `KdfParams::DEFAULT` (64 MiB, 3 iterations, 4 lanes) is used for new vaults.
- `KdfParams::validate()` rejects parameters outside fixed bounds (up to 1 GiB,
  16 iterations, 16 lanes). The parameters are stored with the vault, so a
  tampered vault cannot make the app allocate unbounded memory.
- `derive_key()` takes a 16-byte salt (`SALT_LEN`).

Tests check the Argon2id reference test vector (phc-winner-argon2), that
password, salt and parameters all change the key, and that out-of-range
parameters and short salts are rejected while the inclusive bounds are
accepted.

The derived key is a wrapping key for the vault's own random key. Derivation
is slow on purpose, so callers run it in a blocking context.
