# src-tauri/src/vault/files.rs

Encrypted files stored in the vault.

Each file has:

- A row in the `files` table with its description (name, size, import time)
  as JSON encrypted with the vault key and bound to the file id
  (`lockwell:file:v1:<id>`).
- A blob `files/<id>.lwf` in the vault folder: the magic `LWF1` followed by
  the content encrypted in 64 KiB chunks with STREAM, bound to the file id
  (`lockwell:file-data:v1:<id>`).

The blob name is the random id, so file names and types are not visible on
disk.

`UnlockedVault` methods:

- `list_files()` (sorted by name) and `get_file()`.
- `import_file(source)` encrypts a regular file. Its name must be valid
  Unicode, at most 255 characters, with no control characters or slashes.
- `export_file(id, destination, overwrite)` decrypts to a temporary file next
  to the destination and renames it only after the whole content has been
  authenticated; a damaged blob leaves nothing behind. Without `overwrite`, an
  existing destination is refused.
- `delete_file(id)` removes the blob and the row.

Paths inside the vault folder are refused as source or destination. All
writes go through a temporary file that is renamed when complete.

Tests cover empty, small, chunk-sized and large files, that names and content
are not visible on disk, Unicode names, sorting, overwrite protection,
damaged, swapped, foreign and missing blobs, deletion, invalid sources and
destinations, malformed ids, name rules and a master password change.
