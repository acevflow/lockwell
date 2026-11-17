//! Chunked encryption of large data with the STREAM construction.
//!
//! Uses RustCrypto's `aead-stream` (STREAM, big-endian 32-bit counter) over
//! XChaCha20-Poly1305. The encrypted layout is:
//!
//! ```text
//! nonce prefix (19 bytes) || chunk 0 || chunk 1 || ... || last chunk
//! ```
//!
//! Every chunk except the last holds [`CHUNK_LEN`] plaintext bytes plus a
//! 16-byte tag; the last one holds 0 to [`CHUNK_LEN`] bytes plus its tag.
//! STREAM authenticates the position of each chunk and marks the last one,
//! so reordering, dropping, duplicating or truncating chunks is detected.
//!
//! Decryption writes plaintext as it goes. If it fails part way, some
//! plaintext may already be written: callers must write to a temporary
//! destination and discard it on error.

use std::io::{Read, Write};

use aead_stream::{DecryptorBE32, EncryptorBE32};
use chacha20poly1305::XChaCha20Poly1305;
use zeroize::Zeroizing;

use super::CryptoError;
use super::aead::{TAG_LEN, cipher};
use super::keys::SecretKey;
use super::random::random_array;

/// Plaintext bytes per chunk (64 KiB).
pub const CHUNK_LEN: usize = 64 * 1024;
/// Length of the random nonce prefix: 24-byte XChaCha nonce minus the
/// 5 bytes STREAM uses for its counter and last-chunk flag.
pub const NONCE_PREFIX_LEN: usize = 19;

/// The random per-stream nonce prefix, as typed by the `aead-stream` crate.
type Prefix = aead_stream::Nonce<XChaCha20Poly1305, aead_stream::StreamBE32<XChaCha20Poly1305>>;

/// Encrypts everything read from `reader` into `writer`.
///
/// Memory use is constant (two chunk buffers, wiped on drop), whatever the
/// input size. Each chunk is held back until the next read shows whether it
/// is the last one, because STREAM encrypts the final chunk differently.
///
/// `aad` is authenticated with every chunk, binding the stream to its
/// context (for example, the id of the file it belongs to).
///
/// Returns the number of plaintext bytes encrypted.
///
/// # Errors
/// - [`CryptoError::Io`] if reading or writing fails.
/// - [`CryptoError::Random`] if no nonce can be generated.
/// - [`CryptoError::Encryption`] if the stream exceeds the STREAM limits.
pub fn encrypt<R: Read, W: Write>(
    key: &SecretKey,
    aad: &[u8],
    reader: &mut R,
    writer: &mut W,
) -> Result<u64, CryptoError> {
    let prefix = random_array::<NONCE_PREFIX_LEN>()?;
    writer.write_all(&prefix)?;
    let mut encryptor = EncryptorBE32::from_aead(cipher(key), &Prefix::from(prefix));

    let mut current = new_buffer();
    let mut next = new_buffer();
    read_chunk(reader, &mut current, CHUNK_LEN)?;
    let mut total: u64 = 0;

    // A chunk is the last one when it is short or nothing follows it.
    while current.len() == CHUNK_LEN {
        read_chunk(reader, &mut next, CHUNK_LEN)?;
        if next.is_empty() {
            break;
        }
        encryptor
            .encrypt_next_in_place(aad, &mut *current)
            .map_err(|_| CryptoError::Encryption)?;
        writer.write_all(&current)?;
        total += CHUNK_LEN as u64;
        std::mem::swap(&mut current, &mut next);
    }

    total += current.len() as u64;
    encryptor
        .encrypt_last_in_place(aad, &mut *current)
        .map_err(|_| CryptoError::Encryption)?;
    writer.write_all(&current)?;
    writer.flush()?;
    Ok(total)
}

