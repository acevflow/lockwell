//! XChaCha20-Poly1305 encryption of single records.
//!
//! A sealed record is `nonce (24 bytes) || ciphertext || tag (16 bytes)`.
//! The nonce is random for every encryption; with 192-bit nonces random
//! generation is safe for any realistic number of records.
//!
//! Use this module for small records held in memory. Files and backups go
//! through [`super::stream`] instead.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use zeroize::Zeroizing;

use super::CryptoError;
use super::keys::SecretKey;
use super::random::random_array;

/// Length in bytes of the XChaCha20-Poly1305 nonce.
pub const NONCE_LEN: usize = 24;
/// Length in bytes of the Poly1305 authentication tag.
pub const TAG_LEN: usize = 16;
/// Bytes a sealed record adds to its plaintext.
pub const OVERHEAD: usize = NONCE_LEN + TAG_LEN;

/// Encrypts `plaintext` bound to the associated data `aad`.
///
/// `aad` is authenticated but not encrypted: decryption only succeeds with
/// the same value, which ties a record to its context (type, id, version).
/// Callers must always pass the context, so a valid record cannot be swapped
/// for another one encrypted under the same key.
///
/// The result is `nonce || ciphertext || tag`, [`OVERHEAD`] bytes longer than
/// the plaintext.
///
/// # Errors
/// - [`CryptoError::Random`] if no nonce can be generated.
/// - [`CryptoError::Encryption`] if the input is too long for the cipher.
pub fn seal(key: &SecretKey, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let nonce = random_array::<NONCE_LEN>()?;
    seal_with_nonce(key, &nonce, plaintext, aad)
}

/// Decrypts a record produced by [`seal`] with the same key and `aad`.
///
/// The plaintext is returned in a buffer that is wiped when dropped.
///
/// # Errors
/// - [`CryptoError::Truncated`] if `sealed` is shorter than nonce plus tag.
/// - [`CryptoError::Decryption`] if the key, the `aad` or the data are wrong.
pub fn open(key: &SecretKey, sealed: &[u8], aad: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if sealed.len() < OVERHEAD {
        return Err(CryptoError::Truncated);
    }
    let (nonce, ciphertext) = sealed.split_at(NONCE_LEN);
    let nonce = XNonce::try_from(nonce).map_err(|_| CryptoError::Truncated)?;
    cipher(key)
        .decrypt(
            &nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| CryptoError::Decryption)
}

/// Encrypts with an explicit nonce. Only [`seal`] and the test vectors use it:
/// reusing a nonce with the same key breaks confidentiality.
fn seal_with_nonce(
    key: &SecretKey,
    nonce: &[u8; NONCE_LEN],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let ciphertext = cipher(key)
        .encrypt(
            &XNonce::from(*nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Encryption)?;
    let mut sealed = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    sealed.extend_from_slice(nonce);
    sealed.extend_from_slice(&ciphertext);
    Ok(sealed)
}

/// Builds the cipher for `key`. The cipher wipes its key copy on drop.
pub(crate) fn cipher(key: &SecretKey) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(&(*key.as_bytes()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        let text: String = text.split_whitespace().collect();
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    fn test_key() -> SecretKey {
        SecretKey::from_slice(&[0x42; 32]).unwrap()
    }

    /// Test vector from draft-irtf-cfrg-xchacha-03, appendix A.3.1.
    #[test]
    fn matches_xchacha_draft_test_vector() {
        let key = SecretKey::from_slice(&hex(
            "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f",
        ))
        .unwrap();
        let nonce: [u8; NONCE_LEN] = hex("404142434445464748494a4b4c4d4e4f5051525354555657")
            .try_into()
            .unwrap();
        let aad = hex("50515253c0c1c2c3c4c5c6c7");
        let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you \
only one tip for the future, sunscreen would be it.";
        let expected_ciphertext = hex(
            "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb
             731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452
             2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9
             21f9664c97637da9768812f615c68b13b52e",
        );
        let expected_tag = hex("c0875924c1c7987947deafd8780acf49");

        let sealed = seal_with_nonce(&key, &nonce, plaintext, &aad).unwrap();
        assert_eq!(&sealed[..NONCE_LEN], &nonce);
        assert_eq!(
            &sealed[NONCE_LEN..sealed.len() - TAG_LEN],
            &expected_ciphertext[..]
        );
        assert_eq!(&sealed[sealed.len() - TAG_LEN..], &expected_tag[..]);

        let opened = open(&key, &sealed, &aad).unwrap();
        assert_eq!(&opened[..], &plaintext[..]);
    }

    #[test]
    fn round_trip() {
        let key = test_key();
        let sealed = seal(&key, b"secret record", b"entry:1").unwrap();
        assert_eq!(sealed.len(), b"secret record".len() + OVERHEAD);
        assert_eq!(
            &open(&key, &sealed, b"entry:1").unwrap()[..],
            b"secret record"
        );
    }

    #[test]
    fn round_trip_empty_plaintext() {
        let key = test_key();
        let sealed = seal(&key, b"", b"").unwrap();
        assert_eq!(sealed.len(), OVERHEAD);
        assert!(open(&key, &sealed, b"").unwrap().is_empty());
    }

    /// Encrypting the same data twice gives different nonces and output.
    #[test]
    fn uses_a_fresh_nonce_each_time() {
        let key = test_key();
        let a = seal(&key, b"same", b"").unwrap();
        let b = seal(&key, b"same", b"").unwrap();
        assert_ne!(a[..NONCE_LEN], b[..NONCE_LEN]);
        assert_ne!(a, b);
    }

    #[test]
    fn wrong_key_fails() {
        let sealed = seal(&test_key(), b"data", b"").unwrap();
        let other = SecretKey::from_slice(&[0x43; 32]).unwrap();
        assert_eq!(
            open(&other, &sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn wrong_aad_fails() {
        let key = test_key();
        let sealed = seal(&key, b"data", b"entry:1").unwrap();
        assert_eq!(
            open(&key, &sealed, b"entry:2").unwrap_err(),
            CryptoError::Decryption
        );
        assert_eq!(
            open(&key, &sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    /// Flipping any single byte (nonce, ciphertext or tag) is detected.
    #[test]
    fn tampering_anywhere_fails() {
        let key = test_key();
        let sealed = seal(&key, b"some data to protect", b"ctx").unwrap();
        for index in 0..sealed.len() {
            let mut tampered = sealed.clone();
            tampered[index] ^= 0x01;
            assert_eq!(
                open(&key, &tampered, b"ctx").unwrap_err(),
                CryptoError::Decryption,
                "byte {index} was not authenticated"
            );
        }
    }

    /// Shorter than nonce plus tag is `Truncated`; a cut that keeps that
    /// minimum fails authentication instead.
    #[test]
    fn truncated_input_fails() {
        let key = test_key();
        let sealed = seal(&key, b"data", b"").unwrap();
        assert_eq!(
            open(&key, &sealed[..OVERHEAD - 1], b"").unwrap_err(),
            CryptoError::Truncated
        );
        assert_eq!(open(&key, &[], b"").unwrap_err(), CryptoError::Truncated);
        assert_eq!(
            open(&key, &sealed[..sealed.len() - 1], b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn appended_bytes_fail() {
        let key = test_key();
        let mut sealed = seal(&key, b"data", b"").unwrap();
        sealed.push(0);
        assert_eq!(
            open(&key, &sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }
}
