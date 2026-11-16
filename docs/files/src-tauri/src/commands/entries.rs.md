# src-tauri/src/commands/entries.rs

Password entry commands. All of them need an unlocked vault and return
`locked` otherwise.

| Command        | Arguments     | Result              |
| -------------- | ------------- | ------------------- |
| `list_entries` | none          | `EntrySummary[]`    |
| `get_entry`    | `id`          | `Entry`             |
| `create_entry` | `entry`       | id of the new entry |
| `update_entry` | `id`, `entry` | none                |
| `delete_entry` | `id`          | none                |
