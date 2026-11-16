# src-tauri/src/commands/security.rs

Activity and clipboard commands.

| Command            | Arguments                         | Result |
| ------------------ | --------------------------------- | ------ |
| `touch_activity`   | none                              | none   |
| `copy_entry_field` | `id`, `field` (`username`, `password`) | none   |

`copy_entry_field` reads the entry in the core and copies the field straight
to the clipboard, so the value never passes through the interface. It is
cleared after 30 seconds if still on the clipboard. Empty fields are refused.
