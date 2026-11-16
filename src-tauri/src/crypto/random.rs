//! Random bytes and identifiers from the operating system generator.

use super::CryptoError;

/// Length in bytes of a random identifier (128 bits).
pub const ID_LEN: usize = 16;

/// Fills `buffer` with cryptographically secure random bytes.
///
/// # Errors
/// Returns [`CryptoError::Random`] if the operating system generator fails.
pub fn fill_random(buffer: &mut [u8]) -> Result<(), CryptoError> {
    getrandom::fill(buffer).map_err(|_| CryptoError::Random)
}

/// Returns an array of `N` random bytes.
///
/// # Errors
/// Returns [`CryptoError::Random`] if the operating system generator fails.
pub fn random_array<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut bytes = [0u8; N];
    fill_random(&mut bytes)?;
    Ok(bytes)
}

/// Returns a new random identifier as 32 lowercase hexadecimal characters.
///
/// Identifiers name records and files on disk, so they reveal nothing about
/// the content they refer to.
///
/// # Errors
/// Returns [`CryptoError::Random`] if the operating system generator fails.
pub fn random_id() -> Result<String, CryptoError> {
    Ok(to_hex(&random_array::<ID_LEN>()?))
}

/// Returns `true` if `id` has the exact shape produced by [`random_id`].
pub fn is_valid_id(id: &str) -> bool {
    id.len() == ID_LEN * 2
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Encodes bytes as lowercase hexadecimal.
pub fn to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_random_changes_the_buffer() {
        let mut buffer = [0u8; 64];
        fill_random(&mut buffer).unwrap();
        assert_ne!(buffer, [0u8; 64]);
    }

    #[test]
    fn fill_random_accepts_an_empty_buffer() {
        let mut buffer = [0u8; 0];
        fill_random(&mut buffer).unwrap();
    }

    #[test]
    fn random_ids_are_valid_and_unique() {
        let a = random_id().unwrap();
        let b = random_id().unwrap();
        assert!(is_valid_id(&a));
        assert!(is_valid_id(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn is_valid_id_rejects_bad_shapes() {
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("abc"));
        assert!(!is_valid_id(&"A".repeat(32)));
        assert!(!is_valid_id(&"g".repeat(32)));
        assert!(!is_valid_id(&"0".repeat(33)));
        assert!(!is_valid_id("../../../../../../windows/system"));
        assert!(is_valid_id(&"0a".repeat(16)));
    }

    #[test]
    fn to_hex_encodes_known_values() {
        assert_eq!(to_hex(&[]), "");
        assert_eq!(to_hex(&[0x00, 0x0f, 0xa5, 0xff]), "000fa5ff");
    }
}
