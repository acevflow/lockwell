# Contributing to lockwell

Thank you for your interest in lockwell, a local-only, encrypted password and
file manager for the desktop. This guide explains how to report problems,
propose changes and get a pull request merged.

By taking part in this project you agree to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

> **Security vulnerabilities must not be reported in public issues.** See
> [Reporting vulnerabilities](#reporting-vulnerabilities) below.

## Table of contents

- [Reporting bugs](#reporting-bugs)
- [Proposing improvements](#proposing-improvements)
- [Setting up the development environment](#setting-up-the-development-environment)
- [Project conventions](#project-conventions)
- [Pull request process](#pull-request-process)
- [Reporting vulnerabilities](#reporting-vulnerabilities)
- [License](#license)

## Reporting bugs

1. Search the [existing issues](https://github.com/acevflow/lockwell/issues)
   to check that the bug has not been reported already.
2. If it is new, [open an issue](https://github.com/acevflow/lockwell/issues/new)
   and include:
   - A short, descriptive title.
   - The steps to reproduce the problem.
   - What you expected to happen and what happened instead.
   - The lockwell version or commit hash and your operating system version.
   - Screenshots or log excerpts, if they help.

**Never include real secrets or personal data in an issue**: no master
passwords, vault files, keys, stored passwords or personal documents. Use a
test vault with made-up data, and review screenshots and logs before attaching
them.

## Proposing improvements

Open an issue that describes the problem you want to solve, the change you
propose and any alternatives you considered. For large changes, please wait
for agreement in the issue before you start working on a pull request, so
your effort is not wasted.

Keep in mind the project's design principles:

- lockwell is 100% local. Features that need network connections or send
  telemetry will not be accepted.
- Cryptography uses only standard, audited libraries. Custom cryptographic
  constructions will not be accepted.
- New dependencies must be justified: explain why the dependency is needed and
  why the existing ones are not enough.

## Setting up the development environment

lockwell is a [Tauri 2](https://tauri.app/) desktop app with a Rust core and a
TypeScript and React interface. Windows is the first supported platform; macOS
and Linux will follow.

### Requirements

- [Git](https://git-scm.com/).
- The [Rust](https://www.rust-lang.org/tools/install) toolchain, installed with
  `rustup`, including `rustfmt` and `clippy`.
- [Node.js](https://nodejs.org/) with npm.
- The system dependencies listed in the official
  [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) for
  your operating system (on Windows, the Microsoft C++ Build Tools and
  WebView2).

### Getting the code

```powershell
git clone https://github.com/acevflow/lockwell.git
cd lockwell
```

### Installing, running, testing and linting

npm is the package manager. Run every command from the repository root:

```powershell
# Install the exact dependency versions from package-lock.json
npm ci

# Run the app in development mode (Vite dev server and the Tauri window)
npm run tauri dev

# Build the interface only
npm run build

# Build the desktop app in release mode
npm run tauri build

# Run all tests: Vitest for the interface, then cargo test for the core
npm test

# Lint: ESLint for TypeScript and React, clippy (warnings as errors) for Rust
npm run lint

# Check formatting with Prettier and rustfmt, or apply it
npm run format:check
npm run format

# Type check the TypeScript code
npm run typecheck
```

The tooling is:

- **Tests:** Rust tests for the core, Vitest and Testing Library for the
  interface.
- **Formatting and lint:** rustfmt and clippy for Rust, ESLint and Prettier for
  TypeScript and React.

Run `npm test`, `npm run lint`, `npm run format:check` and `npm run typecheck`
before you open a pull request.

## Project conventions

### Branches

Create one branch per task from an up-to-date `main`, named after the type of
change:

- `feat/<short-description>` for new features, for example `feat/clipboard-timeout`.
- `fix/<short-description>` for bug fixes, for example `fix/autolock-on-resume`.

Use lowercase English words separated by hyphens.

### Commits

Commits follow [Conventional Commits](https://www.conventionalcommits.org/)
and are written in English. Use one of these types:

| Type       | Use it for                                     |
| ---------- | ---------------------------------------------- |
| `feat`     | A new feature                                  |
| `fix`      | A bug fix                                      |
| `docs`     | Documentation only                             |
| `test`     | Adding or changing tests                       |
| `refactor` | Code changes that neither fix nor add features |
| `chore`    | Tooling, configuration and maintenance         |

Example: `feat: clear the clipboard 30 seconds after copying a password`.

Make one commit per completed change, and keep each commit focused.

### Code style

- Format Rust code with rustfmt and fix every clippy warning.
- Format TypeScript and React code with Prettier and fix every ESLint warning.
- Use clear names and small functions.
- Write code comments in English.
- When you change a code file, update its documentation comments in the same
  commit.
- Every cryptographic function must have tests with known test vectors and
  failure cases (for example, wrong key, tampered ciphertext or truncated
  input).
- Never write secrets, keys, real passwords or real data in code, tests, logs
  or commits.

## Pull request process

1. Fork the repository and create a branch as described in
   [Branches](#branches).
2. Make your changes in small, focused commits.
3. Run the tests and linters locally and make sure they pass.
4. Review your staged changes (`git diff --staged`) before every commit to
   make sure no secrets, keys, real data or unrelated files are included.
5. Open a pull request against `main` with a clear description of what changes
   and why, and link the related issue.
6. Wait for the CI checks to pass and address the review comments.

Pull requests are merged only when the tests and lint are green.

### Pull request checklist

- [ ] The branch name follows the `feat/...` or `fix/...` convention.
- [ ] Commit messages follow Conventional Commits and are in English.
- [ ] Tests and lint pass locally.
- [ ] New or changed behavior is covered by tests.
- [ ] Cryptographic code, if any, uses audited libraries and has tests with
      test vectors and failure cases.
- [ ] No new dependency, or the new dependency is justified in the description.
- [ ] No network connections or telemetry were added.
- [ ] Documentation comments and related docs are updated.
- [ ] `CHANGELOG.md` is updated under `Unreleased`, if the change is visible to
      users.
- [ ] No secrets, keys, real passwords or real data are included.

## Reporting vulnerabilities

If you find a security vulnerability, **do not open a public issue or pull
request**. Report it privately through
[GitHub private vulnerability reporting](https://github.com/acevflow/lockwell/security/advisories/new)
or by email to [acevflow@gmail.com](mailto:acevflow@gmail.com). See the
[security policy](SECURITY.md) for details on what to include and what to
expect.

## License

lockwell is licensed under the
[GNU General Public License v3.0 or later](LICENSE). By contributing, you agree
that your contributions are licensed under the same terms.
