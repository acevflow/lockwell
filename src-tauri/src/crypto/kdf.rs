//! Argon2id key derivation from the master password.

use argon2::{Algorithm, Argon2, Params, Version};

use super::CryptoError;
use super::keys::{KEY_LEN, SecretKey};

/// Length in bytes of the Argon2id salt.
pub const SALT_LEN: usize = 16;

/// Largest accepted memory cost, in KiB (1 GiB).
pub const MAX_MEMORY_KIB: u32 = 1024 * 1024;
/// Largest accepted number of iterations.
pub const MAX_ITERATIONS: u32 = 16;
/// Largest accepted degree of parallelism.
pub const MAX_PARALLELISM: u32 = 16;

/// Argon2id cost parameters.
///
/// The parameters are stored next to the vault, so they are validated before
/// use: a tampered vault must not be able to make the app allocate unbounded
/// memory or run for hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    /// Memory cost in KiB.
    pub memory_kib: u32,
    /// Number of passes over the memory.
    pub iterations: u32,
    /// Number of parallel lanes.
    pub parallelism: u32,
}

impl KdfParams {
    /// Parameters used for new vaults: 64 MiB, 3 iterations, 4 lanes.
    pub const DEFAULT: KdfParams = KdfParams {
        memory_kib: 64 * 1024,
        iterations: 3,
        parallelism: 4,
    };

    /// Checks that the parameters are within the accepted bounds.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKdfParams`] describing the first problem.
    pub fn validate(&self) -> Result<(), CryptoError> {
        if self.parallelism == 0 || self.parallelism > MAX_PARALLELISM {
            return Err(CryptoError::InvalidKdfParams("parallelism out of range"));
        }
        if self.iterations == 0 || self.iterations > MAX_ITERATIONS {
            return Err(CryptoError::InvalidKdfParams("iterations out of range"));
        }
        // Argon2 needs at least 8 KiB of memory per lane.
        if self.memory_kib < 8 * self.parallelism || self.memory_kib > MAX_MEMORY_KIB {
            return Err(CryptoError::InvalidKdfParams("memory cost out of range"));
        }
        Ok(())
    }
}

/// Derives a 256-bit key from `password` and `salt` with Argon2id (v1.3).
///
/// # Errors
/// - [`CryptoError::InvalidKdfParams`] if `params` are out of range.
/// - [`CryptoError::KeyDerivation`] if Argon2id fails, for example when the
///   memory cannot be allocated.
pub fn derive_key(
    password: &[u8],
    salt: &[u8; SALT_LEN],
    params: &KdfParams,
) -> Result<SecretKey, CryptoError> {
    derive_key_with_salt(password, salt, params)
}

