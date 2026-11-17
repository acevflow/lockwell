# src-tauri/src/storage/mod.rs

## Purpose

Root of the storage layer. It declares the `db` module and documents the
layer's contract.

## Place in the architecture

- The storage layer is the local SQLite persistence of the vault. It sits
  between the [`vault/`](../vault/mod.rs.md) module above and the file system
  below.
- The database only ever holds encrypted values: SQLite sees random ids and
  opaque blobs. The schema is described in [`db.rs`](db.rs.md).
- This layer does no encryption and no validation of record contents. The
  `vault` module encrypts before calling in and decrypts after reading.

## Public API

`pub mod db;`. The file defines nothing else.

## Internal structure

A single module declaration. Declared from [`lib.rs`](../lib.rs.md) as
`pub mod storage`.

## Dependencies

- Internal: [`db.rs`](db.rs.md).
- External: none.

## Usage example

The module is reached through its submodule, as shown in
[`db.rs`](db.rs.md#usage-example): `lockwell_lib::storage::db::create`.

## Edge cases, invariants and error handling

None of its own. See [`db.rs`](db.rs.md) for the errors of each function.

## Security

The layer assumes that every value it receives is already encrypted. Code that
writes plaintext here would store it unprotected on disk.

## Related files and tests

- Related: [`db.rs`](db.rs.md), [`lib.rs`](../lib.rs.md),
  [`vault/mod.rs`](../vault/mod.rs.md).
- Tests: none of its own. The behaviour is covered by the tests in
  [`db.rs`](../../../../../src-tauri/src/storage/db.rs).
