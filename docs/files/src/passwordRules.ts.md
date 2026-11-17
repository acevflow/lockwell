# src/passwordRules.ts

## Purpose

The master password rules, mirrored from the core to give immediate feedback while the user
types.

## Place in the architecture

[`SetupScreen`](screens/SetupScreen.tsx.md) and [`SettingsTab`](screens/SettingsTab.tsx.md)
call `masterPasswordProblem` before sending a new password to the core. The authority is
[`vault/password_policy.rs`](../src-tauri/src/vault/password_policy.rs.md), which checks
again on create and on change. The rules do not run on unlock, so a vault made under older
rules still opens.

## Public API

| Item                                              | Description                                                                |
| ------------------------------------------------- | -------------------------------------------------------------------------- |
| `MIN_MASTER_PASSWORD_CHARS = 12`                  | Minimum length, in characters.                                             |
| `MAX_MASTER_PASSWORD_CHARS = 1024`                | Maximum length, in characters.                                             |
| `charCount(text: string): number`                 | Number of Unicode code points, as the core counts them.                    |
| `masterPasswordProblem(password: string): string \| null` | The reason the password is not acceptable, or `null` if it is.     |

The messages are `Use at least 12 characters.`, `Use at most 1024 characters.` and `The
password cannot be only spaces.` The functions have no side effects and throw nothing.

## Internal structure

`charCount` spreads the string into code points (`[...text].length`) instead of using
`text.length`, which counts UTF-16 units. `masterPasswordProblem` checks, in order, the
minimum, the maximum and then whether `password.trim()` is empty. The order matches the core.

## Dependencies

- Internal: none.
- External: none.

## Usage example

Checked by running the real module with Node:

```ts
import { charCount, masterPasswordProblem } from "./passwordRules";

charCount("🔒🔒");                    // 2 (the string length is 4)
masterPasswordProblem("abc");          // "Use at least 12 characters."
masterPasswordProblem("abcdefghijkl"); // null
masterPasswordProblem(" ".repeat(20)); // "The password cannot be only spaces."
```

## Edge cases, invariants and error handling

- An emoji counts as one character, so 11 of them are too short and 12 are accepted.
- A character followed by a combining accent counts as two, because both the interface and
  the core count code points and not what the user sees as one letter.
- **Observation, not fixed:** the interface and the core do not trim exactly the same
  characters. JavaScript's `trim()` removes U+FEFF and keeps U+0085, while Rust's `trim()`
  removes U+0085 and keeps U+FEFF. Running both gave this result for 12 repeated characters:

  | Character | This file                          | Core (`validate_master_password`)     |
  | --------- | ---------------------------------- | ------------------------------------- |
  | U+0085    | accepts (`null`)                   | rejects: cannot be only spaces        |
  | U+FEFF    | rejects: cannot be only spaces     | accepts                               |
  | U+00A0    | rejects: cannot be only spaces     | rejects: cannot be only spaces        |

  The first case is harmless, because the core refuses it and the interface shows the core's
  message. The second one blocks a password the core would accept. Both are extremely
  unlikely in practice.
- The constants must stay equal to the Rust ones. Nothing checks that they agree.

## Security

- This check is convenience only. A modified interface cannot weaken the policy, because the
  core enforces it.
- The password is passed in as a string and not stored or logged here.

## Related files and tests

- Related: [`screens/SetupScreen.tsx`](screens/SetupScreen.tsx.md),
  [`screens/SettingsTab.tsx`](screens/SettingsTab.tsx.md),
  [`../src-tauri/src/vault/password_policy.rs`](../src-tauri/src/vault/password_policy.rs.md).
- Tests: [`passwordRules.test.ts`](../../../src/passwordRules.test.ts) covers the minimum, the
  maximum, counting characters and not UTF-16 units, and whitespace-only passwords.
