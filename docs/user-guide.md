# User guide

**Purpose:** explain every feature of lockwell and how to use it.
**Audience:** people who use the app. To build and start it, see
[Getting started](getting-started.md).

> [!WARNING]
> lockwell 0.1 is a beta that has **not been audited**. There is no way to
> recover a forgotten master password. Keep backups of your vault.

## Table of contents

- [Create a vault](#create-a-vault)
- [Unlock and lock](#unlock-and-lock)
- [Passwords](#passwords)
- [Files](#files)
- [Settings](#settings)
- [Limits](#limits)
- [Not available in this version](#not-available-in-this-version)
- [Troubleshooting](#troubleshooting)

## Create a vault

The first time the app starts it shows **Create your vault**.

1. Type a master password of 12 to 1024 characters and repeat it. A password
   made only of spaces is rejected.
2. Tick the box confirming that you understand there is no recovery.
3. Select **Create vault**.

The checks run in this order: length, match, then the acknowledgement. The
vault opens unlocked.

If you already have a backup, select **Restore from a backup instead** and
follow [Restore a backup](#restore-a-backup).

## Unlock and lock

- Type the master password and select **Unlock**. The field is cleared after
  every attempt. A wrong password and a damaged vault give the same message.
- Select **Lock** (top of the unlocked screen) to lock at any time.
- The vault also locks by itself:
  - after 5 minutes without keyboard, mouse or touch activity, and
  - whenever Windows is locked.

  The unlock screen then tells you why it locked.
- Locking wipes the vault key from memory, closes the database and clears the
  clipboard if it still holds something you copied from lockwell. Closing the
  app clears it too.

## Passwords

Open the **Passwords** tab. The list is on the left, sorted by title without
regard to case. The selected entry is on the right.

### Add an entry

1. Select **New entry**.
2. Fill in the fields. Only the title is required (a title made only of
   spaces is rejected); the other fields may be empty. The limits are listed in
   [Limits](#limits).

   | Field | Example (made-up data) |
   | --- | --- |
   | Title | `Example mail` |
   | Username | `user@example.com` |
   | Password | any text, shown masked |
   | Website | `https://example.com` |
   | Notes | free text |

3. Select **Add entry**.

### View, copy, edit and delete

- Select an entry in the list. The password is masked; use **Show** (and
  **Hide**) to reveal it.
- Use **Copy username** or **Copy password** to put the value on the
  clipboard. The text is copied by the core, not by the interface. The
  clipboard is cleared after 30 seconds if it still holds that text, and on
  Windows the copy is excluded from clipboard history and cloud sync.
- Select **Edit**, change the fields and select **Save changes**.
- Select **Delete** and confirm to remove the entry. This cannot be undone.

### Search

The **Search** box filters the list as you type. It matches the title,
username and website, ignoring case. It does not search notes or passwords.

## Files

Open the **Files** tab.

- **Add files** opens a native dialog where you can pick one or more files.
  Each file is encrypted in 64 KiB chunks and stored in the vault folder under
  a random name. Its real name, size and date are stored encrypted. If one file
  fails while adding several, the files added before it stay in the vault and
  the error names the file that failed.
- **Save a copy** decrypts a file to a location you choose in a native
  dialog. The original stays in the vault.
- **Delete** removes the file after you confirm.

Only one file operation runs at a time. The buttons are disabled while it
runs, and a status message tells you what is happening. File names can have up
to 255 characters.

## Settings

### Change the master password

Enter the current password, the new one and its confirmation, then select
**Change password**. The new password must meet the same rules as at setup and
must be different from the current one. Your data is not re-encrypted: only the
key that protects the vault key is replaced.

### Create a backup

Select **Create backup** in the **Encrypted backup** section and choose where
to save the file. The suggested name is `lockwell-backup-YYYY-MM-DD.lwbackup`
(the date is UTC). The backup contains the whole vault in one encrypted file.
Cancelling the dialog does nothing.

Store backups outside the computer, for example on an external drive. A
backup needs the master password the vault had when the backup was made.

### Restore a backup

Restoring replaces **everything** in the current vault.

1. Enter the master password the backup had when it was made.
2. Select **Choose backup and restore**, and pick the `.lwbackup` file.
3. If a vault already exists, confirm **Replace vault**.

After a restore the vault is left locked. Unlock it with the backup's master
password. If the file is damaged, was tampered with or the password is wrong,
the restore fails and your current vault is not changed.

## Limits

| Item | Limit |
| --- | --- |
| Master password | 12 to 1024 characters |
| Entry title | 200 characters |
| Entry username | 500 characters |
| Entry password | 4096 characters |
| Entry website | 2048 characters |
| Entry notes | 20000 characters |
| File name | 255 characters, no `/`, `\` or control characters |

## Not available in this version

- Password recovery. A forgotten master password cannot be recovered.
- A password generator, folders or tags, and importing or exporting entries.
- Clickable website links: the website is shown as text.
- Changing the auto-lock time or the clipboard time.
- Windows is the only supported platform.

## Troubleshooting

| Message or symptom | Meaning |
| --- | --- |
| "the master password is incorrect or the vault is damaged" | Wrong password, or the vault key data is damaged. The two cases are not distinguished on purpose. |
| "the vault was created by a newer version of lockwell" | Update lockwell. |
| "The vault could not be opened: …" at start | Reading the vault status failed. There is no retry button: restart the app. |
| "the vault is locked" after a file dialog | The vault locked by inactivity while the dialog was open. Unlock and repeat the action. |
| "could not add “name”: …" | That file could not be added. Earlier files in the same selection were added. |

See the [reference](reference.md#error-codes) for every error code.
