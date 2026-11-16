# src-tauri/src/vault/backup.rs

Encrypted backup of the whole vault, and restoring it.

Backup file (`.lwbackup`) layout:

```text
magic "LWBK" (4) | format version u32 (4) | vault key header (108)
| STREAM-encrypted contents
```

The key header is the vault's own, so a backup opens with the master
password the vault had when the backup was made. The contents are encrypted
with the vault key using STREAM, with the whole clear prefix as associated
data. Decrypted, they are a container: `LWBC`, then records
(`kind | name length | name | data length | data`) for the database snapshot
(`VACUUM INTO`) and every encrypted file blob, then an end marker. The blobs
are copied as they are, already encrypted.

- `UnlockedVault::create_backup(destination)` writes a backup through a
  temporary file. A missing file blob makes it fail with nothing written.
- `Vault::restore_backup(backup, password)` checks the prefix, derives the key
  with the password, then decrypts and unpacks everything into a staging
  folder next to the vault. It opens the staged database, checks that its
  header matches, decrypts every entry and file description and checks every
  blob. Only then does it lock the vault and swap the folders, keeping the old
  vault until the swap succeeds. On any error the current vault is untouched
  and the staging folder is removed. The vault is left locked.

The container parser refuses unknown records, duplicate or missing databases,
blob names that are not `<id>.lwf` (so no path traversal), truncated data and
data after the end.

Tests cover restoring over an existing vault and on a new computer, the
password at backup time, wrong passwords, truncated, tampered, extended and
foreign files, newer versions, paths inside the vault, that the content is
encrypted, a missing blob and the container parser.
