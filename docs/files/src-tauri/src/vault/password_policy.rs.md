# src-tauri/src/vault/password_policy.rs

Master password rules, enforced in the core (the interface only mirrors them):

- At least 12 characters (`MIN_MASTER_PASSWORD_CHARS`), counted as Unicode
  characters rather than bytes.
- At most 1024 characters (`MAX_MASTER_PASSWORD_CHARS`), to bound the work on
  untrusted input.
- Not only whitespace.

`validate_master_password()` returns an `invalid_input` error that states the
rule that failed.
