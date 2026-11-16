# src/components/entries.test.tsx

Tests for the entry components. `EntryForm`: every typed field and only the
editable fields of the initial entry are
submitted, empty start, field limits, the required title, save errors, the
busy state and cancelling. `EntryList`: the empty message, titles and
usernames, the selected entry and selecting by id. `EntryView`: masked
password, Show and Hide, `None` for empty fields, copy messages (and none on
failure), the last change time, and Edit and Delete.
