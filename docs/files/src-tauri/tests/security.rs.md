# src-tauri/tests/security.rs

Integration tests of the security guarantees: error messages without
passwords, entry data or paths, redacted keys, malicious ids, unknown input
fields, weak master passwords, and locking (manual or idle) that clears a
copied secret but keeps text the user copied.
