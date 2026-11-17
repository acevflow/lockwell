# src/components/EntryList.tsx

## Purpose

Selectable list of password entries. It shows the title and the username, never the
password.

## Place in the architecture

A presentational component used by [`PasswordsTab`](../screens/PasswordsTab.tsx.md).
The tab filters and sorts the entries
([`filterEntries`](../entryRules.ts.md), the core sorts by title) and passes the
result here. Clicking an entry calls `onSelect`, and the tab then loads the full
entry for [`EntryView`](EntryView.tsx.md).

## Public API

Default export `EntryList(props)`:

| Prop           | Type                     | Description                                              |
| -------------- | ------------------------ | -------------------------------------------------------- |
| `entries`      | `EntrySummary[]`         | Entries to show, already filtered and sorted.            |
| `selectedId`   | `string \| null`         | Id of the open entry, if any.                            |
| `onSelect`     | `(id: string) => void`   | Called with the id of the entry the user clicks.         |
| `emptyMessage` | `string`                 | Shown when `entries` is empty.                           |

It renders `p.muted` with the empty message, or `ul.item-list` (label `Entries`)
with one button per entry. The selected one has the class `item selected` and
`aria-current="true"`.

## Internal structure

A plain function component. Each row is `li` with `button.item`, a `span.item-title`
and, only if the username is not empty, a `span.item-subtitle`. The list key is the
entry id.

## Dependencies

- Internal: [`api/types.ts`](../api/types.ts.md) (`EntrySummary`).
- External: none.

## Usage example

As in [`PasswordsTab`](../screens/PasswordsTab.tsx.md):

```tsx
<EntryList
  entries={visible}
  selectedId={selectedId}
  onSelect={(id) => void select(id)}
  emptyMessage={
    entries.length === 0
      ? "No entries yet. Add your first one."
      : "No entries match your search."
  }
/>
```

Checked by the component tests.

## Edge cases, invariants and error handling

- An empty list shows the message instead of an empty `ul`.
- An entry without a username shows only its title.
- The component does not sort or filter, so it shows exactly what it receives.
- Two entries with the same title are told apart by id (the key and the selection).

## Security

`EntrySummary` carries no password or notes, so this list cannot display them even by
mistake. Titles and usernames are rendered as text, not HTML.

## Related files and tests

- Related: [`EntryView.tsx`](EntryView.tsx.md), [`EntryForm.tsx`](EntryForm.tsx.md),
  [`../screens/PasswordsTab.tsx`](../screens/PasswordsTab.tsx.md),
  [`../entryRules.ts`](../entryRules.ts.md), [`../api/types.ts`](../api/types.ts.md).
- Tests: [`entries.test.tsx`](../../../../src/components/entries.test.tsx) covers the
  empty message, listing titles and usernames, marking the selected entry and
  selecting by id.
