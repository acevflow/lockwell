# src/entryRules.ts

## Purpose

Field limits for password entries and the search used on the entry list.

## Place in the architecture

- [`EntryForm`](components/EntryForm.tsx.md) uses `ENTRY_LIMITS` for the `maxLength` of its
  inputs. [`PasswordsTab`](screens/PasswordsTab.tsx.md) uses `filterEntries` for the search
  box.
- The limits mirror the ones in
  [`vault/entries.rs`](../src-tauri/src/vault/entries.rs.md) so the form can stop the user
  early. The core remains the authority and checks every field again.
- The search runs only on the data the list holds ([`EntrySummary`](api/types.ts.md)), so
  it never needs passwords or notes.

## Public API

| Item                                                            | Description                                                                 |
| --------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `ENTRY_LIMITS`                                                  | `{ title: 200, username: 500, password: 4096, url: 2048, notes: 20000 }`, read-only, in characters. |
| `filterEntries(entries: EntrySummary[], query: string): EntrySummary[]` | Case-insensitive match of `query` on title, username and website.           |

`filterEntries` returns the same array, unchanged, when the query is blank (empty or only
spaces). Otherwise it returns a new array in the original order. It has no side effects and
throws nothing.

## Internal structure

`filterEntries` trims and lowercases the query, then keeps the entries where any of the three
fields, lowercased, contains it. Notes and passwords are not searched, because the list does
not hold them.

## Dependencies

- Internal: [`api/types.ts`](api/types.ts.md) (`EntrySummary`).
- External: none.

## Usage example

Checked by running the real module with Node:

```ts
import { ENTRY_LIMITS, filterEntries } from "./entryRules";

filterEntries(entries, "MAIL").map((e) => e.id); // ["1"], matches the title "Mail"
filterEntries(entries, "  ").length;               // 2, a blank query returns everything
filterEntries(entries, "example").length;          // 2, matches the website
ENTRY_LIMITS.password;                             // 4096
```

## Edge cases, invariants and error handling

- The query is trimmed before matching, so surrounding spaces do not matter.
- Matching is a plain substring search. There is no accent folding, and a typed `e` does not
  match `é`.
- Case folding uses `toLowerCase`, which is not locale-aware.
- The limits must stay equal to the Rust constants (`MAX_TITLE_CHARS` and the others). A
  difference does not break anything, but the form would allow more or less than the core
  accepts. Nothing checks that they agree.
- `maxLength` in the browser counts UTF-16 units, so the real character limit can be reached
  a little sooner for characters outside the Basic Multilingual Plane (see
  [`EntryForm`](components/EntryForm.tsx.md)).

## Security

The search runs entirely in memory over data the interface already has. It sends nothing to
the core and never touches passwords or notes, which stay in the core until an entry is
opened.

## Related files and tests

- Related: [`components/EntryForm.tsx`](components/EntryForm.tsx.md),
  [`screens/PasswordsTab.tsx`](screens/PasswordsTab.tsx.md),
  [`api/types.ts`](api/types.ts.md),
  [`../src-tauri/src/vault/entries.rs`](../src-tauri/src/vault/entries.rs.md).
- Tests: [`entryRules.test.ts`](../../../src/entryRules.test.ts) covers a blank query,
  matching on each field without case and no match.
