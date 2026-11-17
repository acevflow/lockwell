//! Local SQLite storage of the vault.
//!
//! The database only ever holds encrypted values: SQLite sees random ids and
//! opaque blobs. See [`db`] for the schema.
//!
//! This layer does no encryption and no validation of record contents: the
//! `vault` module encrypts before calling in and decrypts after reading.

pub mod db;
