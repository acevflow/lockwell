//! Vault key header: Argon2id parameters, salt and the wrapped vault key.
//!
//! The vault key (DEK) is a random 256-bit key that encrypts every record and
//! file. It is stored wrapped (encrypted) with a key derived from the master
//! password (KEK), so changing the password only re-wraps the DEK.
//!
//! Binary layout (little-endian, 108 bytes):
//!
//! ```text
//! magic "LWVH" (4) | format version u32 (4) | memory KiB u32 (4)
//! | iterations u32 (4) | parallelism u32 (4) | salt (16)
//! | wrapped key: nonce (24) + encrypted DEK (32) + tag (16)
//! ```
//!
//! The first 36 bytes are the associated data of the wrapped key, so the
//! parameters and salt cannot be changed without detection.

use zeroize::Zeroizing;

use crate::crypto::aead::{self, OVERHEAD};
use crate::crypto::kdf::{self, KdfParams, SALT_LEN};
use crate::crypto::keys::{KEY_LEN, SecretKey};
use crate::crypto::random::random_array;
use crate::error::{AppError, AppResult};

/// Magic bytes that identify a vault key header.
const MAGIC: &[u8; 4] = b"LWVH";
/// Version of the header format.
pub const FORMAT_VERSION: u32 = 1;
/// Length of the clear part: magic, version, the three Argon2id parameters
/// (3 x 4 bytes) and the salt. It is also the associated data of the wrapped key.
const PARAMS_LEN: usize = 4 + 4 + 12 + SALT_LEN;
/// Length of the wrapped key: the encrypted key plus nonce and tag.
const WRAPPED_LEN: usize = KEY_LEN + OVERHEAD;
/// Total length of an encoded header.
pub const HEADER_LEN: usize = PARAMS_LEN + WRAPPED_LEN;

/// The decoded key header of a vault.
///
/// It holds no secret by itself: the wrapped key is only useful together with
/// the master password. Decoding validates the shape; only
/// [`VaultHeader::unwrap_key`] proves the password and authenticity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultHeader {
    kdf: KdfParams,
    salt: [u8; SALT_LEN],
    wrapped_key: Vec<u8>,
}

impl VaultHeader {
    /// Generates a new vault key and wraps it with `password`.
    ///
    /// # Errors
    /// Fails if randomness or key derivation fails.
    pub fn create(password: &[u8], kdf: KdfParams) -> AppResult<(VaultHeader, SecretKey)> {
        let vault_key = SecretKey::generate()?;
        let header = VaultHeader::wrap(&vault_key, password, kdf)?;
        Ok((header, vault_key))
    }

    /// Wraps an existing vault key with `password`, using a fresh salt.
    ///
    /// Used to change the master password: the vault key stays the same, so no
    /// record or file has to be re-encrypted.
    ///
    /// # Errors
    /// Fails if randomness, key derivation or encryption fails.
    pub fn wrap(vault_key: &SecretKey, password: &[u8], kdf: KdfParams) -> AppResult<VaultHeader> {
        let salt = random_array::<SALT_LEN>()?;
        let wrapping_key = kdf::derive_key(password, &salt, &kdf)?;
        let aad = encode_params(&kdf, &salt);
        let wrapped_key = aead::seal(&wrapping_key, vault_key.as_bytes(), &aad)?;
        Ok(VaultHeader {
            kdf,
            salt,
            wrapped_key,
        })
    }

    /// Recovers the vault key with `password`.
    ///
    /// Key derivation is slow on purpose (see [`kdf::derive_key`]). A wrong
    /// password and a modified header give the same error, so the result does
    /// not reveal which one happened.
    ///
    /// # Errors
    /// - [`AppError::WrongPasswordOrCorrupt`] if the password is wrong or the
    ///   header was modified.
    /// - Other errors if key derivation itself fails.
    pub fn unwrap_key(&self, password: &[u8]) -> AppResult<SecretKey> {
        let wrapping_key = kdf::derive_key(password, &self.salt, &self.kdf)?;
        let aad = encode_params(&self.kdf, &self.salt);
        let key_bytes: Zeroizing<Vec<u8>> = aead::open(&wrapping_key, &self.wrapped_key, &aad)
            .map_err(|_| AppError::WrongPasswordOrCorrupt)?;
        Ok(SecretKey::from_slice(&key_bytes)?)
    }

    /// Encodes the header in its binary layout.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = encode_params(&self.kdf, &self.salt);
        bytes.extend_from_slice(&self.wrapped_key);
        bytes
    }

    /// Decodes and validates a header.
    ///
    /// # Errors
    /// - [`AppError::UnsupportedVersion`] for a newer format version.
    /// - [`AppError::Corrupt`] for any other invalid content, including
    ///   Argon2id parameters out of the accepted range.
    pub fn from_bytes(bytes: &[u8]) -> AppResult<VaultHeader> {
        if bytes.len() < 8 || &bytes[..4] != MAGIC {
            return Err(corrupt("missing vault header"));
        }
        let version = read_u32(bytes, 4);
        if version > FORMAT_VERSION {
            return Err(AppError::UnsupportedVersion);
        }
        if version != FORMAT_VERSION || bytes.len() != HEADER_LEN {
            return Err(corrupt("invalid vault header"));
        }
        let kdf = KdfParams {
            memory_kib: read_u32(bytes, 8),
            iterations: read_u32(bytes, 12),
            parallelism: read_u32(bytes, 16),
        };
        kdf.validate()
            .map_err(|_| corrupt("invalid key derivation parameters"))?;
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&bytes[20..PARAMS_LEN]);
        Ok(VaultHeader {
            kdf,
            salt,
            wrapped_key: bytes[PARAMS_LEN..].to_vec(),
        })
    }
}

