//! Cryptographic core of lockwell.
//!
//! Thin, tested wrappers around audited RustCrypto crates. No cryptographic
//! primitive is implemented here:
//!
//! - [`kdf`]: Argon2id key derivation from the master password.
//! - [`aead`]: XChaCha20-Poly1305 for single records.
//! - [`stream`]: the STREAM construction over XChaCha20-Poly1305 for files
//!   and backups of any size.
//! - [`keys`]: the [`keys::SecretKey`] type, wiped from memory on drop.
//! - [`random`]: random bytes and identifiers from the operating system.

pub mod aead;
pub mod kdf;
pub mod keys;
pub mod random;
pub mod stream;

use thiserror::Error;

/// Errors returned by the cryptographic core.
///
/// Messages never include key material, passwords or plaintext.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    /// The operating system random number generator failed.
    #[error("the system random number generator is unavailable")]
    Random,
    /// The key derivation parameters are outside the accepted range.
    #[error("invalid key derivation parameters: {0}")]
    InvalidKdfParams(&'static str),
    /// Argon2id failed to derive a key.
    #[error("key derivation failed")]
    KeyDerivation,
    /// A key had the wrong length.
    #[error("invalid key length")]
    InvalidKeyLength,
    /// Encryption failed (for example, the input is too long).
    #[error("encryption failed")]
    Encryption,
    /// The data failed authentication: wrong key, wrong context or tampering.
    #[error("decryption failed: wrong key or the data is corrupted")]
    Decryption,
    /// The encrypted data is shorter than its format requires.
    #[error("the encrypted data is truncated")]
    Truncated,
    /// Reading or writing a stream failed.
    #[error("input/output error: {0}")]
    Io(String),
}

impl From<std::io::Error> for CryptoError {
    fn from(error: std::io::Error) -> Self {
        CryptoError::Io(error.kind().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn io_errors_keep_only_their_kind() {
        let error = io::Error::new(io::ErrorKind::PermissionDenied, "C:\\private\\file.txt");
        let mapped = CryptoError::from(error);
        assert_eq!(
            mapped,
            CryptoError::Io(io::ErrorKind::PermissionDenied.to_string())
        );
        assert!(!mapped.to_string().contains("private"));
    }

    #[test]
    fn messages_describe_the_failure() {
        assert_eq!(
            CryptoError::InvalidKdfParams("too much memory").to_string(),
            "invalid key derivation parameters: too much memory"
        );
        assert_eq!(
            CryptoError::Decryption.to_string(),
            "decryption failed: wrong key or the data is corrupted"
        );
        assert_eq!(
            CryptoError::Truncated.to_string(),
            "the encrypted data is truncated"
        );
    }
}
