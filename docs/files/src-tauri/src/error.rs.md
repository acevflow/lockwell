# src-tauri/src/error.rs

`AppError`, the error type of the vault and of every command, and the
`AppResult<T>` alias.

Errors reach the interface serialized as `{ "code": "...", "message": "..." }`.
The `code` is stable (`wrong_password`, `locked`, `vault_exists`,
`vault_not_found`, `invalid_input`, `not_found`, `corrupt`,
`unsupported_version`, `crypto`, `storage`, `io`); the `message` is for people.
No message includes passwords, keys, decrypted content or file paths.

A wrong master password and a damaged key header produce the same
`wrong_password` error on purpose. Conversions from `CryptoError`,
`rusqlite::Error` and `std::io::Error` are provided.

Tests check the code of every variant, the messages of `NotFound` and
`Corrupt`, and every conversion, including that I/O errors drop their paths.
