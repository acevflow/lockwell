//! Symmetric key type that is wiped from memory when dropped.
//!
//! Every symmetric key in the crypto layer is a [`SecretKey`], so wiping and
//! redaction are enforced in one place.

use std::fmt;

use zeroize::{Zeroize, ZeroizeOnDrop};

use super::CryptoError;
use super::random::fill_random;

/// Length in bytes of every symmetric key (256 bits).
pub const KEY_LEN: usize = 32;

/// A 256-bit symmetric key.
///
/// The bytes are overwritten with zeros when the value is dropped, and the
/// `Debug` output never shows them. `Clone` is derived: each clone is wiped on
/// its own drop, but every clone is one more copy in memory, so clone only
/// when a second owner is really needed.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey([u8; KEY_LEN]);

impl SecretKey {
    /// Generates a new random key from the operating system generator.
    ///
    /// # Errors
    /// Returns [`CryptoError::Random`] if the generator is unavailable.
    pub fn generate() -> Result<Self, CryptoError> {
        let mut key = SecretKey([0u8; KEY_LEN]);
        fill_random(&mut key.0)?;
        Ok(key)
    }

    /// Builds a key from a slice of exactly [`KEY_LEN`] bytes.
    ///
    /// The caller remains responsible for wiping its own copy of the bytes.
    ///
    /// # Errors
    /// Returns [`CryptoError::InvalidKeyLength`] if the slice has another length.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != KEY_LEN {
            return Err(CryptoError::InvalidKeyLength);
        }
        let mut key = SecretKey([0u8; KEY_LEN]);
        key.0.copy_from_slice(bytes);
        Ok(key)
    }

    /// Returns a mutable buffer to fill in place, for example as a KDF output.
    pub(crate) fn zeroed() -> Self {
        SecretKey([0u8; KEY_LEN])
    }

    /// Mutable access to the key bytes, used to fill a [`SecretKey::zeroed`] key.
    pub(crate) fn as_mut_bytes(&mut self) -> &mut [u8; KEY_LEN] {
        &mut self.0
    }

    /// Read access to the key bytes.
    ///
    /// Callers must not copy the bytes into buffers that are not wiped.
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_are_different() {
        let a = SecretKey::generate().unwrap();
        let b = SecretKey::generate().unwrap();
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn from_slice_accepts_exact_length() {
        let key = SecretKey::from_slice(&[7u8; KEY_LEN]).unwrap();
        assert_eq!(key.as_bytes(), &[7u8; KEY_LEN]);
    }

    #[test]
    fn from_slice_rejects_wrong_lengths() {
        assert_eq!(
            SecretKey::from_slice(&[0u8; KEY_LEN - 1]).unwrap_err(),
            CryptoError::InvalidKeyLength
        );
        assert_eq!(
            SecretKey::from_slice(&[0u8; KEY_LEN + 1]).unwrap_err(),
            CryptoError::InvalidKeyLength
        );
        assert_eq!(
            SecretKey::from_slice(&[]).unwrap_err(),
            CryptoError::InvalidKeyLength
        );
    }

    #[test]
    fn debug_output_is_redacted() {
        let key = SecretKey::from_slice(&[0xAB; KEY_LEN]).unwrap();
        let shown = format!("{key:?}");
        assert_eq!(shown, "SecretKey(<redacted>)");
        assert!(!shown.contains("ab") && !shown.contains("171"));
    }

    #[test]
    fn zeroize_clears_the_bytes() {
        let mut key = SecretKey::from_slice(&[0xFF; KEY_LEN]).unwrap();
        key.zeroize();
        assert_eq!(key.as_bytes(), &[0u8; KEY_LEN]);
    }
}
