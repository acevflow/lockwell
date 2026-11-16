# src/passwordRules.ts

Master password rules mirrored from the core
(`src-tauri/src/vault/password_policy.rs`) for immediate feedback: 12 to 1024
characters counted as Unicode characters, and not only spaces.
`masterPasswordProblem()` returns the message to show, or `null`. The core
checks the rules again and remains the authority.
