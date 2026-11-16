//! Rules for the master password.

use crate::error::{AppError, AppResult};

/// Minimum length of the master password, in characters.
pub const MIN_MASTER_PASSWORD_CHARS: usize = 12;
/// Maximum length of the master password, in characters. It only bounds the
/// work done on untrusted input; it is far above any practical password.
pub const MAX_MASTER_PASSWORD_CHARS: usize = 1024;

/// Checks that `password` is acceptable as a master password.
///
/// Length is counted in Unicode characters, not bytes, so that a password in
/// any script needs the same number of characters.
///
/// # Errors
/// Returns [`AppError::InvalidInput`] explaining the rule that failed.
pub fn validate_master_password(password: &str) -> AppResult<()> {
    let chars = password.chars().count();
    if chars < MIN_MASTER_PASSWORD_CHARS {
        return Err(AppError::invalid(format!(
            "the master password must have at least {MIN_MASTER_PASSWORD_CHARS} characters"
        )));
    }
    if chars > MAX_MASTER_PASSWORD_CHARS {
        return Err(AppError::invalid(format!(
            "the master password must have at most {MAX_MASTER_PASSWORD_CHARS} characters"
        )));
    }
    if password.trim().is_empty() {
        return Err(AppError::invalid(
            "the master password cannot be only spaces",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_minimum_length() {
        validate_master_password("abcdefghijkl").unwrap();
    }

    #[test]
    fn rejects_short_passwords() {
        for password in ["", "a", "abcdefghijk"] {
            assert!(matches!(
                validate_master_password(password),
                Err(AppError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn counts_characters_not_bytes() {
        // 11 characters, 22 bytes in UTF-8.
        assert!(validate_master_password("ñññññññññññ").is_err());
        // 12 characters.
        validate_master_password("ñññññññññññ1").unwrap();
    }

    #[test]
    fn enforces_the_maximum_length() {
        validate_master_password(&"a".repeat(MAX_MASTER_PASSWORD_CHARS)).unwrap();
        assert!(validate_master_password(&"a".repeat(MAX_MASTER_PASSWORD_CHARS + 1)).is_err());
    }

    #[test]
    fn rejects_only_whitespace() {
        assert!(validate_master_password(&" ".repeat(20)).is_err());
    }

    #[test]
    fn error_message_states_the_rule() {
        let message = validate_master_password("short").unwrap_err().to_string();
        assert!(message.contains("12"));
    }
}