/// Same as [`derive_key`] with a salt of any length accepted by Argon2.
/// Kept private so the vault always uses [`SALT_LEN`]-byte salts; the
/// reference test vectors use shorter ones.
fn derive_key_with_salt(
    password: &[u8],
    salt: &[u8],
    params: &KdfParams,
) -> Result<SecretKey, CryptoError> {
    params.validate()?;
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|_| CryptoError::InvalidKdfParams("rejected by Argon2"))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

    let mut key = SecretKey::zeroed();
    argon
        .hash_password_into(password, salt, key.as_mut_bytes())
        .map_err(|_| CryptoError::KeyDerivation)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Small parameters so the tests run quickly.
    const FAST: KdfParams = KdfParams {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    };

    fn hex_decode(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Test vector from the Argon2 reference implementation
    /// (phc-winner-argon2, `src/test.c`): Argon2id v1.3, t=2, m=2^16, p=1,
    /// password "password", salt "somesalt".
    #[test]
    fn matches_reference_test_vector() {
        let params = KdfParams {
            memory_kib: 1 << 16,
            iterations: 2,
            parallelism: 1,
        };
        let key = derive_key_with_salt(b"password", b"somesalt", &params).unwrap();
        assert_eq!(
            key.as_bytes().to_vec(),
            hex_decode("09316115d5cf24ed5a15a31a3ba326e5cf32edc24702987c02b6566f61913cf7")
        );
    }

    #[test]
    fn rejects_a_salt_shorter_than_argon2_minimum() {
        assert_eq!(
            derive_key_with_salt(b"password", b"short", &FAST).unwrap_err(),
            CryptoError::KeyDerivation
        );
    }

    #[test]
    fn is_deterministic() {
        let salt = [1u8; SALT_LEN];
        let a = derive_key(b"correct horse battery", &salt, &FAST).unwrap();
        let b = derive_key(b"correct horse battery", &salt, &FAST).unwrap();
        assert_eq!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn different_password_gives_different_key() {
        let salt = [1u8; SALT_LEN];
        let a = derive_key(b"correct horse battery", &salt, &FAST).unwrap();
        let b = derive_key(b"correct horse batterz", &salt, &FAST).unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn different_salt_gives_different_key() {
        let a = derive_key(b"password", &[1u8; SALT_LEN], &FAST).unwrap();
        let b = derive_key(b"password", &[2u8; SALT_LEN], &FAST).unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn different_params_give_different_key() {
        let salt = [1u8; SALT_LEN];
        let more = KdfParams {
            iterations: 2,
            ..FAST
        };
        let a = derive_key(b"password", &salt, &FAST).unwrap();
        let b = derive_key(b"password", &salt, &more).unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn accepts_an_empty_password() {
        // Password policy is enforced by the vault, not by the KDF.
        derive_key(b"", &[0u8; SALT_LEN], &FAST).unwrap();
    }

    #[test]
    fn default_params_are_valid() {
        KdfParams::DEFAULT.validate().unwrap();
    }

    #[test]
    fn rejects_out_of_range_params() {
        let cases = [
            KdfParams {
                parallelism: 0,
                ..FAST
            },
            KdfParams {
                parallelism: MAX_PARALLELISM + 1,
                memory_kib: 8 * (MAX_PARALLELISM + 1),
                ..FAST
            },
            KdfParams {
                iterations: 0,
                ..FAST
            },
            KdfParams {
                iterations: MAX_ITERATIONS + 1,
                ..FAST
            },
            KdfParams {
                memory_kib: 7,
                ..FAST
            },
            KdfParams {
                memory_kib: 8 * 4 - 1,
                parallelism: 4,
                ..FAST
            },
            KdfParams {
                memory_kib: MAX_MEMORY_KIB + 1,
                ..FAST
            },
        ];
        for params in cases {
            assert!(
                matches!(
                    derive_key(b"password", &[0u8; SALT_LEN], &params),
                    Err(CryptoError::InvalidKdfParams(_))
                ),
                "accepted {params:?}"
            );
        }
    }

    #[test]
    fn accepts_the_inclusive_bounds() {
        let cases = [
            KdfParams {
                parallelism: MAX_PARALLELISM,
                memory_kib: 8 * MAX_PARALLELISM,
                iterations: 1,
            },
            KdfParams {
                iterations: MAX_ITERATIONS,
                ..FAST
            },
            KdfParams {
                memory_kib: 8,
                parallelism: 1,
                iterations: 1,
            },
            KdfParams {
                memory_kib: MAX_MEMORY_KIB,
                ..FAST
            },
        ];
        for params in cases {
            assert_eq!(params.validate(), Ok(()), "rejected {params:?}");
        }
    }

    #[test]
    fn derives_with_the_smallest_valid_memory_per_lane() {
        let params = KdfParams {
            memory_kib: 8 * 4,
            iterations: 1,
            parallelism: 4,
        };
        let key = derive_key(b"password", &[3u8; SALT_LEN], &params).unwrap();
        assert_ne!(key.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn reports_which_parameter_is_out_of_range() {
        let cases = [
            (
                KdfParams {
                    parallelism: 0,
                    ..FAST
                },
                "parallelism out of range",
            ),
            (
                KdfParams {
                    iterations: 0,
                    ..FAST
                },
                "iterations out of range",
            ),
            (
                KdfParams {
                    memory_kib: 0,
                    ..FAST
                },
                "memory cost out of range",
            ),
        ];
        for (params, problem) in cases {
            assert_eq!(
                params.validate(),
                Err(CryptoError::InvalidKdfParams(problem))
            );
        }
    }
}