/// Encodes the clear part of the header (everything before the wrapped key).
fn encode_params(kdf: &KdfParams, salt: &[u8; SALT_LEN]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER_LEN);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&kdf.memory_kib.to_le_bytes());
    bytes.extend_from_slice(&kdf.iterations.to_le_bytes());
    bytes.extend_from_slice(&kdf.parallelism.to_le_bytes());
    bytes.extend_from_slice(salt);
    bytes
}

/// Reads a little-endian `u32` at `offset`. The callers have already checked
/// that `bytes` is long enough.
fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    let mut word = [0u8; 4];
    word.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(word)
}

fn corrupt(message: &str) -> AppError {
    AppError::Corrupt(message.into())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Small Argon2id parameters so the tests run quickly.
    pub(crate) const FAST_KDF: KdfParams = KdfParams {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    };

    #[test]
    fn create_and_unwrap() {
        let (header, key) = VaultHeader::create(b"master password 1", FAST_KDF).unwrap();
        let unwrapped = header.unwrap_key(b"master password 1").unwrap();
        assert_eq!(unwrapped.as_bytes(), key.as_bytes());
    }

    #[test]
    fn wrong_password_is_rejected() {
        let (header, _) = VaultHeader::create(b"master password 1", FAST_KDF).unwrap();
        assert_eq!(
            header.unwrap_key(b"master password 2").unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
    }

    #[test]
    fn encoding_round_trip() {
        let (header, _) = VaultHeader::create(b"pw", FAST_KDF).unwrap();
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), HEADER_LEN);
        assert_eq!(VaultHeader::from_bytes(&bytes).unwrap(), header);
    }

    #[test]
    fn rewrap_keeps_the_key_and_changes_the_salt() {
        let (header, key) = VaultHeader::create(b"old password", FAST_KDF).unwrap();
        let rewrapped = VaultHeader::wrap(&key, b"new password", FAST_KDF).unwrap();
        assert_ne!(rewrapped.salt, header.salt);
        assert_eq!(
            rewrapped.unwrap_key(b"new password").unwrap().as_bytes(),
            key.as_bytes()
        );
        assert_eq!(
            rewrapped.unwrap_key(b"old password").unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
    }

    /// The clear part is authenticated as associated data, so changing it
    /// makes the password check fail.
    #[test]
    fn tampering_with_params_or_salt_is_detected() {
        let (header, _) = VaultHeader::create(b"pw", FAST_KDF).unwrap();
        let bytes = header.to_bytes();
        // Iterations (still in range) and every salt byte.
        for index in [12].into_iter().chain(20..PARAMS_LEN) {
            let mut tampered = bytes.clone();
            tampered[index] ^= 0x02;
            let decoded = VaultHeader::from_bytes(&tampered).unwrap();
            assert_eq!(
                decoded.unwrap_key(b"pw").unwrap_err(),
                AppError::WrongPasswordOrCorrupt,
                "byte {index}"
            );
        }
    }

    #[test]
    fn tampering_with_the_wrapped_key_is_detected() {
        let (header, _) = VaultHeader::create(b"pw", FAST_KDF).unwrap();
        let mut bytes = header.to_bytes();
        bytes[HEADER_LEN - 1] ^= 0x80;
        assert_eq!(
            VaultHeader::from_bytes(&bytes)
                .unwrap()
                .unwrap_key(b"pw")
                .unwrap_err(),
            AppError::WrongPasswordOrCorrupt
        );
    }

    #[test]
    fn invalid_encodings_are_rejected() {
        let (header, _) = VaultHeader::create(b"pw", FAST_KDF).unwrap();
        let bytes = header.to_bytes();

        assert!(matches!(
            VaultHeader::from_bytes(&[]),
            Err(AppError::Corrupt(_))
        ));
        assert!(matches!(
            VaultHeader::from_bytes(&bytes[..HEADER_LEN - 1]),
            Err(AppError::Corrupt(_))
        ));
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(matches!(
            VaultHeader::from_bytes(&longer),
            Err(AppError::Corrupt(_))
        ));

        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(matches!(
            VaultHeader::from_bytes(&bad_magic),
            Err(AppError::Corrupt(_))
        ));

        let mut old_version = bytes.clone();
        old_version[4..8].copy_from_slice(&0u32.to_le_bytes());
        assert!(matches!(
            VaultHeader::from_bytes(&old_version),
            Err(AppError::Corrupt(_))
        ));
    }

    #[test]
    fn newer_version_is_unsupported() {
        let (header, _) = VaultHeader::create(b"pw", FAST_KDF).unwrap();
        let mut bytes = header.to_bytes();
        bytes[4..8].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
        assert_eq!(
            VaultHeader::from_bytes(&bytes).unwrap_err(),
            AppError::UnsupportedVersion
        );
    }

    /// Decoding refuses oversized Argon2id parameters before any derivation runs.
    #[test]
    fn out_of_range_kdf_params_are_rejected_before_use() {
        let (header, _) = VaultHeader::create(b"pw", FAST_KDF).unwrap();
        let mut bytes = header.to_bytes();
        // Ask for 4 TiB of memory.
        bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            VaultHeader::from_bytes(&bytes),
            Err(AppError::Corrupt(_))
        ));
    }
}
