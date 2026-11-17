# src/api/types.ts

## Purpose

TypeScript types shared with the Rust core: the data the commands send and
receive, and the stable error codes.

## Place in the architecture

The field names match the core's camelCase JSON, so these types must be kept in
step with the Rust structs and with `AppError::code`. Every other file in this
folder, and most components, import them. The file contains no runtime code.

| TypeScript type          | Rust counterpart                                                                    |
| ------------------------ | ----------------------------------------------------------------------------------- |
| `VaultStatus`            | `VaultStatus` in [`vault/session.rs`](../../src-tauri/src/vault/session.rs.md)       |
| `EntryInput`             | `EntryInput` in [`vault/entries.rs`](../../src-tauri/src/vault/entries.rs.md)        |
| `EntrySummary`, `Entry`  | `EntrySummary` and `Entry` in the same file                                         |
| `FileSummary`            | `FileSummary` in [`vault/files.rs`](../../src-tauri/src/vault/files.rs.md)           |
| `ErrorCode`              | `AppError::code` in [`error.rs`](../../src-tauri/src/error.rs.md)                    |

## Public API

| Type            | Fields                                                                                  |
| --------------- | --------------------------------------------------------------------------------------- |
| `VaultStatus`   | `exists: boolean`, `unlocked: boolean`.                                                 |
| `EntryInput`    | `title`, `username`, `password`, `url`, `notes` (all `string`).                         |
| `EntrySummary`  | `id`, `title`, `username`, `url` (strings), `updatedAt: number` (milliseconds since the Unix epoch). No password or notes. |
| `Entry`         | Extends `EntryInput` with `id: string`, `createdAt: number` and `updatedAt: number`.    |
| `FileSummary`   | `id`, `name` (strings), `size: number` (bytes of the original content), `createdAt: number`. |
| `ErrorCode`     | `"wrong_password" \| "locked" \| "vault_exists" \| "vault_not_found" \| "invalid_input" \| "not_found" \| "corrupt" \| "unsupported_version" \| "crypto" \| "storage" \| "io" \| "unknown"`. |

`unknown` is the only code that does not come from the core. It marks failures
that were not command errors (see [`client.ts`](client.ts.md)).

## Internal structure

Only interfaces and one union type. Types are erased at compile time, so nothing
from this file exists at run time.

## Dependencies

- Internal: none.
- External: none.

## Usage example

Types are checked by `npm run typecheck`:

```ts
import type { Entry, EntryInput, ErrorCode } from "./types";

const input: EntryInput = { title: "Mail", username: "", password: "", url: "", notes: "" };
const code: ErrorCode = "locked";
```

## Edge cases, invariants and error handling

- The 11 codes other than `unknown` are exactly the codes of `AppError::code`.
  Adding a code in Rust requires adding it here.
- The core accepts an `EntryInput` with only `title`, filling the other fields with
  empty text. The TypeScript type requires all five, so the interface always
  sends them.
- Sizes and dates are JavaScript numbers. Sizes are exact up to 2^53 - 1 bytes,
  far above any real file, and dates are in milliseconds.
- `Entry.id` and the timestamps are produced by the core and are not part of
  the input.

## Security

- `EntrySummary` has no password or notes, mirroring the core, so listing entries
  never puts secrets in the interface. Only `Entry` carries them.
- `Entry` and `EntryInput` hold secrets as plain strings. JavaScript gives no way
  to wipe them, so they stay in memory until garbage collection.

## Related files and tests

- Related: [`client.ts`](client.ts.md), [`entries.ts`](entries.ts.md),
  [`files.ts`](files.ts.md), [`vault.ts`](vault.ts.md),
  [`../entryRules.ts`](../entryRules.ts.md).
- Tests: none of its own. The compiler checks the types, and the wrapper tests in
  [`api.test.ts`](../../../../src/api/api.test.ts) use them.
