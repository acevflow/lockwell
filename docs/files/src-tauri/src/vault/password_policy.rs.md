# src-tauri/src/vault/password_policy.rs

## Purpose

Defines and checks the rules a master password must meet.

## Place in the architecture

[`session.rs`](session.rs.md) calls the check when a vault is created and when
the master password is changed. It never runs on unlock, so a vault made under
older rules still opens. The interface mirrors the rules in `src/passwordRules.ts`
for instant feedback, but this file is the authority.

## Public API

| Item                                                | Description                                       |
| --------------------------------------------------- | ------------------------------------------------- |
| `MIN_MASTER_PASSWORD_CHARS: usize = 12`             | Minimum length, in characters.                    |
| `MAX_MASTER_PASSWORD_CHARS: usize = 1024`           | Maximum length, in characters.                    |
| `validate_master_password(password: &str) -> AppResult<()>` | Checks the rules.                         |

Errors: `AppError::InvalidInput` with a message that states the failed rule:

- fewer than 12 characters: `the master password must have at least 12 characters`;
- more than 1024 characters: `the master password must have at most 1024 characters`;
- only whitespace: `the master password cannot be only spaces`.

## Internal structure

One function. It counts Unicode characters (`chars().count()`), not bytes,
checks the minimum, then the maximum, then rejects a password that is empty
after `trim()`.

## Dependencies

- Internal: [`error.rs`](../error.rs.md) (`AppError`, `AppResult`).
- External: none.

## Usage example

Checked by running it against the library:

```rust
use lockwell_lib::vault::password_policy::validate_master_password;

assert!(validate_master_password("short").is_err());
// Err(InvalidInput("the master password must have at least 12 characters"))
assert!(validate_master_password("ñññññññññññ").is_err());   // 11 characters, 22 bytes
assert!(validate_master_password("ñññññññññññ1").is_ok());    // 12 characters
assert!(validate_master_password(&" ".repeat(20)).is_err());
// Err(InvalidInput("the master password cannot be only spaces"))
```

## Edge cases, invariants and error handling

- The length is in characters, so a password in any script needs the same
  number of characters. A character here is a Unicode scalar value, not a
  grapheme cluster.
- A password of 12 or more characters that contains letters and spaces is
  accepted. Only an all-whitespace password is rejected.
- The maximum only bounds the work done on untrusted input. It is far above any
  practical password.
- The order of the checks means a short all-space password reports the length
  rule first.
- No complexity rules are applied: only length and the whitespace check.

## Security

- Enforced only on creation and on password change, never on unlock.
- The error messages never include the password.
- Passing a `&str` means the caller holds the password as text. Wiping that
  text is the caller's job.

## Related files and tests

- Related: [`session.rs`](session.rs.md), [`mod.rs`](mod.rs.md),
  [`error.rs`](../error.rs.md), [`commands/backup.rs`](../commands/backup.rs.md)
  (uses `MAX_MASTER_PASSWORD_CHARS`).
- Tests: inline `tests` module in
  [`password_policy.rs`](../../../../../src-tauri/src/vault/password_policy.rs):
  minimum length, short inputs, characters versus bytes, maximum length,
  whitespace only and the message text.
