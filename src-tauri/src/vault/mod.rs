//! The vault: key header, master password rules, the locked/unlocked
//! session and the password entries.

pub mod entries;
pub mod files;
pub mod header;
pub mod password_policy;
pub mod session;

use std::time::{SystemTime, UNIX_EPOCH};

pub use session::{Vault, VaultStatus};

/// Current time in milliseconds since the Unix epoch (0 if the clock is
/// before 1970).
pub(crate) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}
