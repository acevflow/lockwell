# src-tauri/src/security/clipboard.rs

Copies secrets to the clipboard and clears them afterwards.

- `SystemClipboard` uses `arboard`. On Windows the text is excluded from the
  clipboard history (Win+V) and from cloud clipboard sync.
- `SecureClipboard::copy()` copies a value and remembers only its SHA-256
  digest and a generation number, never the value itself.
- `SecureClipboard::clear_if_unchanged()` clears the clipboard only if it still
  holds the copied value, so anything the user copied since is left alone. A
  timer passes its generation, so an old timer never clears a newer copy;
  locking passes none and clears any copied secret.
- `CLEAR_AFTER` is 30 seconds.

The `ClipboardBackend` trait lets the tests use an in-memory clipboard. They
cover copying, clearing only its own copy, generations, an unavailable
clipboard, a clear that fails (reported, not retried) and error messages that
never contain the secret.
