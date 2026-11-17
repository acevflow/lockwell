# src-tauri/src/crypto/kdf.rs

## Purpose

Derives a 256-bit wrapping key from the master password with Argon2id, after
validating the cost parameters.

## Place in the architecture

- The derived key never encrypts user data directly. The vault header uses it
  only to encrypt the vault's own random key
  ([`vault/header.rs`](../vault/header.rs.md)).
- The salt and the cost parameters are not secret and are stored with the
  vault. Because a stored value controls how much work and memory the app
  spends, the parameters are validated before every derivation.

## Public API

| Item                                                               | Description                                                       |
| ------------------------------------------------------------------ | ----------------------------------------------------------------- |
| `SALT_LEN: usize = 16`                                             | Salt length in bytes.                                             |
| `MAX_MEMORY_KIB: u32 = 1_048_576`                                  | Largest memory cost (1 GiB).                                      |
| `MAX_ITERATIONS: u32 = 16`                                         | Largest iteration count.                                          |
| `MAX_PARALLELISM: u32 = 16`                                        | Largest number of lanes.                                          |
| `struct KdfParams { memory_kib, iterations, parallelism }`         | Cost parameters (`Debug, Clone, Copy, PartialEq, Eq`).            |
| `KdfParams::DEFAULT`                                               | 64 MiB (`65536` KiB), 3 iterations, 4 lanes. Used for new vaults. |
| `KdfParams::validate(&self) -> Result<(), CryptoError>`            | Checks the bounds.                                                |
| `derive_key(password: &[u8], salt: &[u8; 16], params: &KdfParams)` | Returns `Result<SecretKey, CryptoError>`.                         |

Errors:

- `CryptoError::InvalidKdfParams(reason)` from `validate` and `derive_key`.
  The reason is one of `"parallelism out of range"`,
  `"iterations out of range"`, `"memory cost out of range"` or
  `"rejected by Argon2"`.
- `CryptoError::KeyDerivation` if Argon2id fails, for example when the memory
  cannot be allocated.

Side effects: allocates `memory_kib` KiB for the duration of the call and
takes noticeable CPU time.

## Internal structure

`derive_key` calls the private `derive_key_with_salt`, which validates the
parameters, builds Argon2id version 1.3 with a 32-byte output and writes the
result straight into a zeroed `SecretKey`. The variant that accepts any salt
length is private so the vault always uses 16-byte salts. The reference test
vector uses an 8-byte salt.

Validation rules:

- `parallelism` between 1 and 16.
- `iterations` between 1 and 16.
- `memory_kib` at least `8 * parallelism` (Argon2 needs 8 KiB per lane) and at
  most `MAX_MEMORY_KIB`.

## Dependencies

- Internal: [`mod.rs`](mod.rs.md) (`CryptoError`), [`keys.rs`](keys.rs.md)
  (`SecretKey`, `KEY_LEN`).
- External: `argon2` (the audited Argon2id implementation).

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::crypto::{kdf::{self, KdfParams}, random};

let salt = random::random_array::<{ kdf::SALT_LEN }>()?;
let params = KdfParams { memory_kib: 64, iterations: 1, parallelism: 1 };
let a = kdf::derive_key(b"correct horse battery", &salt, &params)?;
let b = kdf::derive_key(b"correct horse battery", &salt, &params)?;
assert_eq!(a.as_bytes(), b.as_bytes()); // deterministic

let bad = KdfParams { memory_kib: 0, ..KdfParams::DEFAULT };
// bad.validate() == Err(InvalidKdfParams("memory cost out of range"))
```

The small parameters above are for demonstration. New vaults use
`KdfParams::DEFAULT`.

## Edge cases, invariants and error handling

- The same password, salt and parameters always give the same key. Changing
  any of them gives a different key.
- Salts shorter than 8 bytes fail in Argon2 with `KeyDerivation`. This is only
  reachable through the private function, since the public one takes 16 bytes.
- An empty password is accepted here. The password policy is enforced by
  [`vault/password_policy.rs`](../vault/password_policy.rs.md), not by the KDF.
- The bounds are inclusive. For example, 16 lanes with 128 KiB, or 16
  iterations, are accepted.

## Security

- The call is slow and memory-hungry by design. Call it from a blocking
  context, never from an async task.
- The salt must be random and unique per vault.
- The caller owns the `password` bytes and must wipe them. The returned key is
  wiped on drop.
- Bounding the parameters prevents a tampered vault from forcing unbounded
  memory or run time.

## Related files and tests

- Related: [`keys.rs`](keys.rs.md), [`mod.rs`](mod.rs.md),
  [`vault/header.rs`](../vault/header.rs.md),
  [`vault/session.rs`](../vault/session.rs.md).
- Tests: the inline `tests` module in
  [`kdf.rs`](../../../../../src-tauri/src/crypto/kdf.rs). It checks the Argon2
  reference vector (Argon2id v1.3, t=2, m=2^16, p=1, password `password`, salt
  `somesalt`), determinism, parameter sensitivity, every bound and the error
  reasons.