/// Decrypts a stream produced by [`encrypt`] with the same key and `aad`.
///
/// Plaintext is written chunk by chunk, so on error `writer` may already hold
/// verified chunks (see the module docs): discard the destination.
///
/// Returns the number of plaintext bytes written.
///
/// # Errors
/// - [`CryptoError::Truncated`] if the data ends before the last chunk.
/// - [`CryptoError::Decryption`] on a wrong key, wrong `aad` or any tampering
///   (including removed, reordered or appended chunks).
/// - [`CryptoError::Io`] if reading or writing fails.
pub fn decrypt<R: Read, W: Write>(
    key: &SecretKey,
    aad: &[u8],
    reader: &mut R,
    writer: &mut W,
) -> Result<u64, CryptoError> {
    let mut prefix = [0u8; NONCE_PREFIX_LEN];
    if read_full(reader, &mut prefix)? != NONCE_PREFIX_LEN {
        return Err(CryptoError::Truncated);
    }
    let mut decryptor = DecryptorBE32::from_aead(cipher(key), &Prefix::from(prefix));

    let sealed_len = CHUNK_LEN + TAG_LEN;
    let mut current = new_buffer();
    let mut next = new_buffer();
    read_chunk(reader, &mut current, sealed_len)?;
    let mut total: u64 = 0;

    while current.len() == sealed_len {
        read_chunk(reader, &mut next, sealed_len)?;
        if next.is_empty() {
            break;
        }
        decryptor
            .decrypt_next_in_place(aad, &mut *current)
            .map_err(|_| CryptoError::Decryption)?;
        writer.write_all(&current)?;
        total += current.len() as u64;
        std::mem::swap(&mut current, &mut next);
    }

    if current.len() < TAG_LEN {
        return Err(CryptoError::Truncated);
    }
    decryptor
        .decrypt_last_in_place(aad, &mut *current)
        .map_err(|_| CryptoError::Decryption)?;
    writer.write_all(&current)?;
    writer.flush()?;
    total += current.len() as u64;
    Ok(total)
}

/// Returns the encrypted size of `plaintext_len` bytes, header included.
/// Only the tests use it, to check the layout.
#[cfg(test)]
pub fn encrypted_len(plaintext_len: u64) -> u64 {
    let chunk = CHUNK_LEN as u64;
    let chunks = if plaintext_len == 0 {
        1
    } else {
        plaintext_len.div_ceil(chunk)
    };
    NONCE_PREFIX_LEN as u64 + plaintext_len + chunks * TAG_LEN as u64
}

/// A chunk buffer with room for the tag, so encrypting in place never
/// reallocates (which would leave an unwiped copy of the plaintext).
fn new_buffer() -> Zeroizing<Vec<u8>> {
    Zeroizing::new(Vec::with_capacity(CHUNK_LEN + TAG_LEN))
}

/// Replaces the content of `buffer` with up to `limit` bytes from `reader`.
fn read_chunk<R: Read>(reader: &mut R, buffer: &mut Vec<u8>, limit: usize) -> std::io::Result<()> {
    buffer.clear();
    buffer.resize(limit, 0);
    let filled = read_full(reader, buffer)?;
    buffer.truncate(filled);
    Ok(())
}

