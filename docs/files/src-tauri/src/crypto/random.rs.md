# src-tauri/src/crypto/random.rs

Randomness from the operating system generator (`getrandom`).

- `fill_random()` and `random_array()` produce salts, keys and nonces.
- `random_id()` returns a 128-bit identifier as 32 lowercase hex characters.
  Ids name records and files on disk and reveal nothing about their content.
- `is_valid_id()` checks the exact id shape, so ids coming from the interface
  can be validated before they are used, for example in file paths.
- `to_hex()` encodes bytes as lowercase hexadecimal.
