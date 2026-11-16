//! Local SQLite storage of the vault.
//!
//! The database only ever holds encrypted values: SQLite sees random ids and
//! opaque blobs. See [`db`] for the schema.

pub mod db;