/// Reads until `buffer` is full or the reader is exhausted.
fn read_full<R: Read>(reader: &mut R, buffer: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEALED_CHUNK: usize = CHUNK_LEN + TAG_LEN;

    fn key() -> SecretKey {
        SecretKey::from_slice(&[0x24; 32]).unwrap()
    }

    /// Deterministic, non-repeating test data.
    fn data(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i * 31 % 251) as u8).collect()
    }

    fn encrypt_vec(plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let written = encrypt(&key(), aad, &mut &plaintext[..], &mut out).unwrap();
        assert_eq!(written, plaintext.len() as u64);
        out
    }

    fn decrypt_vec(sealed: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut out = Vec::new();
        decrypt(&key(), aad, &mut &sealed[..], &mut out)?;
        Ok(out)
    }

    #[test]
    fn round_trips_various_sizes() {
        for len in [
            0,
            1,
            CHUNK_LEN - 1,
            CHUNK_LEN,
            CHUNK_LEN + 1,
            2 * CHUNK_LEN,
            3 * CHUNK_LEN + 12_345,
        ] {
            let plaintext = data(len);
            let sealed = encrypt_vec(&plaintext, b"file:1");
            assert_eq!(sealed.len() as u64, encrypted_len(len as u64), "size {len}");
            assert_eq!(
                decrypt_vec(&sealed, b"file:1").unwrap(),
                plaintext,
                "size {len}"
            );
        }
    }

    #[test]
    fn same_input_encrypts_differently() {
        let plaintext = data(100);
        assert_ne!(encrypt_vec(&plaintext, b""), encrypt_vec(&plaintext, b""));
    }

    #[test]
    fn wrong_key_fails() {
        let sealed = encrypt_vec(&data(10), b"");
        let other = SecretKey::from_slice(&[0x25; 32]).unwrap();
        let mut out = Vec::new();
        assert_eq!(
            decrypt(&other, b"", &mut &sealed[..], &mut out).unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn wrong_aad_fails() {
        let sealed = encrypt_vec(&data(10), b"file:1");
        assert_eq!(
            decrypt_vec(&sealed, b"file:2").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn tampered_prefix_fails() {
        let mut sealed = encrypt_vec(&data(10), b"");
        sealed[0] ^= 1;
        assert_eq!(
            decrypt_vec(&sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn tampered_chunk_fails() {
        let mut sealed = encrypt_vec(&data(2 * CHUNK_LEN + 10), b"");
        sealed[NONCE_PREFIX_LEN + SEALED_CHUNK + 5] ^= 1;
        assert_eq!(
            decrypt_vec(&sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn reordered_chunks_fail() {
        let sealed = encrypt_vec(&data(2 * CHUNK_LEN + 10), b"");
        let start = NONCE_PREFIX_LEN;
        let mut reordered = sealed[..start].to_vec();
        reordered.extend_from_slice(&sealed[start + SEALED_CHUNK..start + 2 * SEALED_CHUNK]);
        reordered.extend_from_slice(&sealed[start..start + SEALED_CHUNK]);
        reordered.extend_from_slice(&sealed[start + 2 * SEALED_CHUNK..]);
        assert_eq!(
            decrypt_vec(&reordered, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn dropped_middle_chunk_fails() {
        let sealed = encrypt_vec(&data(3 * CHUNK_LEN + 10), b"");
        let start = NONCE_PREFIX_LEN;
        let mut dropped = sealed[..start + SEALED_CHUNK].to_vec();
        dropped.extend_from_slice(&sealed[start + 2 * SEALED_CHUNK..]);
        assert_eq!(
            decrypt_vec(&dropped, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn truncation_at_a_chunk_boundary_fails() {
        // Dropping the last chunk leaves a valid-looking full chunk, which
        // was not encrypted as the last one.
        let sealed = encrypt_vec(&data(2 * CHUNK_LEN + 10), b"");
        let cut = NONCE_PREFIX_LEN + 2 * SEALED_CHUNK;
        assert_eq!(
            decrypt_vec(&sealed[..cut], b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    /// Cutting one byte off the last chunk breaks its tag.
    #[test]
    fn truncation_inside_a_chunk_fails() {
        let sealed = encrypt_vec(&data(CHUNK_LEN + 100), b"");
        assert_eq!(
            decrypt_vec(&sealed[..sealed.len() - 1], b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    /// Inputs too short to hold a prefix, a chunk or even a tag are `Truncated`.
    #[test]
    fn missing_or_short_data_is_truncated() {
        let sealed = encrypt_vec(&data(10), b"");
        assert_eq!(decrypt_vec(&[], b"").unwrap_err(), CryptoError::Truncated);
        assert_eq!(
            decrypt_vec(&sealed[..NONCE_PREFIX_LEN - 1], b"").unwrap_err(),
            CryptoError::Truncated
        );
        assert_eq!(
            decrypt_vec(&sealed[..NONCE_PREFIX_LEN], b"").unwrap_err(),
            CryptoError::Truncated
        );
        assert_eq!(
            decrypt_vec(&sealed[..NONCE_PREFIX_LEN + TAG_LEN - 1], b"").unwrap_err(),
            CryptoError::Truncated
        );
    }

    #[test]
    fn appended_data_fails() {
        let mut sealed = encrypt_vec(&data(10), b"");
        sealed.extend_from_slice(&[0u8; 20]);
        assert_eq!(
            decrypt_vec(&sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    /// A valid chunk appended after a full-size last chunk is rejected: the old
    /// last chunk no longer decrypts as a regular one.
    #[test]
    fn appended_chunk_after_full_last_chunk_fails() {
        let mut sealed = encrypt_vec(&data(CHUNK_LEN), b"");
        let copy = sealed[NONCE_PREFIX_LEN..].to_vec();
        sealed.extend_from_slice(&copy);
        assert_eq!(
            decrypt_vec(&sealed, b"").unwrap_err(),
            CryptoError::Decryption
        );
    }

    #[test]
    fn io_errors_are_reported() {
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk failure"))
            }
        }
        let mut out = Vec::new();
        assert!(matches!(
            encrypt(&key(), b"", &mut Failing, &mut out),
            Err(CryptoError::Io(_))
        ));
        assert!(matches!(
            decrypt(&key(), b"", &mut Failing, &mut out),
            Err(CryptoError::Io(_))
        ));
    }

    #[test]
    fn encrypted_len_matches_layout() {
        let tag = TAG_LEN as u64;
        let prefix = NONCE_PREFIX_LEN as u64;
        let chunk = CHUNK_LEN as u64;
        assert_eq!(encrypted_len(0), prefix + tag);
        assert_eq!(encrypted_len(1), prefix + 1 + tag);
        assert_eq!(encrypted_len(chunk), prefix + chunk + tag);
        assert_eq!(encrypted_len(chunk + 1), prefix + chunk + 1 + 2 * tag);
    }

    /// Reader that hands out at most `step` bytes per call and is interrupted
    /// before every successful read, like a slow pipe receiving signals.
    struct Trickle<'a> {
        data: &'a [u8],
        step: usize,
        interrupt_next: bool,
    }

    impl<'a> Trickle<'a> {
        fn new(data: &'a [u8], step: usize) -> Self {
            Trickle {
                data,
                step,
                interrupt_next: true,
            }
        }
    }

    impl Read for Trickle<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if self.interrupt_next && !self.data.is_empty() {
                self.interrupt_next = false;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            self.interrupt_next = true;
            let len = self.step.min(buffer.len()).min(self.data.len());
            buffer[..len].copy_from_slice(&self.data[..len]);
            self.data = &self.data[len..];
            Ok(len)
        }
    }

    /// Writer that accepts `capacity` bytes and then fails.
    struct FullDisk {
        capacity: usize,
    }

    impl Write for FullDisk {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            if self.capacity == 0 {
                return Err(std::io::ErrorKind::StorageFull.into());
            }
            let len = buffer.len().min(self.capacity);
            self.capacity -= len;
            Ok(len)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// `Interrupted` errors and short reads must not corrupt or abort the stream.
    #[test]
    fn short_and_interrupted_reads_are_retried() {
        let plaintext = data(2 * CHUNK_LEN + 5);
        let mut sealed = Vec::new();
        let written = encrypt(
            &key(),
            b"aad",
            &mut Trickle::new(&plaintext, 7000),
            &mut sealed,
        )
        .unwrap();
        assert_eq!(written, plaintext.len() as u64);

        let mut opened = Vec::new();
        let read = decrypt(
            &key(),
            b"aad",
            &mut Trickle::new(&sealed, 9000),
            &mut opened,
        )
        .unwrap();
        assert_eq!(read, plaintext.len() as u64);
        assert_eq!(opened, plaintext);
    }

    #[test]
    fn write_errors_are_reported() {
        let plaintext = data(CHUNK_LEN + 1);
        let sealed = encrypt_vec(&plaintext, b"");

        let expected = || CryptoError::from(std::io::Error::from(std::io::ErrorKind::StorageFull));

        // Fails on the prefix or a middle chunk, then on a chunk or the last one.
        for capacity in [0, NONCE_PREFIX_LEN + 10] {
            let result = encrypt(&key(), b"", &mut &plaintext[..], &mut FullDisk { capacity });
            assert_eq!(result, Err(expected()), "capacity {capacity}");
        }
        for capacity in [10, CHUNK_LEN] {
            let result = decrypt(&key(), b"", &mut &sealed[..], &mut FullDisk { capacity });
            assert_eq!(result, Err(expected()), "capacity {capacity}");
        }
    }
}
