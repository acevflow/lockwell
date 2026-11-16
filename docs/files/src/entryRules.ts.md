# src/entryRules.ts

Shared entry helpers:

- `ENTRY_LIMITS`: field length limits mirrored from
  `src-tauri/src/vault/entries.rs`, used as `maxLength` in the entry form.
- `filterEntries()`: case-insensitive search on title, username and website
  of the listed entries. Search runs over the summaries, so it never needs
  passwords or notes.
