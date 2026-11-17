# Security Policy

lockwell is a local-only, encrypted password and file manager for the desktop.
Because it is designed to protect secrets, security reports are taken
seriously and handled privately. Thank you for helping keep lockwell and its
users safe.

## Supported versions

lockwell 0.1.x is a beta that has not been audited. Only the latest published
version and the `main` branch receive security fixes.

| Version                  | Supported |
| ------------------------ | --------- |
| Latest published version | Yes       |
| `main` branch            | Yes       |
| Any other branch or fork | No        |

## Reporting a vulnerability

**Do not open a public issue, discussion or pull request for a security
vulnerability.** Report it privately through one of these channels:

1. **GitHub private vulnerability reporting (preferred):**
   [Report a vulnerability](https://github.com/acevflow/lockwell/security/advisories/new)
   in the repository's Security tab.
2. **Email:** [acevflow@gmail.com](mailto:acevflow@gmail.com), with a subject
   starting with `[lockwell security]`.

### What to include

Please include as much of the following as you can:

- A clear description of the vulnerability and the component it affects.
- Step-by-step instructions to reproduce it, or a minimal proof of concept.
- The affected version or commit hash, and your operating system version.
- The impact: what an attacker could achieve and under which conditions.

**Never include real secrets or personal data** in a report: no real master
passwords, vault files, keys, passwords or personal documents. Use test vaults
and made-up data only.

## What to expect

lockwell is maintained by a single person. Every report receives a reasonable
effort, but the timelines below are goals, not guarantees.

- **Acknowledgement:** within 7 days of your report.
- **Assessment and plan:** within 30 days, including whether the report is
  accepted and how it will be addressed.
- **Coordinated disclosure:** details are published only after a fix is
  available. Please do not disclose the issue publicly before then. With your
  permission, you will be credited in the advisory.

## Scope

### In scope

Vulnerabilities in the code of this repository, for example:

- Weaknesses in how the vault is encrypted (XChaCha20-Poly1305) or how the key
  is derived from the master password (Argon2id), including misuse of the
  cryptographic libraries.
- Ways to read, modify or forge vault records, encrypted files or encrypted
  file names without the master password.
- Secrets (master password, keys, decrypted records or files) that are written
  to disk unencrypted, written to logs, or kept in memory after the vault is
  locked.
- Failures of the security controls: automatic lock after 5 minutes of
  inactivity or when Windows is locked, clearing the clipboard after
  30 seconds, or enforcement of the 12-character minimum master password.
- Any network connection or telemetry made by the app, which by design must
  make none.
- Issues in the build, CI or release process of this repository that could
  let an attacker tamper with the code or the distributed app.

### Out of scope

- Attacks that require an already compromised computer, such as malware
  running with the user's or administrator's privileges, keyloggers, or a
  debugger attached to the running app.
- Physical access to a computer while the vault is unlocked.
- Weak, reused or forgotten master passwords. There is no recovery mechanism
  by design.
- Vulnerabilities in third-party dependencies that do not affect lockwell.
  Please report those to their maintainers.
- Platforms that are not supported yet (macOS and Linux).
- Reports from automated scanners without a demonstrated, realistic impact.

## Project status

lockwell is in early development and has **not been audited**. Do not rely on
it to protect sensitive data yet.
