//! Application error type returned by every vault operation and command.
//!
//! Errors reach the interface serialized as `{ "code": "...", "message": "..." }`.
//! The `code` is stable and meant for programs; the `message` is meant for
//! people. Neither ever contains passwords, keys or decrypted content.

use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

use crate::crypto::CryptoError;

/// Errors of the vault and its commands.
///
/// Variants carry only text that is safe to show: validation messages,
/// error kinds and fixed descriptions, never secrets or decrypted data.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppError {
    /// The master password is wrong, or the vault key data is corrupted.
    /// Both cases are reported the same way on purpose.
    #[error("the master password is incorrect or the vault is damaged")]
    WrongPasswordOrCorrupt,
    /// The operation needs an unlocked vault.
    #[error("the vault is locked")]
    Locked,
    /// A vault already exists where a new one would be created.
    #[error("a vault already exists")]
    VaultExists,
    /// There is no vault to open.
    #[error("no vault has been created yet")]
    VaultNotFound,
    /// The input was rejected by validation.
    #[error("{0}")]
    InvalidInput(String),
    /// The requested item does not exist.
    #[error("{0} not found")]
    NotFound(&'static str),
    /// Stored data failed authentication or has an invalid format.
    #[error("the vault data is damaged: {0}")]
    Corrupt(String),
    /// The vault was created by a newer version of lockwell.
    #[error("the vault was created by a newer version of lockwell")]
    UnsupportedVersion,
    /// A cryptographic operation failed for a reason other than authentication.
    #[error("cryptographic error: {0}")]
    Crypto(String),
    /// The database failed.
    #[error("storage error: {0}")]
    Storage(String),
    /// A file system operation failed.
    #[error("file system error: {0}")]
    Io(String),
}

impl AppError {
    /// Stable machine-readable code for the interface.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::WrongPasswordOrCorrupt => "wrong_password",
            AppError::Locked => "locked",
            AppError::VaultExists => "vault_exists",
            AppError::VaultNotFound => "vault_not_found",
            AppError::InvalidInput(_) => "invalid_input",
            AppError::NotFound(_) => "not_found",
            AppError::Corrupt(_) => "corrupt",
            AppError::UnsupportedVersion => "unsupported_version",
            AppError::Crypto(_) => "crypto",
            AppError::Storage(_) => "storage",
            AppError::Io(_) => "io",
        }
    }

    /// Shortcut for [`AppError::InvalidInput`].
    pub fn invalid(message: impl Into<String>) -> Self {
        AppError::InvalidInput(message.into())
    }
}

/// Serializes to `{ "code": ..., "message": ... }`, the shape the interface
/// expects from a failed command (see `src/api/client.ts`).
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

/// Authentication failures and truncation become [`AppError::Corrupt`], I/O
/// errors stay I/O, and the rest becomes [`AppError::Crypto`]. Callers that
/// know a decryption failure means a wrong password map it themselves (see
/// [`AppError::WrongPasswordOrCorrupt`]).
impl From<CryptoError> for AppError {
    fn from(error: CryptoError) -> Self {
        match error {
            CryptoError::Decryption | CryptoError::Truncated => {
                AppError::Corrupt(error.to_string())
            }
            CryptoError::Io(message) => AppError::Io(message),
            other => AppError::Crypto(other.to_string()),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        // SQLite messages describe the query, never the (encrypted) values.
        AppError::Storage(error.to_string())
    }
}

/// Keeps only the error kind: the original message may include file paths.
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error.kind().to_string())
    }
}

/// Result type of the vault and its commands.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_code_and_message() {
        let json = serde_json::to_string(&AppError::Locked).unwrap();
        assert_eq!(json, r#"{"code":"locked","message":"the vault is locked"}"#);
    }

    #[test]
    fn serializes_input_messages() {
        let json = serde_json::to_value(AppError::invalid("title is required")).unwrap();
        assert_eq!(json["code"], "invalid_input");
        assert_eq!(json["message"], "title is required");
    }

    #[test]
    fn maps_authentication_failures_to_corrupt() {
        assert_eq!(AppError::from(CryptoError::Decryption).code(), "corrupt");
        assert_eq!(AppError::from(CryptoError::Truncated).code(), "corrupt");
    }

    #[test]
    fn maps_other_crypto_errors() {
        assert_eq!(AppError::from(CryptoError::Random).code(), "crypto");
        assert_eq!(
            AppError::from(CryptoError::Io("denied".into())),
            AppError::Io("denied".into())
        );
    }

    #[test]
    fn maps_io_errors_without_paths() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "C:\\secret\\path");
        let mapped = AppError::from(error);
        assert_eq!(mapped.code(), "io");
        assert!(!mapped.to_string().contains("secret"));
    }

    #[test]
    fn every_variant_has_a_stable_code() {
        let cases = [
            (AppError::WrongPasswordOrCorrupt, "wrong_password"),
            (AppError::Locked, "locked"),
            (AppError::VaultExists, "vault_exists"),
            (AppError::VaultNotFound, "vault_not_found"),
            (AppError::invalid("x"), "invalid_input"),
            (AppError::NotFound("entry"), "not_found"),
            (AppError::Corrupt("x".into()), "corrupt"),
            (AppError::UnsupportedVersion, "unsupported_version"),
            (AppError::Crypto("x".into()), "crypto"),
            (AppError::Storage("x".into()), "storage"),
            (AppError::Io("x".into()), "io"),
        ];
        for (error, code) in cases {
            assert_eq!(error.code(), code, "{error:?}");
        }
    }

    #[test]
    fn messages_name_the_missing_item_and_the_damage() {
        assert_eq!(AppError::NotFound("entry").to_string(), "entry not found");
        let json = serde_json::to_value(AppError::Corrupt("bad header".into())).unwrap();
        assert_eq!(json["code"], "corrupt");
        assert_eq!(json["message"], "the vault data is damaged: bad header");
    }

    #[test]
    fn wrong_password_and_damage_share_one_message() {
        let json = serde_json::to_value(AppError::WrongPasswordOrCorrupt).unwrap();
        assert_eq!(
            json["message"],
            "the master password is incorrect or the vault is damaged"
        );
    }

    #[test]
    fn maps_non_authentication_crypto_errors_to_crypto() {
        let errors = [
            CryptoError::InvalidKdfParams("memory"),
            CryptoError::KeyDerivation,
            CryptoError::InvalidKeyLength,
            CryptoError::Encryption,
        ];
        for error in errors {
            let message = error.to_string();
            assert_eq!(AppError::from(error), AppError::Crypto(message));
        }
    }

    #[test]
    fn maps_database_errors_to_storage() {
        let error = rusqlite::Error::QueryReturnedNoRows;
        let mapped = AppError::from(error);
        assert_eq!(mapped.code(), "storage");
        assert!(mapped.to_string().starts_with("storage error: "));
    }
}
