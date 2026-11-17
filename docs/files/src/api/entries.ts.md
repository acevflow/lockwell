# src/api/entries.ts

## Purpose

Typed wrappers for the password entry commands: list, read, create, update and
delete.

## Place in the architecture

One function per command in
[`commands/entries.rs`](../../src-tauri/src/commands/entries.rs.md), through
[`call`](client.ts.md). [`PasswordsTab`](../screens/PasswordsTab.tsx.md) uses
them. All of them need an unlocked vault and reject with `locked` otherwise.

## Public API

| Function                                              | Command         | Resolves to / errors                                                        |
| ----------------------------------------------------- | --------------- | --------------------------------------------------------------------------- |
| `listEntries(): Promise<EntrySummary[]>`              | `list_entries`  | Entries without passwords or notes. `locked`, `corrupt`.                    |
| `getEntry(id: string): Promise<Entry>`                | `get_entry`     | One complete entry, with its password. `invalid_input`, `not_found`.        |
| `createEntry(entry: EntryInput): Promise<string>`     | `create_entry`  | The new id. `invalid_input`.                                                |
| `updateEntry(id: string, entry: EntryInput): Promise<void>` | `update_entry` | Replaces every field. `invalid_input`, `not_found`.                  |
| `deleteEntry(id: string): Promise<void>`              | `delete_entry`  | Deletes for good. `invalid_input`, `not_found`.                             |

All of them reject with an [`ApiError`](client.ts.md). The argument names match
the Rust parameters: `id` and `entry`.

## Internal structure

No logic: each function is one `call` with the command name and its arguments.

## Dependencies

- Internal: [`client.ts`](client.ts.md), [`types.ts`](types.ts.md).
- External: none directly.

## Usage example

Checked by the wrapper tests with the bridge mocked:

```ts
import { createEntry, getEntry } from "./entries";

const id = await createEntry({ title: "Mail", username: "me", password: "p", url: "", notes: "" });
const entry = await getEntry(id);   // invoke("get_entry", { id })
```

## Edge cases, invariants and error handling

- `updateEntry` replaces every field, so the caller must send the whole entry,
  not only what changed.
- The core trims the title, username and URL, and keeps the password and notes
  exactly as typed.
- Limits on field length are enforced by the core
  ([`vault/entries.rs`](../../src-tauri/src/vault/entries.rs.md)). The copy in
  [`entryRules.ts`](../entryRules.ts.md) mirrors them for early feedback.
- A malformed id gives `invalid_input`, an unknown one `not_found`.

## Security

- Only `getEntry` returns the password and the notes to the interface. The list
  carries none, so opening the list screen does not put secrets in memory.
- To copy a password without it reaching the interface, use
  [`copyEntryField`](security.ts.md) instead of `getEntry`.

## Related files and tests

- Related: [`client.ts`](client.ts.md), [`types.ts`](types.ts.md),
  [`security.ts`](security.ts.md), [`../screens/PasswordsTab.tsx`](../screens/PasswordsTab.tsx.md),
  [`../../src-tauri/src/commands/entries.rs`](../../src-tauri/src/commands/entries.rs.md).
- Tests: [`api.test.ts`](../../../../src/api/api.test.ts) checks each command name
  and arguments.
