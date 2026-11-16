//! The vault: key header, master password rules and the locked/unlocked
//! session.

pub mod header;
pub mod password_policy;
pub mod session;

pub use session::{Vault, VaultStatus};
