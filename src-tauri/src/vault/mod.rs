//! The vault: key header, master password rules, the locked/unlocked
//! session and the password entries.

pub mod backup;
pub mod entries;
pub mod files;
pub mod header;
pub mod password_policy;
pub mod session;

use std::time::{SystemTime, UNIX_EPOCH};

pub use session::{Vault, VaultStatus};

use crate::crypto::CryptoError;
use crate::crypto::random::{random_array, to_hex};
use crate::error::{AppError, AppResult};

/// Random text of 16 hexadecimal digits, to name temporary files and folders.
pub(crate) fn random_suffix() -> AppResult<String> {
    Ok(to_hex(&random_array::<8>()?))
}

/// Maps a failure while decrypting a stream: file system errors stay as they
/// are and anything else means the data is damaged, described by `damaged`.
pub(crate) fn decrypt_failure(error: CryptoError, damaged: &str) -> AppError {
    match error {
        CryptoError::Io(message) => AppError::Io(message),
        _ => AppError::Corrupt(damaged.into()),
    }
}

/// Current time in milliseconds since the Unix epoch (0 if the clock is
/// before 1970).
pub(crate) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffixes_are_16_hex_digits_and_differ() {
        let first = random_suffix().unwrap();
        assert_eq!(first.len(), 16);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, random_suffix().unwrap());
    }

    #[test]
    fn decrypt_failures_keep_io_errors_and_describe_the_rest() {
        let io = decrypt_failure(CryptoError::Io("disk full".into()), "damaged");
        assert!(matches!(io, AppError::Io(message) if message == "disk full"));
        for error in [CryptoError::Decryption, CryptoError::Truncated] {
            let other = decrypt_failure(error, "damaged");
            assert!(matches!(other, AppError::Corrupt(message) if message == "damaged"));
        }
    }
}
